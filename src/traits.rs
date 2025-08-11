//! # GraphQL Conversion Traits
//!
//! This module defines a comprehensive set of traits for converting SQLite database
//! schema information into dynamic GraphQL schemas. The traits work together to
//! provide a complete mapping from database tables to GraphQL types, queries, and mutations.
//!
//! ## Architecture Overview
//!
//! The conversion process follows a layered approach:
//!
//! 1. **Column-level traits** (`ToGraphqlScalarExt`, `ToGraphqlInputValueExt`, `ToGraphqlFieldExt`, `ToGraphqlTypeRefExt`)
//!    - Convert individual SQLite columns to GraphQL scalar types, input values, fields, and type references
//!    - Handle nullability based on `NOT NULL` constraints and default values
//!    - Support automatic foreign key relationship detection and conversion
//!
//! 2. **Table-level traits** (`ToGraphqlMutations`, `ToGraphqlQueries`, `ToGraphqlNode`)
//!    - Generate complete CRUD operations (Create, Read, Update, Delete)
//!    - Create paginated list queries and single-record view queries
//!    - Build GraphQL object types representing database records
//!
//! 3. **Schema orchestration trait** (`ToGraphqlObject`)
//!    - Combines all other traits to generate complete GraphQL schema components
//!    - Produces the final types, queries, mutations, and input objects for registration
//!
//! ## Key Features
//!
//! - **Automatic type mapping**: SQLite types are automatically mapped to appropriate GraphQL scalars
//! - **Foreign key relationships**: Columns ending in `_id` with matching foreign key info become relationship fields
//! - **Nullability handling**: GraphQL field nullability reflects database constraints
//! - **Pagination support**: List queries include offset-based pagination
//! - **Complete CRUD**: Full Create, Read, Update, Delete operation generation
//! - **Dynamic schema**: All types are generated at runtime using async-graphql's dynamic API
//!
//! ## Usage Example
//!
//! ```rust
//! // Generate complete GraphQL schema for a database table
//! let (node, queries, mutations, inputs) = sqlite_table.to_object()?;
//!
//! // Register with GraphQL schema builder
//! let mut schema = Schema::build(query_type, mutation_type, None)
//!     .register(node);
//!
//! for query in queries { schema = schema.register(query); }
//! for mutation in mutations { /* add to mutation type */ }
//! for input in inputs { schema = schema.register(input); }
//! ```

use async_graphql::dynamic::{Enum, Field, InputObject, InputValue, Object, Scalar, TypeRef};
use sea_query::SimpleExpr;
use sqlparser::ast::DataType;

/// Converts SQLite column definitions to GraphQL scalar types.
///
/// This trait provides functionality to map SQLite column types to their corresponding
/// GraphQL scalar types according to the following mapping:
/// - `TEXT` → `String`
/// - `INTEGER` → `Int`
/// - `REAL`/`FLOAT` → `Float`
/// - `BOOLEAN` → `Boolean`
/// - `BLOB` → `String` (as base64 encoded string)
/// - Custom types → `String` (fallback)
///
/// # Examples
///
/// ```rust
/// // For a ColumnDef representing an INTEGER column
/// let scalar = column_def.to_scalar()?; // Returns Scalar::new(TypeRef::INT)
///
/// // For a ColumnDef representing a TEXT column  
/// let scalar = column_def.to_scalar()?; // Returns Scalar::new(TypeRef::STRING)
///
/// // For a ColumnDef representing a REAL column
/// let scalar = column_def.to_scalar()?; // Returns Scalar::new(TypeRef::FLOAT)
/// ```
pub trait ToGraphqlScalarExt {
    /// Converts the implementor to a GraphQL scalar type.
    ///
    /// # Returns
    ///
    /// A `Result` containing the generated `Scalar` on success, or an `async_graphql::Error`
    /// if the conversion fails (e.g., unable to determine the column type).
    fn to_scalar(&self) -> async_graphql::Result<Scalar>;
}

pub trait ToGraphqlEnumExt {
    fn to_enum(&self, table_name: &str) -> async_graphql::Result<Enum>;
}

