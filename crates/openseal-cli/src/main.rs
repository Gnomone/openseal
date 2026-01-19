use clap::{Parser, Subcommand};
use openseal_core::compute_project_identity;
use std::path::{Path, PathBuf};
use std::fs;
use ignore::WalkBuilder;
use anyhow::{Result, Context, anyhow};
use std::process::{Command, Stdio};
use tokio::net::TcpListener;
use openseal_runtime::run_proxy_server;
use std::time::{Duration, Instant};
use std::net::TcpStream;

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
        #[arg(short, long, default_value = "7325")]
        port: u16,

        /// Setup command override (if not using openseal.json)
        #[arg(long)]
        cmd: Option<String>,
    },
    /// Verify an OpenSeal response to check integrity (Dev Mode)
    Verify {
        /// Path to the API response JSON file
        #[arg(short, long)]
        response: PathBuf,

        /// Wax (Challenge) string used for the request
        #[arg(short, long)]
        wax: String,

        /// (Optional) Expected Root Hash (A-hash seed) to verify identity
        #[arg(long)]
        root_hash: Option<String>,
    }
}

/// Waits for the given port to become available (app is ready)
async fn wait_for_port(port: u16, timeout_secs: u64) -> Result<()> {
    let start = Instant::now();
    let addr = format!("127.0.0.1:{}", port);
    println!("   ⏳ Waiting for internal app to bind to port {}...", port);
    
    while start.elapsed().as_secs() < timeout_secs {
        if TcpStream::connect(&addr).is_ok() {
            println!("   ✅ Internal app is READY (detected in {:?})", start.elapsed());
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    Err(anyhow!("Timeout: Internal app failed to bind to port {} within {}s", port, timeout_secs))
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Build { source, output, exec } => {
            println!("📦 OpenSeal Packaging System v0.1.0");
            println!("   Source: {:?}", source);
            println!("   Output: {:?}", output);

            // 0. Ensure Configuration Files exist (Lazy Init)
            ensure_config_files(source)?;

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
                .add_custom_ignore_filename(".opensealignore")
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

            // [FIX] Force copy config files to ensure Runtime respects ignore rules
            // They might be ignored by walker if listed in .opensealignore (Self-exclusion)
            for file in &[".opensealignore", ".openseal_mutable"] {
                let src_path = source.join(file);
                if src_path.exists() {
                     let dest_path = output.join(file);
                     fs::copy(&src_path, &dest_path)?;
                }
            }
            println!("   📥 Copied {} files to build directory.", copied_count);

            // 4. Create & Write Seal Manifest
            let mut manifest = serde_json::json!({
                "version": "1.0.0",
                "identity": identity,
                "sealed": true,
                "timestamp": chrono::Utc::now().to_rfc3339()
            });

            if let Some(cmd) = exec {
                manifest["exec"] = serde_json::Value::String(cmd.clone());
                println!("   ⚙️  Entry Command Registered: {}", cmd);
            }

            // [AUTO-GEN] Write to Source (The Proclaimed Identity)
            let source_manifest_path = source.join("openseal.json");
            let source_file = fs::File::create(&source_manifest_path)?;
            serde_json::to_writer_pretty(source_file, &manifest)?;
            println!("   💾 Identity Manifest saved to {:?}", source_manifest_path);
            
            // Write to Output (The Bundled Identity)
            let output_manifest_path = output.join("openseal.json");
            let output_file = fs::File::create(output_manifest_path)?;
            serde_json::to_writer_pretty(output_file, &manifest)?;

            println!("   ✨ Build Complete! Artifacts in {:?}", output);
        },
        Commands::Run { app, port, cmd } => {
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

            println!("   ✨ Spawning Application (Sanitized Environment)...");
            let mut child = Command::new(program)
                .args(args)
                .current_dir(app)
                .env_clear() // 🛡️ Security: Clear all host environment variables
                .env("PORT", internal_port.to_string())
                .env("OPENSEAL_PORT", internal_port.to_string())
                .env("PATH", std::env::var("PATH").unwrap_or_default()) // Essential for finding executables
                .env("NODE_ENV", std::env::var("NODE_ENV").unwrap_or_else(|_| "production".to_string()))
                .env("PYTHONDONTWRITEBYTECODE", "1") // 🛡️ Security: Prevent .pyc generation interfering with A-hash
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()
                .context("Failed to spawn application")?;

            // Dynamic Port Polling (Security & Reliability)
            wait_for_port(internal_port, 10).await?;

            // 5. Start Runtime Proxy with Graceful Shutdown
            let target_url = format!("http://127.0.0.1:{}", internal_port);
            
            // Use tokio::select to handle both proxy and Ctrl+C
            tokio::select! {
                res = run_proxy_server(*port, target_url, app.clone()) => {
                    if let Err(e) = res {
                         eprintln!("   ❌ Runtime Error: {}", e);
                    }
                },
                _ = tokio::signal::ctrl_c() => {
                    println!("\n   🛑 Received Ctrl+C, shutting down...");
                }
            }

            println!("   🧹 Cleaning up child process...");
            let _ = child.kill(); // Synchronous kill for std::process::Child
            // Oh, we used std::process::Command. Wait, main is async.
            // If we use tokio::process::Command, we can await kill.
            // Let's check imports.
            
            // Currently using std::process::Command.
            // Changing to tokio::process::Command is better for async.
            // BUT for now, let's just use the current child.
            // std::process::Child doesn't have kill().await. It has kill().
             let _ = child.kill();
             let _ = child.kill();
        },
        Commands::Verify { response, wax, root_hash } => {
            println!("🔍 OpenSeal Verifier (Dev Mode)");
            println!("   Response File: {:?}", response);
            println!("   Wax Challenge: {}", wax);
            if let Some(h) = root_hash {
                println!("   Expected Root: {}", h);
            }

            let content = fs::read_to_string(response).context("Failed to read response file")?;
            let json: serde_json::Value = serde_json::from_str(&content).context("Failed to parse JSON")?;

            // Delegate to core verification logic
            let report = openseal_core::verify_seal(&json, wax, root_hash.as_deref())?;

            println!("\n🔍 Verification Report:");
            println!("   Signature Valid: {}", if report.signature_verified { "✅" } else { "❌" });
            println!("   Binding Valid:   {}", if report.binding_verified { "✅" } else { "❌" });
            if root_hash.is_some() {
                println!("   Identity Valid:  {}", if report.identity_verified { "✅" } else { "❌" });
            }
            println!("   ----------------------------------------");
            println!("   Result: {}", report.message);

            if !report.valid {
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

fn ensure_config_files(source: &Path) -> Result<()> {
    let ignore_path = source.join(".opensealignore");
    if !ignore_path.exists() {
        println!("   📝 Creating default .opensealignore...");
        fs::write(&ignore_path, "# OpenSeal Ignore Rules\n# Add files/folders to exclude from the File Integrity Check (A-hash)\n# Syntax is same as .gitignore\n\nnode_modules/\nvenv/\n__pycache__/\n.env\n*.md\n\n# OpenSeal Artifacts (Self-exclusion)\nopenseal.json\n.opensealignore\n.openseal_mutable\n")?;
    } else {
        // [AUTO-FIX] Ensure openseal.json is ignored to prevent spiral hashing
        let content = fs::read_to_string(&ignore_path)?;
        if !content.contains("openseal.json") {
            println!("   🔧 Auto-patching .opensealignore: Adding openseal.json exclusion");
            let mut file = fs::OpenOptions::new().append(true).open(&ignore_path)?;
            use std::io::Write;
            writeln!(file, "\n# Auto-added by OpenSeal CLI\nopenseal.json")?;
        }
    }

    let mutable_path = source.join(".openseal_mutable");
    if !mutable_path.exists() {
        println!("   📝 Creating default .openseal_mutable...");
        fs::write(&mutable_path, "# OpenSeal Mutable Files\n# Add files whose presence is sealed but content can change\n# (e.g., local databases, logs)\n\n# *.db\n# logs/\n")?;
    }
    Ok(())
}
