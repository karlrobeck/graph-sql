//! # Utility Functions
//!
//! This module contains helper functions used throughout the graph-sql codebase
//! for common operations like foreign key detection, string manipulation, and
//! primary key identification.

use anyhow::anyhow;
use async_graphql::dynamic::{InputObject, InputValue, TypeRef, ValueAccessor};
use sea_query::SimpleExpr;
use sqlparser::ast::{ColumnDef, ColumnOption, CreateTable, DataType, TableConstraint};
use tracing::{debug, instrument, warn};

use crate::traits::ToSimpleExpr;

/// Strips the "_id" suffix from a column name if present.
///
/// This is commonly used when converting foreign key column names to GraphQL field names.
/// For example: "user_id" becomes "user", while "email" remains "email".
///
/// # Arguments
/// * `name` - The column name to process
///
/// # Returns
/// A new string with the "_id" suffix removed if it was present
///
/// # Examples
/// ```
/// assert_eq!(strip_id_suffix("user_id"), "user");
/// assert_eq!(strip_id_suffix("email"), "email");
/// assert_eq!(strip_id_suffix("category_id"), "category");
/// ```
pub fn strip_id_suffix(name: &str) -> String {
    name.strip_suffix("_id").unwrap_or(name).to_string()
}

/// Finds the primary key column for a given table.
///
/// This function searches for primary key definitions in both column options
/// (inline UNIQUE PRIMARY KEY) and table-level constraints.
/// It supports both single-column primary keys and composite primary keys,
/// but returns only the first column for composite keys.
///
/// # Arguments
/// * `table` - The table definition to search for primary key columns
///
/// # Returns
/// `Ok(&ColumnDef)` if a primary key column is found, or an error if no primary key exists
///
/// # Errors
/// Returns an error if:
/// - No primary key constraint is found
/// - Primary key constraint exists but references non-existent columns
///
/// # Examples
/// ```
/// match find_primary_key_column(&table_def) {
///     Ok(pk_col) => println!("Primary key: {}", pk_col.name),
///     Err(e) => eprintln!("No primary key found: {}", e),
/// }
/// ```
#[instrument(skip(table), fields(table_name = %table.name), level = "debug")]
pub fn find_primary_key_column(table: &CreateTable) -> anyhow::Result<&ColumnDef> {
    debug!("Looking for primary key in table '{}'", table.name);

    // Check for explicit UNIQUE PRIMARY KEY column options
    if let Some(pk_col) = table.columns.iter().find(|col| {
        col.options.iter().any(|opt| {
            matches!(
                opt.option,
                ColumnOption::Unique {
                    is_primary: true,
                    ..
                }
            )
        })
    }) {
        debug!(
            "Found primary key column '{}' via column option",
            pk_col.name
        );
        return Ok(pk_col);
    }

    // Check for table-level PRIMARY KEY constraint
    debug!("Checking table-level PRIMARY KEY constraints");
    for constraint in &table.constraints {
        if let TableConstraint::PrimaryKey { columns, .. } = constraint {
            debug!(
                "Found table-level PRIMARY KEY constraint with {} columns",
                columns.len()
            );
            if let Some(pk_col_spec) = columns.first() {
                // Extract column name from the column field which contains an OrderByExpr
                if let sqlparser::ast::Expr::Identifier(ident) = &pk_col_spec.column.expr {
                    let column_name = ident;
                    debug!("Primary key constraint references column '{}'", column_name);

                    if let Some(pk_col) = table.columns.iter().find(|col| col.name == *column_name)
                    {
                        debug!(
                            "Found primary key column '{}' via table constraint",
                            pk_col.name
                        );
                        return Ok(pk_col);
                    } else {
                        warn!(
                            "Primary key constraint references non-existent column '{}' in table '{}'",
                            column_name, table.name
                        );
                        return Err(anyhow!(
                            "Primary key constraint references non-existent column '{}' in table '{}'",
                            column_name,
                            table.name
                        ));
                    }
                } else {
                    warn!("Primary key constraint contains non-identifier expression, skipping");
                }
            }
        }
    }

    Err(anyhow!(
        "Unable to find primary key for table '{}'",
        table.name
    ))
}