/// Converts SQLite column definitions to GraphQL input values.
///
/// This trait handles the conversion of database column definitions to GraphQL input values,
/// taking into account nullability constraints and default values to determine whether
/// the resulting GraphQL field should be nullable or non-nullable.
///
/// The nullability logic is:
/// - `NOT NULL` columns without default values → Non-nullable input (`Type!`)
/// - `NOT NULL` columns with default values → Nullable input (`Type`)
/// - Nullable columns → Nullable input (`Type`)
/// - When `force_nullable` is true → Always nullable (used for update mutations)
///
/// # Examples
///
/// ```rust
/// // For a NOT NULL column without default value
/// let input = column_def.to_input_value(false)?; // Non-nullable: name: String!
///
/// // For a NOT NULL column with default value
/// let input = column_def.to_input_value(false)?; // Nullable: created_at: String
///
/// // Force nullable even for NOT NULL columns (useful for update mutations)
/// let input = column_def.to_input_value(true)?; // Always nullable: name: String
/// ```
pub trait ToGraphqlInputValueExt {
    /// Converts the implementor to a GraphQL input value.
    ///
    /// # Arguments
    ///
    /// * `force_nullable` - When `true`, forces the generated input value to be nullable
    ///   regardless of the column's NOT NULL constraint. This is useful for update mutations
    ///   where all fields should be optional.
    ///
    /// # Returns
    ///
    /// A `Result` containing the generated `InputValue` on success, or an `async_graphql::Error`
    /// if the conversion fails.
    fn to_input_value(
        &self,
        table_name: &str,
        force_nullable: bool,
    ) -> async_graphql::Result<InputValue>;
}

/// Converts SQLite column definitions to GraphQL field definitions.
///
/// This trait creates GraphQL fields that represent database columns, including
/// their types and resolvers for fetching column data from the database.
///
/// **Foreign Key Support**: When a foreign key relationship is detected (based on column
/// name ending with `_id` and matching foreign key info), the field name is automatically
/// stripped of the `_id` suffix and the field type becomes a reference to the related
/// table's node type instead of the raw scalar value.
///
/// # Examples
///
/// ```rust
/// // Convert a regular column to a GraphQL field
/// let field = column_def.to_field("users".to_string(), None)?;
/// // Creates: name: String! (for a NOT NULL TEXT column named "name")
///
/// // Convert a foreign key column to a GraphQL field  
/// let foreign_key = ForeignKeyInfo { table: "categories", from: "category_id", to: "id" };
/// let field = column_def.to_field("posts".to_string(), Some(foreign_key))?;
/// // Creates: category: category_node! (strips "_id" suffix and references the related table)
/// ```
pub trait ToGraphqlFieldExt {
    /// Converts the implementor to a GraphQL field.
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the database table that owns this column.
    ///   Used by the field resolver to query the correct table.
    /// * `f_col` - Optional foreign key information. When provided, transforms the field
    ///   into a relationship field that resolves to the related record.
    ///
    /// # Returns
    ///
    /// A `Result` containing the generated `Field` with appropriate resolver on success,
    /// or an `async_graphql::Error` if the conversion fails.

    fn to_field(&self, table_name: String) -> async_graphql::Result<Field>;
}

/// Converts SQLite column definitions to GraphQL type references.
///
/// This trait determines the appropriate GraphQL type reference based on the
/// SQLite column type and constraints, handling nullability based on NOT NULL
/// constraints and default values.
///
/// **Nullability Logic**:
/// - Columns with `NOT NULL` constraint AND no default value → Non-nullable (`Type!`)
/// - All other columns (nullable, or NOT NULL with default) → Nullable (`Type`)
///
/// # Examples
///
/// ```rust
/// // For a NOT NULL INTEGER column without default
/// let type_ref = column_def.to_type_ref()?; // Returns TypeRef::named_nn(TypeRef::INT)
///
/// // For a nullable TEXT column
/// let type_ref = column_def.to_type_ref()?; // Returns TypeRef::named(TypeRef::STRING)
///
/// // For a NOT NULL column with default value
/// let type_ref = column_def.to_type_ref()?; // Returns TypeRef::named(TypeRef::STRING)
/// ```
pub trait ToGraphqlTypeRefExt {
    /// Converts the implementor to a GraphQL type reference.
    ///
    /// # Returns
    ///
    /// A `Result` containing the generated `TypeRef` on success, or an `async_graphql::Error`
    /// if the conversion fails. The type reference will be non-nullable (`TypeRef::named_nn`)
    /// for columns with NOT NULL constraints and no default values, or nullable (`TypeRef::named`)
    /// otherwise.
    fn to_type_ref(&self, table_name: &str) -> async_graphql::Result<TypeRef>;
}

