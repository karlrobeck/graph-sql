use async_graphql::{
    Value,
    dynamic::{Field, FieldFuture, Object, Schema, SchemaBuilder, TypeRef},
};
use sea_query::{Alias, ColumnDef, Iden};
use sqlx::SqlitePool;
use tracing::{debug, info, instrument, warn};

use crate::{
    traits::ToGraphqlObject,
    types::{ColumnInfo, ForeignKeyInfo, SqliteTable, TableInfo},
};

pub mod resolvers;
pub mod traits;
pub mod types;

#[instrument(skip(db), level = "debug")]
pub async fn introspect(db: &SqlitePool) -> async_graphql::Result<SchemaBuilder> {
    debug!("Starting database introspection");

    let tables = sqlx::query_as::<_, TableInfo>(
            "SELECT name,sql FROM sqlite_master WHERE type='table' and name not in  ('_sqlx_migrations','sqlite_sequence')",
        )
        .fetch_all(db)
        .await?;

    if tables.is_empty() {
        warn!("No tables found in database");
        return Err(async_graphql::Error::new("No tables found in database"));
    }

    info!("Found {} tables in database", tables.len());
    debug!(
        "Tables: {:?}",
        tables.iter().map(|t| &t.name).collect::<Vec<_>>()
    );

    let mut sqlite_tables = Vec::new();

    // columns
    for table in tables.iter() {
        debug!("Processing table: {}", table.name);

        let columns = sqlx::query_as::<_, ColumnInfo>(
            r#"
              select name,type,"notnull",pk,dflt_value from pragma_table_info(?)
            "#,
        )
        .bind(&table.name)
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|col| {
            debug!("Processing column: {} (type: {})", col.name, col.r#type);
            let mut col_def = ColumnDef::new(Alias::new(col.name));

            match col.r#type.to_lowercase().as_str() {
                "text" => col_def.text(),
                "real" | "numeric" => col_def.float(),
                "blob" => col_def.blob(),
                "boolean" => col_def.boolean(),
                "integer" => col_def.integer(),
                _ => col_def.text(),
            };

            if col.notnull == 1 {
                col_def.not_null();
            }

            if col.pk == 1 {
                col_def.primary_key();
            }

            if col.dflt_value.is_some() {
                col_def.default("");
            }

            col_def
        })
        .collect::<Vec<_>>();

        let query = r#"
                select "table","from","to" from pragma_foreign_key_list(?)
            "#;

        let foreign_keys = sqlx::query_as::<_, ForeignKeyInfo>(query)
            .bind(table.name.clone())
            .fetch_all(db)
            .await?;

        debug!(
            "Found {} foreign keys for table {}",
            foreign_keys.len(),
            table.name
        );

        sqlite_tables.push(SqliteTable {
            table_info: table.clone(),
            column_info: columns,
            foreign_key_info: foreign_keys,
        });
    }

    debug!(
        "Processed {} tables with column and foreign key information",
        sqlite_tables.len()
    );

    debug!(
        "Processed {} tables with column and foreign key information",
        sqlite_tables.len()
    );

    let mut query_object = Object::new("Query");
    let mut mutation_object = Object::new("Mutation");

    let mut table_objects = vec![];
    let mut inputs = vec![];

    debug!("Converting tables to GraphQL objects");

    for table in sqlite_tables {
        let name = table.table_name();
        debug!("Converting table '{:?}' to GraphQL object", name);

        let graphql = table.to_object()?;

        // add query
        for query in graphql.queries {
            debug!("Adding query field for table: {:?}", name);
            query_object = query_object.field(Field::new(
                name.to_string(),
                TypeRef::named_nn(query.type_name()),
                |_| FieldFuture::new(async move { Ok(Some(Value::Null)) }),
            ));

            table_objects.push(query);
        }

        // add mutations
        for mutation in graphql.mutations.into_iter() {
            debug!("Adding mutation field for table: {:?}", name);
            mutation_object = mutation_object.field(mutation);
        }

        // register types
        table_objects.push(graphql.table);
        inputs.extend(graphql.inputs);
    }

    debug!(
        "Building GraphQL schema with {} table objects and {} inputs",
        table_objects.len(),
        inputs.len()
    );

    let mut schema = Schema::build(
        query_object.type_name(),
        Some(mutation_object.type_name()),
        None,
    )
    .register(query_object)
    .register(mutation_object);

    for object in table_objects {
        schema = schema.register(object);
    }

    for input in inputs {
        schema = schema.register(input);
    }

    info!("Successfully built GraphQL schema");
    Ok(schema.data(db.clone()))
}