/// Validates that a string is a valid GraphQL identifier.
///
/// GraphQL identifiers must start with a letter or underscore and contain only
/// letters, digits, and underscores.
///
/// # Arguments
/// * `name` - The string to validate
///
/// # Returns
/// `true` if the string is a valid GraphQL identifier, `false` otherwise
///
/// # Examples
/// ```
/// assert!(is_valid_graphql_identifier("user"));
/// assert!(is_valid_graphql_identifier("user_id"));
/// assert!(is_valid_graphql_identifier("_internal"));
/// assert!(!is_valid_graphql_identifier("123invalid"));
/// assert!(!is_valid_graphql_identifier("with-dashes"));
/// ```
pub fn is_valid_graphql_identifier(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let mut chars = name.chars();

    // First character must be letter or underscore
    if let Some(first) = chars.next() {
        if !first.is_ascii_alphabetic() && first != '_' {
            return false;
        }
    }

    // Remaining characters must be letters, digits, or underscores
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Converts a database table/column name to a valid GraphQL identifier.
///
/// This function ensures the resulting name is a valid GraphQL identifier by:
/// - Replacing invalid characters with underscores
/// - Ensuring the name starts with a letter or underscore
/// - Converting to snake_case if needed
///
/// # Arguments
/// * `name` - The database name to convert
///
/// # Returns
/// A valid GraphQL identifier
///
/// # Examples
/// ```
/// assert_eq!(sanitize_graphql_name("user-profile"), "user_profile");
/// assert_eq!(sanitize_graphql_name("123_table"), "_123_table");
/// assert_eq!(sanitize_graphql_name("valid_name"), "valid_name");
/// ```
pub fn sanitize_graphql_name(name: &str) -> String {
    if name.is_empty() {
        return "_empty".to_string();
    }

    let mut result = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();

    // Ensure it starts with a letter or underscore
    if let Some(first) = result.chars().next() {
        if first.is_ascii_digit() {
            result = format!("_{}", result);
        }
    }

    result
}

impl ToSimpleExpr for ValueAccessor<'_> {
    fn to_simple_expr(
        self,
        data_type: &sqlparser::ast::DataType,
    ) -> async_graphql::Result<SimpleExpr> {
        match data_type {
            DataType::Text => self.string().map(Into::into),
            DataType::Float(_) => self.f64().map(Into::into),
            DataType::Int(_) => self.i64().map(Into::into),
            DataType::Blob(_) => self.string().map(Into::into),
            // enum
            DataType::Custom(name, _) => {
                if name.to_string().starts_with("enum_") {
                    self.enum_name().map(Into::into)
                } else {
                    panic!("Unsupported data type")
                }
            }
            _ => panic!("Unsupported data type"),
        }
    }
}

pub struct StringFilter {
    pub eq: Option<String>,
    pub ne: Option<String>,
    pub contains: Option<String>,
    pub r#in: Option<String>,
}

impl StringFilter {
    pub fn to_object() -> InputObject {
        InputObject::new("string_filter")
            .field(InputValue::new("eq", TypeRef::named(TypeRef::STRING)))
            .field(InputValue::new("ne", TypeRef::named(TypeRef::STRING)))
            .field(InputValue::new("contains", TypeRef::named(TypeRef::STRING)))
            .field(InputValue::new("in", TypeRef::named(TypeRef::STRING)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_id_suffix() {
        assert_eq!(strip_id_suffix("user_id"), "user");
        assert_eq!(strip_id_suffix("category_id"), "category");
        assert_eq!(strip_id_suffix("email"), "email");
        assert_eq!(strip_id_suffix("name"), "name");
        assert_eq!(strip_id_suffix("id"), "id"); // Just "id" should remain as is
    }

    #[test]
    fn test_is_valid_graphql_identifier() {
        // Valid identifiers
        assert!(is_valid_graphql_identifier("user"));
        assert!(is_valid_graphql_identifier("user_id"));
        assert!(is_valid_graphql_identifier("_internal"));
        assert!(is_valid_graphql_identifier("User123"));
        assert!(is_valid_graphql_identifier("a"));
        assert!(is_valid_graphql_identifier("_"));

        // Invalid identifiers
        assert!(!is_valid_graphql_identifier(""));
        assert!(!is_valid_graphql_identifier("123invalid"));
        assert!(!is_valid_graphql_identifier("with-dashes"));
        assert!(!is_valid_graphql_identifier("with spaces"));
        assert!(!is_valid_graphql_identifier("with.dots"));
    }

    #[test]
    fn test_sanitize_graphql_name() {
        assert_eq!(sanitize_graphql_name("user-profile"), "user_profile");
        assert_eq!(sanitize_graphql_name("123_table"), "_123_table");
        assert_eq!(sanitize_graphql_name("valid_name"), "valid_name");
        assert_eq!(sanitize_graphql_name("with spaces"), "with_spaces");
        assert_eq!(sanitize_graphql_name(""), "_empty");
        assert_eq!(sanitize_graphql_name("user.email"), "user_email");
    }
}
