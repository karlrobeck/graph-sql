# Architecture

graph-sql follows a database-first architecture that introspects SQLite
databases and generates GraphQL APIs automatically. This page explains how the
system works internally - from database introspection to query execution.

## Core Process Flow

```
1. Database Introspection
   ├── Query sqlite_master for CREATE TABLE statements
   ├── Parse SQL using sqlparser-rs with SQLite dialect
   ├── Extract table structure, columns, and constraints
   └── Detect foreign key relationships

2. Schema Compilation
   ├── Convert SQLite types to GraphQL scalars via traits
   ├── Generate object types for each table
   ├── Create CRUD resolvers with DataLoader optimization
   └── Build dynamic GraphQL schema

3. Query Execution
   ├── Receive GraphQL query through async-graphql
   ├── Route to appropriate resolver (list, view, insert, etc.)
   ├── Generate SQL using sea-query builder
   ├── Execute via sqlx with connection pooling
   └── Transform results back to GraphQL format
```

## Database Introspection

### SQL Discovery Process

graph-sql starts by querying SQLite's internal metadata:

```rust
// Query sqlite_master to get all CREATE TABLE statements
let tables = sqlx::query_as::<_, (String,)>(
    "SELECT sql FROM sqlite_master WHERE type='table' and name not in ('_sqlx_migrations','sqlite_sequence')",
)
.fetch_all(db)
.await?;
```

### SQL Parsing with sqlparser-rs

Each CREATE TABLE statement is parsed into a structured AST:

```rust
let sqlite_dialect = SQLiteDialect {};

let tables = tables
    .into_iter()
    .flat_map(|(sql,)| {
        // Parse SQL string into Abstract Syntax Tree
        Parser::parse_sql(&sqlite_dialect, &sql).unwrap()
    })
    .filter_map(|statement| {
        if let Statement::CreateTable(table) = statement {
            Some(table) // Extract CreateTable from AST
        } else {
            None
        }
    })
    .collect::<Vec<_>>();
```

### Structure Extraction

From the parsed AST, graph-sql extracts:

- **Table name**: Becomes GraphQL object type name
- **Column definitions**: Include name, data type, constraints
- **Primary key constraints**: Used for ID fields and relationships
- **Foreign key constraints**: Both column-level and table-level
- **Nullability constraints**: Determines GraphQL field optionality

## Schema Compilation

### Type Conversion System

graph-sql uses a trait-based system to convert SQLite structures to GraphQL:

#### Column-Level Conversion

```rust
impl ToGraphqlScalarExt for ColumnDef {
    fn to_scalar(&self) -> async_graphql::Result<Scalar> {
        match &self.data_type {
            DataType::Int(_) | DataType::Integer(_) => Scalar::new(TypeRef::INT),
            DataType::Text | DataType::Varchar(_) => Scalar::new(TypeRef::STRING),
            DataType::Real | DataType::Float(_) => Scalar::new(TypeRef::FLOAT),
            DataType::Boolean => Scalar::new(TypeRef::BOOLEAN),
            DataType::Blob(_) => Scalar::new(TypeRef::STRING), // Base64 encoded
            _ => Scalar::new(TypeRef::STRING), // Default fallback
        }
    }
}
```

#### Nullability Detection

```rust
impl ToGraphqlTypeRefExt for ColumnDef {
    fn to_type_ref(&self) -> async_graphql::Result<TypeRef> {
        let scalar = self.to_scalar()?;

        // Check for NOT NULL constraint
        if self.options.iter().any(|spec| {
            matches!(spec.option, ColumnOption::NotNull)
        }) {
            Ok(TypeRef::named_nn(scalar.type_name())) // Non-nullable
        } else {
            Ok(TypeRef::named(scalar.type_name())) // Nullable
        }
    }
}
```

### Object Type Generation

Each table becomes a GraphQL object type:

```rust
impl ToGraphqlNode for CreateTable {
    fn to_node(&self) -> async_graphql::Result<Object> {
        let mut table_node = Object::new(format!("{}_node", self.name));

        // Add all columns as fields
        for col in self.columns.iter() {
            let field = col.to_field_ext(self.name.to_string())?;
            table_node = table_node.field(field);
        }

        // Process foreign key relationships
        for constraint in self.constraints.iter() {
            if let TableConstraint::ForeignKey { columns, foreign_table, .. } = constraint {
                // Create relationship fields
                let relationship_field = create_foreign_key_field(columns, foreign_table)?;
                table_node = table_node.field(relationship_field);
            }
        }

        Ok(table_node)
    }
}
```

### CRUD Operation Generation

graph-sql automatically generates five types of operations for each table:

#### Query Operations

```rust
impl ToGraphqlQueries for CreateTable {
    // Paginated list: { posts { list(input: {page: 1, limit: 10}) } }
    fn to_list_query(&self) -> async_graphql::Result<(InputObject, Field)> {
        let input = InputObject::new(format!("list_{}_input", self.name))
            .field(InputValue::new("page", TypeRef::named_nn(TypeRef::INT)))
            .field(InputValue::new("limit", TypeRef::named_nn(TypeRef::INT)));

        let field = Field::new("list", TypeRef::named_list(format!("{}_node", self.name)), 
            move |ctx| list_resolver_ext(self.clone(), ctx));

        Ok((input, field))
    }

    // Single item: { posts { view(input: {id: 1}) } }
    fn to_view_query(&self) -> async_graphql::Result<(InputObject, Field)> {
        let input = InputObject::new(format!("view_{}_input", self.name))
            .field(InputValue::new("id", TypeRef::named_nn(TypeRef::INT)));

        let field = Field::new("view", TypeRef::named(format!("{}_node", self.name)),
            move |ctx| view_resolver_ext(self.clone(), ctx));

        Ok((input, field))
    }
}
```

#### Mutation Operations

```rust
impl ToGraphqlMutations for CreateTable {
    // Insert: mutation { insert_posts(input: {...}) }
    fn to_insert_mutation(&self) -> async_graphql::Result<(InputObject, Field)> {
        let mut input = InputObject::new(format!("insert_{}_input", self.name));

        // Add all columns as input fields
        for col in self.columns.iter() {
            input = input.field(col.to_input_value(false)?);
        }

        let field = Field::new(format!("insert_{}", self.name),
            TypeRef::named_nn(format!("{}_node", self.name)),
            move |ctx| insert_resolver_ext(self.clone(), ctx));

        Ok((input, field))
    }

    // Update and Delete follow similar patterns...
}
```

## Query Execution

### Resolver Architecture

Each GraphQL field is backed by a resolver that translates the request into SQL:

#### List Resolver (Paginated Queries)

```rust
pub fn list_resolver_ext(table: CreateTable, ctx: ResolverContext<'_>) -> FieldFuture<'_> {
    FieldFuture::new(async move {
        // Extract pagination parameters
        let input = ctx.args.try_get("input")?.object()?;
        let page = input.try_get("page")?.u64()?;
        let limit = input.try_get("limit")?.u64()?;

        // Generate SQL with sea-query
        let query = Query::select()
            .from(Alias::new(table.name.to_string()))
            .column(Alias::new(pk_column_name))
            .offset((page - 1) * limit)
            .limit(limit)
            .to_string(SqliteQueryBuilder);

        // Execute with sqlx
        let results = sqlx::query_as::<_, (i64,)>(&query)
            .fetch_all(db)
            .await?;

        // Convert to GraphQL format
        let graphql_results = results.into_iter()
            .map(|(id,)| serde_json::json!({"name": pk_column_name, "id": id}))
            .map(|val| Value::from_json(val).unwrap())
            .collect::<Vec<_>>();

        Ok(Some(Value::List(graphql_results)))
    })
}
```

#### View Resolver (Single Item Queries)

```rust
pub fn view_resolver_ext(table: CreateTable, ctx: ResolverContext<'_>) -> FieldFuture<'_> {
    FieldFuture::new(async move {
        // Extract ID parameter
        let id = ctx.args.get("input")?.object()?.get("id")?.i64()?;

        // Generate SQL query
        let query = Query::select()
            .from(Alias::new(table.name.to_string()))
            .column(Alias::new(pk_column_name))
            .and_where(Expr::col(Alias::new(pk_column_name)).eq(id))
            .to_string(SqliteQueryBuilder);

        // Execute and return single result
        let result = sqlx::query_as::<_, (i64,)>(&query)
            .fetch_one(db)
            .await?;

        Ok(Some(Value::from_json(serde_json::json!({
            "name": pk_column_name,
            "id": result.0
        }))?))
    })
}
```

