use async_graphql::dynamic::{Field, InputObject, InputValue, Object, Scalar, TypeRef};

pub trait ToGraphqlScalarExt {
    fn to_scalar(&self) -> async_graphql::Result<Scalar>;
}

pub trait ToGraphqlInputValueExt {
    fn to_input_value(&self, force_nullable: bool) -> async_graphql::Result<InputValue>;
}

pub trait ToGraphqlFieldExt {
    fn to_field(&self, table_name: String) -> async_graphql::Result<Field>;
}

pub trait ToGraphqlTypeRefExt {
    fn to_type_ref(&self) -> async_graphql::Result<TypeRef>;
}

pub trait ToGraphqlMutations {
    fn to_insert_mutation(&self) -> async_graphql::Result<(InputObject, Field)>;
    fn to_update_mutation(&self) -> async_graphql::Result<(InputObject, Field)>;
    fn to_delete_mutation(&self) -> async_graphql::Result<(InputObject, Field)>;
}

pub trait ToGraphqlQueries {
    fn to_list_query(&self) -> async_graphql::Result<(InputObject, Field)>;
    fn to_view_query(&self) -> async_graphql::Result<(InputObject, Field)>;
}

pub trait ToGraphqlNode {
    fn to_node(&self) -> async_graphql::Result<Object>;
}
