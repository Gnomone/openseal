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

        /// Explicit dependency folder to ghost/link (e.g. "node_modules", "venv")
        /// If not provided, common patterns will be auto-detected.
        #[arg(long)]
        deps: Option<String>,
    },
    /// Run the seal-bundled application
    Run {
        /// Path to the sealed project directory (Bundle)
        #[arg(long, default_value = ".")]
        app: PathBuf,

        /// Port to expose to the public (OpenSeal Proxy)
        #[arg(short, long, default_value = "7325", alias = "port")]
        public_port: u16,

        /// Setup command override (if not using openseal.json)
        #[arg(long)]
        cmd: Option<String>,

        /// Run in daemon mode (background)
        #[arg(short, long)]
        daemon: bool,

        /// Log file for daemon mode
        #[arg(long, default_value = "openseal.log")]
        log_file: String,
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
    // Internal logs hidden as requested
    
    while start.elapsed().as_secs() < timeout_secs {
        if TcpStream::connect(&addr).is_ok() {
            // println!("   Internal app is READY (detected in {:?})", start.elapsed());
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
        Commands::Build { source, output, exec, deps } => {
            println!("OpenSeal Packaging System v0.2.4");
            println!("   Source: {:?}", source);
            println!("   Output: {:?}", output);

            // 0. Safety Guardrail: Project Detection
            if !is_project_root(source) {
                println!("   ⚠️  WARNING: No standard project files (package.json, Cargo.toml, .git, etc.) detected in {:?}.", source);
                print!("      Do you want to proceed with Sealing this directory anyway? (y/N): ");
                use std::io::{self, Write};
                io::stdout().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                if !input.trim().to_lowercase().starts_with('y') {
                    println!("   ❌ Build aborted by user.");
                    return Ok(());
                }
            }

            // 1. Ensure Configuration Files exist (Lazy Init)
            ensure_config_files(source)?;

            // 1. Calculate Identity (Verification)
            println!("   Scanning and Sealing...");
            let identity = compute_project_identity(source)?;
            println!("   ✅ Root A-Hash: {}", identity.root_hash.to_hex());
            println!("   Files Indexed: {}", identity.file_count);

            // 2. Prepare Output
            if output.exists() {
                println!("   Cleaning previous build...");
                // CRITICAL: Use read_link to detect and safely remove symlinks without following them
                for entry in fs::read_dir(output)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_symlink() {
                        fs::remove_file(&path)?; // Remove symlink itself, not target
                    } else if path.is_dir() {
                        fs::remove_dir_all(&path)?;
                    } else {
                        fs::remove_file(&path)?;
                    }
                }
            } else {
                fs::create_dir_all(output).context("Failed to create output directory")?;
            }

            // 3. Copy Files (Packaging) using .gitignore respect
            println!("   Copying source code...");
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
            println!("   Copied {} files to build directory.", copied_count);

            // 4. Create & Write Seal Manifest
            let mut manifest = serde_json::json!({
                "version": "1.0.0",
                "identity": identity,
                "sealed": true,
                "timestamp": chrono::Utc::now().to_rfc3339()
            });

            if let Some(cmd) = exec {
                manifest["exec"] = serde_json::Value::String(cmd.clone());
                println!("   Entry Command Registered: {}", cmd);
            }

            // 5. Dependency Ghosting (Automated Linking)
            println!("   🔗 Ghosting Dependencies (Runtime-only Linking)...");
            
            // Collect candidates
            let mut candidates = vec![];
            if let Some(explicit_deps) = deps {
                candidates.push(explicit_deps.clone());
            } else {
                // Auto-detect common patterns
                candidates.extend(vec![
                    "node_modules".to_string(),
                    "venv".to_string(),
                    ".venv".to_string(),
                    "env".to_string(),
                ]);
            }


            let mut linked_any = false;
            let mut linked_deps = Vec::new();
            for dep_name in candidates {
                let dep_src = source.join(&dep_name);
                if dep_src.exists() && dep_src.is_dir() {
                    let dep_dest = output.join(&dep_name);
                    
                    // Create symbolic link (platform specific)
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::symlink;
                        // Remove existing symlink if present (for rebuild compatibility)
                        if dep_dest.exists() || dep_dest.is_symlink() {
                            let _ = fs::remove_file(&dep_dest);
                        }
                        if let Err(e) = symlink(&dep_src, &dep_dest) {
                            eprintln!("   ⚠️  Failed to link {}: {}", dep_name, e);
                        } else {
                            println!("   ✅ Automatically ghosted: {}", dep_name);
                            linked_any = true;
                            linked_deps.push(dep_name.clone());
                        }
                    }

                    #[cfg(windows)]
                    {
                        use std::os::windows::fs::symlink_dir;
                        // Remove existing symlink if present (for rebuild compatibility)
                        if dep_dest.exists() || dep_dest.is_symlink() {
                            let _ = fs::remove_file(&dep_dest);
                        }
                        if let Err(e) = symlink_dir(&dep_src, &dep_dest) {
                            eprintln!("   ⚠️  Failed to link {}: {}", dep_name, e);
                        } else {
                            println!("   ✅ Automatically ghosted: {}", dep_name);
                            linked_any = true;
                            linked_deps.push(dep_name.clone());
                        }
                    }
                }
            }

            // Record all linked dependencies in manifest
            if !linked_deps.is_empty() {
                manifest["deps"] = serde_json::Value::Array(
                    linked_deps.into_iter().map(serde_json::Value::String).collect()
                );
            }

            if !linked_any && deps.is_some() {
                println!("   ⚠️  Warning: Explicitly requested deps folder {:?} not found.", deps.as_ref().unwrap());
            }

            // 6. Save Manifests
            // [AUTO-GEN] Write to Source (The Proclaimed Identity)
            let source_manifest_path = source.join("openseal.json");
            let source_file = fs::File::create(&source_manifest_path)?;
            serde_json::to_writer_pretty(source_file, &manifest)?;
            println!("   Identity Manifest saved to {:?}", source_manifest_path);
            
            // Write to Output (The Bundled Identity)
            let output_manifest_path = output.join("openseal.json");
            let output_file = fs::File::create(output_manifest_path)?;
            serde_json::to_writer_pretty(output_file, &manifest)?;

            println!("   ✨ Build Complete! Artifacts in {:?}", output);
        },
        Commands::Run { app, public_port, cmd, daemon, log_file } => {
            // Daemon mode: re-execute self in background
            if *daemon {
                println!("🚀 Starting OpenSeal in daemon mode...");
                println!("   Log file: {}", log_file);
                
                let current_exe = std::env::current_exe()?;
                let mut args = vec![
                    "run".to_string(),
                    "--app".to_string(),
                    app.to_str().unwrap().to_string(),
                    "--port".to_string(),
                    public_port.to_string(),
                ];
                
                if let Some(c) = cmd {
                    args.push("--cmd".to_string());
                    args.push(c.clone());
                }

                let log_file_path = PathBuf::from(log_file);
                let log_handle = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&log_file_path)?;

                Command::new(current_exe)
                    .args(&args)
                    .stdout(log_handle.try_clone()?)
                    .stderr(log_handle)
                    .spawn()
                    .context("Failed to spawn daemon process")?;

                println!("   ✅ OpenSeal daemon started");
                println!("   📝 View logs: tail -f {}", log_file_path.display());
                println!("   🛑 Stop: pkill -f 'openseal run'");
                return Ok(());
            }

            println!("🚀 OpenSeal Runner v0.2.4");
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
            
            // println!("   🔒 Caller Monopoly Active");
            println!("   🌐 Proxy Port (Public): {}", public_port);
            // println!("   🔌 Internal Port (Hidden): {}", internal_port);
            // println!("   🛠️  Service Command: {}", run_cmd);

            // 4. Spawn Child Process (Using shell for command string support & environment stability)
            let mut cmd_builder = if cfg!(target_os = "windows") {
                let mut c = Command::new("cmd");
                c.arg("/C").arg(&run_cmd);
                c
            } else {
                let mut c = Command::new("sh");
                c.arg("-c").arg(&run_cmd);
                c
            };

            // println!("   ✨ Spawning Application (Sanitized Environment)...");
            let mut child = cmd_builder
                .current_dir(app)
                .env_clear() // 🛡️ Security: Clear all host environment variables
                .env("PORT", internal_port.to_string())
                .env("OPENSEAL_PORT", internal_port.to_string())
                .env("PATH", std::env::var("PATH").unwrap_or_default()) 
                .env("HOME", std::env::var("HOME").unwrap_or_default())
                .env("USER", std::env::var("USER").unwrap_or_default())
                .env("TERM", std::env::var("TERM").unwrap_or_default())
                .env("PWD", app.to_str().unwrap_or_default())
                .env("TMPDIR", std::env::var("TMPDIR").unwrap_or_default()) // Needed for some build tools
                .env("NODE_ENV", "production") 
                .env("PYTHONDONTWRITEBYTECODE", "1")
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()
                .context("Failed to spawn application. Make sure the command exists and dependencies are installed.")?;

            // Dynamic Port Polling (Security & Reliability)
            wait_for_port(internal_port, 30).await?;

            // 5. Start Runtime Proxy with Graceful Shutdown
            let target_url = format!("http://127.0.0.1:{}", internal_port);
            
            // Use tokio::select to handle both proxy and Ctrl+C
            tokio::select! {
                res = run_proxy_server(*public_port, target_url, app.clone()) => {
                    if let Err(e) = res {
                         eprintln!("   ❌ Runtime Error: {}", e);
                    }
                },
                _ = tokio::signal::ctrl_c() => {
                    println!("\n   🛑 Received Ctrl+C, shutting down...");
                }
            }

            let _ = child.kill(); 
            let _ = child.wait(); // Prevent zombie processes
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
        fs::write(&ignore_path, "# OpenSeal Ignore Rules\n# Add files/folders to exclude from the File Integrity Check (A-hash)\n# Syntax is same as .gitignore\n\nnode_modules/\nvenv/\n__pycache__/\n.env\n*.md\n\n# Build Outcomes (Source Integrity Only)\ndist/\nbuild/\n\n# OpenSeal Artifacts (Self-exclusion)\nopenseal.json\n.opensealignore\n.openseal_mutable\n")?;
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

fn is_project_root(path: &Path) -> bool {
    let indicators = [
        "package.json",    // Node.js
        "Cargo.toml",      // Rust
        "requirements.txt", // Python
        "pyproject.toml",  // Python
        "go.mod",         // Go
        "composer.json",   // PHP
        "Gemfile",        // Ruby
        ".git",           // Version Control
        ".opensealignore"  // Existing OpenSeal project
    ];

    for indicator in &indicators {
        if path.join(indicator).exists() {
            return true;
        }
    }
    false
}
