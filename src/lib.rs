use async_graphql::{
    Value,
    dataloader::DataLoader,
    dynamic::{Field, FieldFuture, Object, Schema, SchemaBuilder, TypeRef},
    http::GraphiQLSource,
};
use async_graphql_axum::GraphQL;
use axum::{Router, response::Html};
use sea_query::{Alias, Expr, Query, SqliteQueryBuilder};
use sqlparser::{
    ast::{CreateTable, Statement},
    dialect::SQLiteDialect,
    parser::Parser,
};
use sqlx::SqlitePool;
use tokio::net::TcpListener;
use tracing::{debug, info, warn};

use crate::{
    config::GraphSQLConfig,
    loader::ColumnRowLoader,
    parser::{Introspector, TableDef},
    traits::{GraphQLObjectOutput, ToGraphqlObject},
    utils::StringFilter,
};

pub mod config;
pub mod loader;
pub mod parser;
pub mod resolvers;
pub mod traits;
pub mod utils;

pub struct GraphSQL {
    config: GraphSQLConfig,
}

impl GraphSQL {
    pub fn new(config: GraphSQLConfig) -> Self {
        Self { config }
    }

    pub async fn introspect(&self, db: &SqlitePool) -> async_graphql::Result<Vec<TableDef>> {
        info!("Starting database introspection");

        Ok(TableDef::introspect(db).await?)
    }

    pub fn build_schema(&self, tables: Vec<TableDef>) -> async_graphql::Result<SchemaBuilder> {
        let mut query_object = Object::new("Query");
        let mut mutation_object = Object::new("Mutation");

        let mut table_objects = vec![];
        let mut inputs = vec![];
        let mut enums = vec![];

        info!("Converting {} tables to GraphQL objects", tables.len());

        for table in tables {
            let name = table.name.to_string();

            debug!("Converting table '{:?}' to GraphQL object", name);

            let graphql = GraphQLObjectOutput::from(table);

            // add query
            for query_field in graphql.queries {
                query_object = query_object.field(query_field);
            }

            // add mutations
            for mutation in graphql.mutations.into_iter() {
                debug!("Adding mutation field for table: {}", name);
                mutation_object = mutation_object.field(mutation);
            }

            // register types
            table_objects.push(graphql.table);
            inputs.extend(graphql.inputs);
            enums.extend(graphql.enums);
        }

        // register filter operators
        inputs.push(StringFilter::to_object());

        info!(
            "Building GraphQL schema with {} objects and {} inputs",
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

        for enum_item in enums {
            schema = schema.register(enum_item);
        }

        info!("Successfully built GraphQL schema");

        Ok(self.config.graphql.apply(schema))
    }

    pub async fn build(&self, db: &SqlitePool) -> async_graphql::Result<(Router, TcpListener)> {
        let mut tables = self.introspect(db).await?;

        // remove private tables
        tables = tables
            .into_iter()
            .filter(|table| table.name == "_sqlx_migrations")
            .collect::<Vec<_>>();

        let schema = self.build_schema(tables)?;

        let schema = schema
            .data(DataLoader::new(
                ColumnRowLoader { pool: db.clone() },
                tokio::spawn,
            ))
            .data(db.clone())
            .finish()?;

        let mut router = Router::new();

        if self.config.graphql.enable_playground.unwrap_or(true) {
            router = router.route(
                "/",
                axum::routing::get(|| async move {
                    Html(GraphiQLSource::build().endpoint("/").finish())
                })
                .post_service(GraphQL::new(schema)),
            );
        } else {
            router = router.route("/", axum::routing::post_service(GraphQL::new(schema)));
        }

        let listener = TcpListener::bind(format!(
            "{}:{}",
            self.config.server.host, self.config.server.port
        ))
        .await?;

        Ok((router, listener))
    }
}