### DataLoader Optimization

graph-sql uses DataLoader to solve the N+1 query problem:

#### The N+1 Problem

Without optimization, fetching nested data creates multiple queries:

```
// Query 1: Get posts
SELECT id FROM posts LIMIT 10;

// Query 2-11: Get each post's title (N+1 problem)
SELECT title FROM posts WHERE id = 1;
SELECT title FROM posts WHERE id = 2;
SELECT title FROM posts WHERE id = 3;
...
```

#### DataLoader Solution

```rust
pub struct ColumnRowLoader {
    pub pool: SqlitePool,
}

impl Loader<ColumnRowDef> for ColumnRowLoader {
    async fn load(&self, keys: &[ColumnRowDef]) -> Result<HashMap<ColumnRowDef, Vec<u8>>> {
        // Group requests by (table, primary_column, target_column)
        let mut grouped_keys = HashMap::new();
        for key in keys {
            let group = (key.table.clone(), key.primary_column.clone(), key.column.clone());
            grouped_keys.entry(group).or_insert_with(Vec::new).push(key.value);
        }

        // Generate batched SQL queries
        for ((table, pk_col, val_col), pk_values) in grouped_keys {
            let sql = Query::select()
                .from(table.clone())
                .exprs([
                    Expr::col(pk_col.clone()),
                    Expr::expr(Expr::col(val_col.clone()).cast_as("blob")),
                ])
                .and_where(Expr::col(pk_col.clone()).is_in(pk_values))
                .to_string(SqliteQueryBuilder);

            // Single query fetches all needed values
            let rows = sqlx::query(&sql).fetch_all(&self.pool).await?;
            // Process and cache results...
        }
    }
}
```

### Column Value Resolution

Individual field values are fetched efficiently:

```rust
pub fn column_resolver_ext(table_name: String, col: ColumnDef, ctx: ResolverContext<'_>) -> FieldFuture<'_> {
    FieldFuture::new(async move {
        let loader = ctx.data::<DataLoader<ColumnRowLoader>>()?;

        // Extract parent object info
        let parent = ctx.parent_value.as_value()?.into_json()?;
        let pk_name = parent["name"].as_str().unwrap();
        let pk_id = parent["id"].as_i64().unwrap();

        // Request value through DataLoader (automatic batching)
        let result = loader.load_one(ColumnRowDef {
            table: Alias::new(table_name),
            column: Alias::new(col.name.to_string()),
            primary_column: Alias::new(pk_name),
            value: pk_id,
        }).await?;

        // Convert raw bytes to appropriate GraphQL type
        let graphql_value = match String::from_utf8(result.clone()) {
            Ok(string_val) => {
                // Try parsing as number, boolean, or keep as string
                if let Ok(int_val) = string_val.parse::<i64>() {
                    Value::from_json(serde_json::Value::Number(serde_json::Number::from(int_val)))
                } else if let Ok(float_val) = string_val.parse::<f64>() {
                    Value::from_json(serde_json::Value::Number(serde_json::Number::from_f64(float_val).unwrap()))
                } else {
                    Value::from_json(serde_json::Value::String(string_val))
                }
            }
            Err(_) => {
                // Binary data - encode as base64
                let base64_string = base64::encode(&result);
                Value::from_json(serde_json::Value::String(base64_string))
            }
        }?;

        Ok(Some(graphql_value))
    })
}
```

### Foreign Key Resolution

Foreign key relationships are resolved with SQL JOINs:

```rust
pub fn foreign_key_resolver_ext(
    table_name: String,
    foreign_table: String, 
    referred_column: String,
    col: ColumnDef,
    ctx: ResolverContext<'_>
) -> FieldFuture<'_> {
    FieldFuture::new(async move {
        // Extract parent record info
        let parent = ctx.parent_value.as_value()?.into_json()?;
        let pk_name = parent["name"].as_str().unwrap();
        let pk_id = parent["id"].as_i64().unwrap();

        // Generate JOIN query to fetch related record
        let query = Query::select()
            .from_as(Alias::new(foreign_table.clone()), Alias::new("f"))
            .expr(Expr::cust_with_values(
                format!("json_object(?,f.{})", referred_column),
                [referred_column.clone()],
            ))
            .inner_join(
                Alias::new(table_name.clone()),
                Expr::col((Alias::new(table_name.clone()), Alias::new(col.name.to_string())))
                    .equals((Alias::new("f"), Alias::new(referred_column.clone()))),
            )
            .and_where(Expr::col((Alias::new(table_name), Alias::new(pk_name))).eq(pk_id))
            .to_string(SqliteQueryBuilder);

        // Execute JOIN and return related object
        let result = sqlx::query_as::<_, (serde_json::Value,)>(&query)
            .fetch_one(db)
            .await?;

        Ok(Some(Value::from_json(serde_json::json!({
            "name": referred_column,
            "id": result.0.as_object().unwrap().get(&referred_column).unwrap()
        }))?))
    })
}
```

### Mutation Execution

Mutations generate SQL with parameter binding:

```rust
pub fn insert_resolver_ext(table: CreateTable, ctx: ResolverContext<'_>) -> FieldFuture<'_> {
    FieldFuture::new(async move {
        let input = ctx.args.try_get("input")?.object()?;

        // Build INSERT query
        let mut query = Query::insert().into_table(Alias::new(table.name.to_string()));
        
        // Add columns and values
        let columns: Vec<_> = input.iter().map(|(name, _)| Alias::new(name.to_string())).collect();
        query = query.columns(columns);

        let mut values = vec![];
        for (key, val) in input.iter() {
            // Convert GraphQL value to SQL expression based on column type
            let col_type = table.columns.iter()
                .find(|col| col.name.to_string() == *key)
                .unwrap().data_type;
                
            values.push(val.to_simple_expr(&col_type)?);
        }

        // Execute with RETURNING clause
        let query = query
            .values(values)?
            .returning(Query::returning().column(Alias::new(pk_column_name)))
            .to_string(SqliteQueryBuilder);

        let result = sqlx::query_as::<_, (i64,)>(&query)
            .fetch_one(db)
            .await?;

        Ok(Some(Value::from_json(serde_json::json!({
            "name": pk_column_name,
            "id": result.0
        }))?))
    })
}
```

## Schema Assembly

The final step combines all generated components:

```rust
pub fn build_schema(tables: Vec<CreateTable>) -> async_graphql::Result<SchemaBuilder> {
    let mut query_object = Object::new("Query");
    let mut mutation_object = Object::new("Mutation");
    let mut all_objects = vec![];
    let mut all_inputs = vec![];

    // Process each table
    for table in tables {
        let graphql_output = table.to_object()?; // Uses all traits

        // Add nested query structure: { tablename { list(...), view(...) } }
        for query_obj in graphql_output.queries {
            query_object = query_object.field(Field::new(
                table.name.to_string(),
                TypeRef::named_nn(query_obj.type_name()),
                |_| FieldFuture::new(async move { Ok(Some(Value::Null)) }),
            ));
            all_objects.push(query_obj);
        }

        // Add flat mutation structure: insert_tablename, update_tablename, delete_tablename
        for mutation_field in graphql_output.mutations {
            mutation_object = mutation_object.field(mutation_field);
        }

        // Register object types and inputs
        all_objects.push(graphql_output.table);
        all_inputs.extend(graphql_output.inputs);
    }

    // Build final schema
    let mut schema = Schema::build(
        query_object.type_name(),
        Some(mutation_object.type_name()),
        None,
    )
    .register(query_object)
    .register(mutation_object);

    // Register all generated types
    for object in all_objects {
        schema = schema.register(object);
    }
    for input in all_inputs {
        schema = schema.register(input);
    }

    // Add DataLoader for optimization
    schema = schema.data(DataLoader::new(
        ColumnRowLoader { pool: db.clone() },
        tokio::spawn,
    ));

    Ok(schema)
}
```

This process transforms SQLite table definitions into a fully functional GraphQL
API with automatic CRUD operations, relationship traversal, and optimized query
execution.
