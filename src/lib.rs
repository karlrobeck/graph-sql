use async_graphql::{
    Value,
    dynamic::{Field, FieldFuture, Object, Schema, SchemaBuilder, TypeRef},
};
use sea_query::Iden;
use sqlx::SqlitePool;

use crate::{traits::ToGraphqlObject, types::SqliteTable};

pub mod resolvers;
pub mod traits;
pub mod types;

pub async fn introspect(db: &SqlitePool) -> async_graphql::Result<SchemaBuilder> {
    let tables = SqliteTable::introspect(&db).await?;

    let mut query_object = Object::new("Query");
    let mut mutation_object = Object::new("Mutation");

    let mut table_objects = vec![];
    let mut inputs = vec![];

    for table in tables {
        let name = table.table_name();

        let (node, queries, mutations, mutation_inputs) = table.to_object()?;

        // add query
        for query in queries {
            query_object = query_object.field(Field::new(
                name.to_string(),
                TypeRef::named_nn(query.type_name()),
                |_| FieldFuture::new(async move { Ok(Some(Value::Null)) }),
            ));

            table_objects.push(query);
        }

        // add mutations
        for mutation in mutations.into_iter() {
            mutation_object = mutation_object.field(mutation);
        }

        // register types
        table_objects.push(node);
        inputs.extend(mutation_inputs);
    }

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

    Ok(schema.data(db.clone()))
}
