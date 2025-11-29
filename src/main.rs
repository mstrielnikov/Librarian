use clap::Parser;
use std::path::PathBuf;
use anyhow::Result;

// Import modules (assuming you refactored logic into lib or modules)
mod server;
// mod indexer;

#[derive(Parser, Debug)]
pub struct Cli {
    #[arg(short, long)]
    dir: PathBuf,

    #[arg(short = 'b', long, default_value = "./mdkb_data")]
    db: PathBuf,

    #[arg(long)]
    rebuild: bool,

    #[arg(long)]
    json_graph: bool,

    /// Start the web server for visualization
    #[arg(long)]
    server: bool,

    #[arg(long, default_value = "3000")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // 1. Run Indexing (Logic from previous step)
    // indexer::run_indexer(&cli).await?;
    eprintln!("Indexing complete!");

    // 2. Output JSON if requested
    if cli.json_graph {
        // ... json print logic
    }

    // 3. Start Server if requested
    if cli.server {
        server::start_server(cli.port, cli.db).await?;
    }

    Ok(())
}

