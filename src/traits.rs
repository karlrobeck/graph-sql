use async_graphql::dynamic::{Field, InputObject, InputValue, Object, Scalar, TypeRef};

/// Converts SQLite column definitions to GraphQL scalar types.
///
/// This trait provides functionality to map SQLite column types (TEXT, INTEGER, REAL, BOOLEAN, etc.)
/// to their corresponding GraphQL scalar types (String, Int, Float, Boolean).
///
/// # Examples
///
/// ```rust
/// // For a ColumnDef representing an INTEGER column
/// let scalar = column_def.to_scalar()?; // Returns Scalar::new(TypeRef::INT)
///
/// // For a ColumnDef representing a TEXT column  
/// let scalar = column_def.to_scalar()?; // Returns Scalar::new(TypeRef::STRING)
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

/// Converts SQLite column definitions to GraphQL input values.
///
/// This trait handles the conversion of database column definitions to GraphQL input values,
/// taking into account nullability constraints and default values to determine whether
/// the resulting GraphQL field should be nullable or non-nullable.
///
/// # Examples
///
/// ```rust
/// // For a NOT NULL column without default value
/// let input = column_def.to_input_value(false)?; // Non-nullable input
///
/// // Force nullable even for NOT NULL columns (useful for update mutations)
/// let input = column_def.to_input_value(true)?; // Nullable input
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
    fn to_input_value(&self, force_nullable: bool) -> async_graphql::Result<InputValue>;
}

/// Converts SQLite column definitions to GraphQL field definitions.
///
/// This trait creates GraphQL fields that represent database columns, including
/// their types and resolvers for fetching column data from the database.
///
/// # Examples
///
/// ```rust
/// // Convert a column definition to a GraphQL field
/// let field = column_def.to_field("users".to_string())?;
/// // Creates a field like: name: String! (for a NOT NULL TEXT column named "name")
/// ```
pub trait ToGraphqlFieldExt {
    /// Converts the implementor to a GraphQL field.
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the database table that owns this column.
    ///   Used by the field resolver to query the correct table.
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
/// # Examples
///
/// ```rust
/// // For a NOT NULL INTEGER column
/// let type_ref = column_def.to_type_ref()?; // Returns TypeRef::named_nn(TypeRef::INT)
///
/// // For a nullable TEXT column
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
    fn to_type_ref(&self) -> async_graphql::Result<TypeRef>;
}

/// Generates GraphQL mutation operations for database tables.
///
/// This trait provides functionality to create the three fundamental CRUD mutation operations:
/// INSERT, UPDATE, and DELETE. Each operation returns both an input object (for mutation arguments)
/// and a field definition (for the mutation schema).
///
/// # Examples
///
/// ```rust
/// // Generate insert mutation for a table
/// let (input, field) = table.to_insert_mutation()?;
/// // Creates: insert_tablename(input: insert_tablename_input!): tablename_node!
///
/// // Generate update mutation for a table  
/// let (input, field) = table.to_update_mutation()?;
/// // Creates: update_tablename(id: Int!, input: update_tablename_input!): tablename_node!
///
/// // Generate delete mutation for a table
/// let (input, field) = table.to_delete_mutation()?;
/// // Creates: delete_tablename(input: delete_tablename_input!): Boolean!
/// ```
pub trait ToGraphqlMutations {
    /// Generates an INSERT mutation for creating new records.
    ///
    /// Creates a mutation that accepts an input object containing all table columns
    /// (except auto-increment primary keys) and returns the created record.
    /// Required fields are determined by NOT NULL constraints and lack of default values.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - `InputObject`: The input type definition for the mutation arguments
    /// - `Field`: The mutation field definition with resolver
    fn to_insert_mutation(&self) -> async_graphql::Result<(InputObject, Field)>;

    /// Generates an UPDATE mutation for modifying existing records.
    ///
    /// Creates a mutation that accepts a record ID (primary key) and an input object
    /// with all table columns as optional fields. Only provided fields will be updated.
    /// Returns the updated record.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - `InputObject`: The input type definition for the mutation arguments  
    /// - `Field`: The mutation field definition with resolver
    fn to_update_mutation(&self) -> async_graphql::Result<(InputObject, Field)>;

    /// Generates a DELETE mutation for removing records.
    ///
    /// Creates a mutation that accepts a record ID (primary key) and removes
    /// the corresponding record from the database. Returns a boolean indicating success.
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
/// # Examples
///
/// ```rust
/// // Generate list query for paginated results
/// let (input, field) = table.to_list_query()?;
/// // Creates: list(input: list_tablename_input!): [tablename_node]
/// // Where input requires: { page: Int!, limit: Int! }
///
/// // Generate view query for single record
/// let (input, field) = table.to_view_query()?;  
/// // Creates: view(input: view_tablename_input!): tablename_node
/// // Where input requires: { id: Int! }
/// ```
pub trait ToGraphqlQueries {
    /// Generates a LIST query for fetching multiple records with pagination.
    ///
    /// Creates a query that accepts pagination parameters (page and limit) and returns
    /// an array of records. Implements simple offset-based pagination.
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
    /// corresponding record if found, or null if not found.
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
/// //   created_at: String
/// // }
/// ```
pub trait ToGraphqlNode {
    /// Converts the implementor to a GraphQL object representing a database record.
    ///
    /// Creates an object type with fields corresponding to each table column.
    /// Each field includes appropriate type information (nullable/non-nullable) and
    /// a resolver for fetching the column value from database query results.
    ///
    /// # Returns
    ///
    /// A `Result` containing the generated `Object` on success, or an `async_graphql::Error`
    /// if the conversion fails.
    fn to_node(&self) -> async_graphql::Result<Object>;
}

/// Orchestrates the complete GraphQL schema generation for database tables.
///
/// This trait combines all other traits to generate a complete GraphQL representation
/// of a database table, including the main object type, all mutation operations,
/// all query operations, and their corresponding input types.
///
/// # Examples
///
/// ```rust
/// // Generate complete GraphQL schema for a table
/// let (object, mutations, inputs) = table.to_object()?;
/// // Returns:
/// // - object: The main table node with embedded list/view queries
/// // - mutations: Vec of all mutation fields (insert, update, delete)  
/// // - inputs: Vec of all input object types for mutations and queries
/// ```
pub trait ToGraphqlObject {
    /// Generates a complete GraphQL schema representation of the database table.
    ///
    /// This method orchestrates the creation of:
    /// - A main object type representing table records (via `to_node()`)
    /// - LIST and VIEW query operations embedded in the main object (via `to_list_query()` and `to_view_query()`)
    /// - INSERT, UPDATE, and DELETE mutation operations (via `to_insert_mutation()`, `to_update_mutation()`, `to_delete_mutation()`)
    /// - All corresponding input object types for the operations
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - `Object`: The main table object as a node
    /// - `Vec<Field>`: List and View query field definition to be added to the schema's Query type
    /// - `Vec<Field>`: All mutation field definitions to be added to the schema's Mutation type
    /// - `Vec<InputObject>`: All input object type definitions to be registered with the schema
    fn to_object(
        &self,
    ) -> async_graphql::Result<(Object, Vec<Object>, Vec<Field>, Vec<InputObject>)>;
}
