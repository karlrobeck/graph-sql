use crate::cli::Cli;
use clap::Parser;

mod cli;

#[tokio::main]
async fn main() -> async_graphql::Result<()> {
    // Initialize tracing with more detailed configuration
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,graph_sql=debug")),
        )
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    let cli = Cli::parse();

    cli.start().await
}
