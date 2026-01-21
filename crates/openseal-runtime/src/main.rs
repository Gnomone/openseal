use clap::Parser;
use std::path::PathBuf;
use openseal_runtime::run_proxy_server;

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on (External entry point)
    #[arg(short, long, default_value = "7325")]
    port: u16,

    /// Target URL of the internal application to wrap (e.g., http://127.0.0.1:8000)
    #[arg(long, default_value = "http://127.0.0.1:8000")]
    target: String,

    /// Path to the project root to seal (Identity source)
    #[arg(long, default_value = ".")]
    project_root: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    
    // In stand-alone mode, we assume no dependency hint for now 
    // or we could add it to Args.
    let project_identity = openseal_runtime::prepare_runtime(&args.project_root, None).await?;
    
    run_proxy_server(args.port, args.target, args.project_root, project_identity).await
}
