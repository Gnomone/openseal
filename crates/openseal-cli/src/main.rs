use clap::{Parser, Subcommand};
use openseal_core::compute_project_identity;
use std::path::{Path, PathBuf};
use std::fs;
use ignore::WalkBuilder;
use anyhow::{Result, Context, anyhow};
use std::process::{Command, Stdio};
use tokio::net::TcpListener;
use openseal_runtime::run_proxy_server;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build and package the project (Seal Source Code)
    Build {
        /// Source directory to seal
        #[arg(short, long, default_value = ".")]
        source: PathBuf,

        /// Output Bundle Directory
        #[arg(short, long, default_value = "./dist")]
        output: PathBuf,

        /// Entry command specific to this project (e.g. "node app.js")
        #[arg(long)]
        exec: Option<String>,
    },
    /// Run the seal-bundled application
    Run {
        /// Path to the sealed project directory (Bundle)
        #[arg(long, default_value = ".")]
        app: PathBuf,

        /// Port to expose (Public Face)
        #[arg(long, default_value = "8080")]
        port: u16,

        /// Setup command override (if not using openseal.json)
        #[arg(long)]
        cmd: Option<String>,

        /// Disable signing key generation (Unsafe/unsigned Mode)
        #[arg(long)]
        no_key: bool,
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Build { source, output, exec } => {
            println!("📦 OpenSeal Packaging System v0.1.0");
            println!("   Source: {:?}", source);
            println!("   Output: {:?}", output);

            // 1. Calculate Identity (Verification)
            println!("   🔍 Scanning and Sealing...");
            let identity = compute_project_identity(source)?;
            println!("   ✅ Root A-Hash: {}", identity.root_hash.to_hex());
            println!("   📄 Files Indexed: {}", identity.file_count);

            // 2. Prepare Output
            if output.exists() {
                println!("   🧹 Cleaning previous build...");
                fs::remove_dir_all(output).context("Failed to clean output directory")?;
            }
            fs::create_dir_all(output).context("Failed to create output directory")?;

            // 3. Copy Files (Packaging) using .gitignore respect
            println!("   🚚 Copying source code...");
            let walker = WalkBuilder::new(source)
                .hidden(false)
                .git_ignore(true)
                .require_git(false)
                .build();

            let mut copied_count = 0;
            for result in walker {
                match result {
                    Ok(entry) => {
                        let path = entry.path();
                        if path.is_dir() { continue; }
                        
                        let relative_path = match path.strip_prefix(source) {
                            Ok(p) => p,
                            Err(_) => continue,
                        };

                        if path.starts_with(output) {
                            continue;
                        }

                        let dest_path = output.join(relative_path);

                        if let Some(parent) = dest_path.parent() {
                            fs::create_dir_all(parent)?;
                        }
                        
                        fs::copy(path, &dest_path)?;
                        copied_count += 1;
                    }
                    Err(err) => eprintln!("Warning: {}", err),
                }
            }
            println!("   📥 Copied {} files to build directory.", copied_count);

            // 4. Write Seal Manifest
            let manifest_path = output.join("openseal.json");
            let mut manifest = serde_json::json!({
                "version": "2.0",
                "identity": identity,
                "sealed": true
            });

            if let Some(cmd) = exec {
                manifest["exec"] = serde_json::Value::String(cmd.clone());
                println!("   ⚙️  Entry Command Registered: {}", cmd);
            }
            
            let file = fs::File::create(manifest_path)?;
            serde_json::to_writer_pretty(file, &manifest)?;

            println!("   ✨ Build Complete! Artifacts in {:?}", output);
        },
        Commands::Run { app, port, cmd, no_key } => {
            println!("🚀 OpenSeal Runner v0.2.0");
            println!("   Bundle: {:?}", app);

            // 1. Validating Bundle
            let manifest_path = app.join("openseal.json");
            if !manifest_path.exists() {
                return Err(anyhow!("Invalid OpenSeal Bundle: openseal.json not found in {:?}", app));
            }
            let file = fs::File::open(&manifest_path)?;
            let manifest: serde_json::Value = serde_json::from_reader(file)?;
            
            // 2. Determine Command
            let run_cmd = if let Some(c) = cmd {
                c.clone()
            } else if let Some(c) = manifest.get("exec").and_then(|v| v.as_str()) {
                c.to_string()
            } else {
                return Err(anyhow!("No execution command found. Please use --cmd or specify during build."));
            };

            // 3. Find Ephemeral Port
            let listener = TcpListener::bind("127.0.0.1:0").await?;
            let internal_port = listener.local_addr()?.port();
            drop(listener); // Close so child can use it? No, child binds to it.
            // Wait, we need to pick a port for the child app to listen ON.
            // We just found a free one.
            
            println!("   🔒 Caller Monopoly Active");
            println!("   Hidden Internal Port: {}", internal_port);
            println!("   Command: {}", run_cmd);

            // 4. Spawn Child Process
            let parts: Vec<&str> = run_cmd.split_whitespace().collect();
            if parts.is_empty() {
                return Err(anyhow!("Empty command string"));
            }
            let program = parts[0];
            let args = &parts[1..];

            println!("   ✨ Spawning Application...");
            let mut child = Command::new(program)
                .args(args)
                .current_dir(app)
                .env("PORT", internal_port.to_string()) // Standard PORT env
                .env("OPENSEAL_PORT", internal_port.to_string()) // Custom one just in case
                .stdout(Stdio::inherit()) // Relay stdout to parent
                .stderr(Stdio::inherit())
                .spawn()
                .context("Failed to spawn application")?;

            // Give it a moment to start (Naive health check)
            // In production, we should loop-check connection to 127.0.0.1:internal_port
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            // 5. Start Runtime Proxy
            // We run this in the current process (tokio main)
            // If runtime proxy dies, we kill child.
            let target_url = format!("http://localhost:{}", internal_port);
            
            // Handle Ctrl+C or Proxy Error to kill child
            let use_key = !no_key;
            let result = run_proxy_server(*port, target_url, app.clone(), use_key).await;
            
            println!("   🛑 Shutting down...");
            let _ = child.kill();
            
            result?;
        }
    }

    Ok(())
}
