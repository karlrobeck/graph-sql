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

use crate::{config::GraphSQLConfig, loader::ColumnRowLoader, traits::ToGraphqlObject};

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

    pub async fn introspect(&self, db: &SqlitePool) -> async_graphql::Result<Vec<CreateTable>> {
        info!("Starting database introspection");

        let mut sql = Query::select();

        let mut sql = sql
            .from(Alias::new("sqlite_master"))
            .column(Alias::new("sql"))
            .and_where(Expr::col(Alias::new("type")).eq("table"));

        // TODO: implement include and exclude table based on config
        sql = sql.and_where(Expr::col("name").is_not_in(["_sqlx_migrations", "sqlite_sequence"]));

        let tables = sqlx::query_as::<_, (String,)>(&sql.to_string(SqliteQueryBuilder))
            .fetch_all(db)
            .await?;

        debug!("Found {} tables in database", tables.len());

        if tables.is_empty() {
            warn!("No tables found in database");
            return Err(async_graphql::Error::new("No tables found in database"));
        }

        let sqlite_dialect = SQLiteDialect {};

        debug!("Parsing SQL statements with SQLite dialect");

        let tables = tables
            .into_iter()
            .flat_map(|(sql,)| {
                debug!("Parsing SQL: {}", sql);
                Parser::parse_sql(&sqlite_dialect, &sql).unwrap()
            })
            .filter_map(|statement| {
                if let Statement::CreateTable(table) = statement {
                    debug!("Found CREATE TABLE statement for: {}", table.name);
                    Some(table)
                } else {
                    debug!("Skipping non-CREATE TABLE statement");
                    None
                }
            })
            .collect::<Vec<_>>();

        Ok(tables)
    }

    pub fn build_schema(&self, tables: Vec<CreateTable>) -> async_graphql::Result<SchemaBuilder> {
        let mut query_object = Object::new("Query");
        let mut mutation_object = Object::new("Mutation");

        let mut table_objects = vec![];
        let mut inputs = vec![];
        let mut enums = vec![];

        info!("Converting {} tables to GraphQL objects", tables.len());

        for table in tables {
            let name = table.name.to_string();

            debug!("Converting table '{:?}' to GraphQL object", name);

            let graphql = table.to_object()?;

            // add query
            for query in graphql.queries {
                debug!(
                    "Adding query field '{}' for table: {}",
                    query.type_name(),
                    name
                );
                query_object = query_object.field(Field::new(
                    name.to_string(),
                    TypeRef::named_nn(query.type_name()),
                    |_| FieldFuture::new(async move { Ok(Some(Value::Null)) }),
                ));

                table_objects.push(query);
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
        let tables = self.introspect(db).await?;

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