/// Generates GraphQL mutation operations for database tables.
///
/// This trait provides functionality to create the three fundamental CRUD mutation operations:
/// INSERT, UPDATE, and DELETE. Each operation returns both an input object (for mutation arguments)
/// and a field definition (for the mutation schema).
///
/// **Important Notes**:
/// - INSERT mutations exclude auto-increment primary key columns from input
/// - UPDATE mutations make all fields optional and require separate `id` argument
/// - DELETE mutations return a boolean success indicator with `rows_affected` count
/// - All mutations use the table's primary key for record identification
///
/// # Examples
///
/// ```rust
/// // Generate insert mutation for a table
/// let (input, field) = table.to_insert_mutation()?;
/// // Creates: insert_tablename(input: insert_tablename_input!): tablename_node!
/// // Input excludes primary key, includes all other columns based on nullability
///
/// // Generate update mutation for a table  
/// let (input, field) = table.to_update_mutation()?;
/// // Creates: update_tablename(id: Int!, input: update_tablename_input!): tablename_node!
/// // All input fields are optional to allow partial updates
///
/// // Generate delete mutation for a table
/// let (input, field) = table.to_delete_mutation()?;
/// // Creates: delete_tablename(input: delete_tablename_input!): Boolean!
/// // Returns: { rows_affected: Int! }
/// ```
pub trait ToGraphqlMutations {
    /// Generates an INSERT mutation for creating new records.
    ///
    /// Creates a mutation that accepts an input object containing all table columns
    /// except auto-increment primary keys. Required fields are determined by NOT NULL
    /// constraints and lack of default values. Returns the created record.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - `InputObject`: The input type definition for the mutation arguments
    /// - `Field`: The mutation field definition with resolver
    fn to_insert_mutation(&self) -> async_graphql::Result<(InputObject, Field)>;

    /// Generates an UPDATE mutation for modifying existing records.
    ///
    /// Creates a mutation that accepts a record ID (primary key) as a separate argument
    /// and an input object with all table columns as optional fields. Only provided
    /// fields will be updated. Returns the updated record.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - `InputObject`: The input type definition for the mutation arguments  
    /// - `Field`: The mutation field definition with resolver
    fn to_update_mutation(&self) -> async_graphql::Result<(InputObject, Field)>;

    /// Generates a DELETE mutation for removing records.
    ///
    /// Creates a mutation that accepts a record ID (primary key) in an input object
    /// and removes the corresponding record from the database. Returns a boolean
    /// result with `rows_affected` count.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - `InputObject`: The input type definition for the mutation arguments
    /// - `Field`: The mutation field definition with resolver  
    fn to_delete_mutation(&self) -> async_graphql::Result<(InputObject, Field)>;
}

/// Generates GraphQL query operations for database tables.
///
/// This trait provides functionality to create query operations for retrieving data:
/// LIST (for fetching multiple records with pagination) and VIEW (for fetching a single record by ID).
///
/// **Query Implementation Details**:
/// - LIST queries use simple offset-based pagination with `page` and `limit` parameters
/// - VIEW queries fetch single records by primary key ID
/// - Both return minimal data (primary key info) that gets resolved by field resolvers
/// - Query resolvers return arrays of `{name: "column_name", id: value}` objects
///
/// # Examples
///
/// ```rust
/// // Generate list query for paginated results
/// let (input, field) = table.to_list_query()?;
/// // Creates: list(input: list_tablename_input!): [tablename_node]
/// // Input type: { page: Int!, limit: Int! }
///
/// // Generate view query for single record
/// let (input, field) = table.to_view_query()?;  
/// // Creates: view(input: view_tablename_input!): tablename_node
/// // Input type: { id: Int! }
/// ```
pub trait ToGraphqlQueries {
    /// Generates a LIST query for fetching multiple records with pagination.
    ///
    /// Creates a query that accepts pagination parameters (page and limit) and returns
    /// an array of records. Uses simple offset-based pagination: `OFFSET (page-1)*limit LIMIT limit`.
    /// The resolver returns minimal record data that gets expanded by field resolvers.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - `InputObject`: The input type definition requiring `page: Int!` and `limit: Int!`
    /// - `Field`: The query field definition with resolver returning `[tablename_node]`
    fn to_list_query(&self) -> async_graphql::Result<(InputObject, Field)>;

    /// Generates a VIEW query for fetching a single record by primary key.
    ///
    /// Creates a query that accepts a record ID (primary key) and returns the
    /// corresponding record if found, or null if not found. The resolver returns
    /// minimal record data that gets expanded by field resolvers.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - `InputObject`: The input type definition requiring `id: Int!`
    /// - `Field`: The query field definition with resolver returning `tablename_node`
    fn to_view_query(&self) -> async_graphql::Result<(InputObject, Field)>;
}

