use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "plit-tui", about = "Terminal UI for the plit ecosystem")]
struct Cli {
    /// Override the Pipelit URL (defaults to value from `plit auth login`)
    #[arg(long)]
    url: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    plit_tui::run(cli.url).await
}