/// Converts database tables to GraphQL node objects.
///
/// This trait creates the fundamental GraphQL object type that represents a database table.
/// The node object contains fields for each table column, with appropriate types and resolvers.
///
/// **Field Resolution Strategy**:
/// - Regular columns get `column_resolver` that fetches individual column values
/// - Foreign key columns get `foreign_key_resolver` that joins to related tables
/// - Field names for foreign keys are automatically stripped of `_id` suffix
/// - Each field resolver receives parent context containing `{name: "pk_column", id: pk_value}`
///
/// # Examples
///
/// ```rust
/// // Convert a table to a GraphQL node object
/// let node = table.to_node()?;
/// // Creates an object like:
/// // type tablename_node {
/// //   id: Int!
/// //   name: String!
/// //   email: String
/// //   category: category_node!  // Foreign key relationship
/// //   created_at: String
/// // }
/// ```
pub trait ToGraphqlNode {
    /// Converts the implementor to a GraphQL object representing a database record.
    ///
    /// Creates an object type with fields corresponding to each table column.
    /// Each field includes appropriate type information (nullable/non-nullable) and
    /// a resolver for fetching the column value from database query results.
    /// Foreign key relationships are automatically detected and converted to
    /// relationship fields that resolve to related records.
    ///
    /// # Returns
    ///
    /// A `Result` containing the generated `Object` on success, or an `async_graphql::Error`
    /// if the conversion fails.
    fn to_node(&self) -> async_graphql::Result<Object>;
}

/// Output struct for complete GraphQL schema generation for a database table.
///
/// Contains all components needed to register the table's schema:
/// - `table`: The main node object type representing table records
/// - `queries`: Query objects containing list and view operations
/// - `mutations`: Mutation field definitions for insert, update, delete
/// - `inputs`: Input object type definitions for queries and mutations
pub struct GraphQLObjectOutput {
    pub table: Object,
    pub queries: Vec<Object>,
    pub mutations: Vec<Field>,
    pub inputs: Vec<InputObject>,
    pub enums: Vec<Enum>,
}

/// Orchestrates the complete GraphQL schema generation for database tables.
///
/// This trait combines all other traits to generate a complete GraphQL representation
/// of a database table, including the main object type, all mutation operations,
/// all query operations, and their corresponding input types.
///
/// **Schema Architecture**:
/// - Creates a main `tablename_node` object with all table fields
/// - Generates a query object `tablename` containing `list` and `view` operations  
/// - Produces separate mutation fields for `insert_tablename`, `update_tablename`, `delete_tablename`
/// - Creates all necessary input types for mutations and queries
/// - The resulting schema follows the nested query pattern: `{ tablename { list(...) } }`
///
/// # Examples
///
/// ```rust
/// // Generate complete GraphQL schema for a table
/// let (node, queries, mutations, inputs) = table.to_object()?;
/// // Returns:
/// // - node: The main tablename_node object type
/// // - queries: Vec containing the tablename query object with list/view operations
/// // - mutations: Vec of mutation fields (insert, update, delete)  
/// // - inputs: Vec of all input object types for mutations and queries
/// ```
pub trait ToGraphqlObject {
    /// Generates a complete GraphQL schema representation of the database table.
    ///
    /// This method orchestrates the creation of:
    /// - A main object type representing table records (via `to_node()`)
    /// - A query object containing LIST and VIEW operations (via `to_list_query()` and `to_view_query()`)
    /// - INSERT, UPDATE, and DELETE mutation operations (via mutation traits)
    /// - All corresponding input object types for the operations
    ///
    /// The generated components must be registered with the GraphQL schema builder:
    /// - The node object is registered as a type
    /// - Query objects are added as fields to the root Query type
    /// - Mutation fields are added to the root Mutation type  
    /// - Input objects are registered as types
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - `Object`: The main table node object type
    /// - `Vec<Object>`: Query object containing list and view operations to be added to Query type
    /// - `Vec<Field>`: All mutation field definitions to be added to the Mutation type
    /// - `Vec<InputObject>`: All input object type definitions to be registered with the schema
    fn to_object(&self) -> async_graphql::Result<GraphQLObjectOutput>;
}

pub trait ToSimpleExpr {
    fn to_simple_expr(self, data_type: &DataType) -> async_graphql::Result<SimpleExpr>;
}
