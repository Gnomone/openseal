use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderValue, StatusCode},
    response::IntoResponse,
    routing::any,
    Router,
};
use openseal_core::{compute_a_hash, compute_project_identity, ProjectIdentity};
use openseal_secret::compute_b_hash;
use rand::{rngs::OsRng, RngCore};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use ed25519_dalek::{SigningKey, Signer};
use anyhow::{anyhow, Context};
use std::io::{self, Write};
use std::process::Command;

#[derive(Clone)]
struct AppState {
    target_url: String,
    project_identity: ProjectIdentity,
    signing_key: SigningKey,
}

pub async fn prepare_runtime(
    project_root: &PathBuf,
    dependency_hint: Option<String>,
) -> anyhow::Result<ProjectIdentity> {
    println!("🔐 OpenSeal Runtime v{} Initializing...", env!("CARGO_PKG_VERSION"));
    
    // 1. Static Commitment: Compute A-hash at startup
    let live_identity = compute_project_identity(&project_root)?;
    
    // 2. Load Expected Hash from openseal.json
    let manifest_path = project_root.join("openseal.json");
    if manifest_path.exists() {
        let manifest_content = std::fs::read_to_string(&manifest_path)?;
        let manifest: serde_json::Value = serde_json::from_str(&manifest_content)?;
        
        if let Some(expected_hash_array) = manifest["identity"]["root_hash"].as_array() {
            let expected_hash_bytes: Vec<u8> = expected_hash_array
                .iter()
                .filter_map(|v| v.as_u64().map(|n| n as u8))
                .collect();
            
            if live_identity.root_hash.as_bytes() != expected_hash_bytes.as_slice() {
                eprintln!("\n🚨 ═══════════════════════════════════════════════════════════");
                eprintln!("   CRITICAL: INTEGRITY VIOLATION DETECTED");
                eprintln!("   ═══════════════════════════════════════════════════════════");
                eprintln!("   The sealed bundle has been modified!");
                eprintln!("   ");
                eprintln!("   Expected Hash: {}", hex::encode(&expected_hash_bytes));
                eprintln!("   Actual Hash:   {}", live_identity.root_hash.to_hex());
                eprintln!("   ");
                eprintln!("   This runtime will NOT start for security reasons.");
                eprintln!("   Please rebuild with 'openseal build' to restore integrity.");
                eprintln!("   ═══════════════════════════════════════════════════════════\n");
                
                return Err(anyhow!("Integrity violation detected - Runtime aborted"));
            }
            println!("   ✅ Integrity Verified!");
        }
    }

    // 3. Dependency Management
    handle_dependencies(project_root, dependency_hint).await?;
    
    Ok(live_identity)
}

pub async fn run_proxy_server(
    port: u16, 
    target_url: String, 
    _project_root: PathBuf,
    project_identity: ProjectIdentity,
) -> anyhow::Result<()> {

    // Generate a strictly ephemeral signing key for this runtime session (Mandatory in v2.0)
    let mut csprng = OsRng;
    let mut key_bytes = [0u8; 32];
    csprng.fill_bytes(&mut key_bytes);
    let key = SigningKey::from_bytes(&key_bytes);
    let verifying_key = key.verifying_key();
    println!("   🔑 Public Key (Ephemeral): {}", hex::encode(verifying_key.to_bytes()));

    let state = Arc::new(AppState {
        target_url,
        project_identity,
        signing_key: key,
    });

    let app = Router::new()
        .route("/.openseal/identity", any(identity_handler))
        .route("/*path", any(handler))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    println!("🚀 OpenSeal Running on {}", addr);
    println!("   Standard Identity Endpoint: http://{}/.openseal/identity", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_dependencies(
    project_root: &std::path::Path,
    dependency_hint: Option<String>,
) -> anyhow::Result<()> {
    // 1. Explicit dependency specification
    if let Some(dep_dir) = dependency_hint {
        let dep_path = std::path::Path::new(&dep_dir);
        
        // We check if the dependency source exists anywhere (could be absolute or relative to where openseal run was called)
        if dep_path.is_dir() {
            let src_abs = std::fs::canonicalize(dep_path).context(format!("Failed to resolve absolute path for {:?}", dep_path))?;
            
            // Destination is "node_modules" inside the runtime project_root (e.g. dist_opensealed/node_modules)
            let dep_dest = project_root.join("node_modules");
            
            // If it's already there and is a symlink, we check if it points to the right place.
            // If it's not there, we create it.
            if !dep_dest.exists() && !dep_dest.is_symlink() {
                println!("   🔗 Linking dependencies: {} -> {:?}", dep_dir, src_abs);
                #[cfg(unix)]
                {
                    use std::os::unix::fs::symlink;
                    symlink(&src_abs, &dep_dest).context("Failed to create dependency symlink")?;
                }
                #[cfg(windows)]
                {
                    use std::os::windows::fs::symlink_dir;
                    symlink_dir(&src_abs, &dep_dest).context("Failed to create dependency symlink")?;
                }
            } else {
                println!("   ✅ Using existing dependencies: {}/", dep_dir);
            }
            return Ok(());
        } else {
            eprintln!("   ⚠️  Specified dependency '{}' not found or is not a directory. Retrying with auto-detection...", dep_dir);
        }
    }

    // 2. Auto-detection
    let dep_info = detect_dependencies(project_root)?;

    match dep_info {
        Some(DepInfo::NodeJs { exists, .. }) if exists => {
            println!("   ✅ Node.js dependencies found");
            Ok(())
        }
        Some(DepInfo::Python { exists, .. }) if exists => {
            println!("   ✅ Python dependencies found");
            Ok(())
        }
        Some(dep_info) => {
            // Case 3: Project detected but dependencies missing - Ask user or auto-install in daemon mode
            prompt_and_install(project_root, dep_info).await
        }
        None => {
            // println!("   ℹ️  No standard dependency patterns detected");
            Ok(())
        }
    }
}

enum DepInfo {
    NodeJs { exists: bool, _package_json: std::path::PathBuf },
    Python { exists: bool, _requirements: std::path::PathBuf },
}

fn detect_dependencies(project_root: &std::path::Path) -> anyhow::Result<Option<DepInfo>> {
    // Node.js
    let package_json = project_root.join("package.json");
    if package_json.exists() {
        let node_modules = project_root.join("node_modules");
        let exists = node_modules.exists() && !node_modules.is_symlink();
        return Ok(Some(DepInfo::NodeJs { exists, _package_json: package_json }));
    }

    // Python
    let requirements = project_root.join("requirements.txt");
    if requirements.exists() {
        // venv or .venv
        let venv = project_root.join("venv");
        let dot_venv = project_root.join(".venv");
        let exists = (venv.exists() && !venv.is_symlink()) || (dot_venv.exists() && !dot_venv.is_symlink());
        return Ok(Some(DepInfo::Python { exists, _requirements: requirements }));
    }

    Ok(None)
}

async fn prompt_and_install(
    project_root: &std::path::Path,
    dep_info: DepInfo,
) -> anyhow::Result<()> {
    // Check if we are in interactive mode or daemon
    let is_atty = atty::is(atty::Stream::Stdin);
    let is_non_interactive = std::env::var("OPENSEAL_NON_INTERACTIVE").unwrap_or_default() == "1";
    let is_daemon = std::env::var("OPENSEAL_DAEMON").unwrap_or_default() == "1";

    match dep_info {
        DepInfo::NodeJs { .. } => {
            if !is_atty || is_non_interactive || is_daemon {
                 println!("   📦 Automatically installing Node.js dependencies (Non-interactive mode)...");
                 return install_npm_dependencies(project_root);
            }

            println!("\n📦 Node.js project detected, but 'node_modules/' is missing or is just a symlink.");
            print!("   Would you like to install dependencies now? (Y/n): ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            if input.trim().is_empty() || input.trim().to_lowercase().starts_with('y') {
                install_npm_dependencies(project_root)
            } else {
                eprintln!("   ⚠️  Skipping installation. The application may fail to start.");
                Ok(())
            }
        }
        DepInfo::Python { .. } => {
            if !is_atty || is_non_interactive || is_daemon {
                 println!("   📦 Automatically installing Python dependencies (Non-interactive mode)...");
                 return install_pip_dependencies(project_root);
            }

            println!("\n📦 Python project detected, but 'venv/' is missing or is just a symlink.");
            print!("   Would you like to install dependencies now? (Y/n): ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            if input.trim().is_empty() || input.trim().to_lowercase().starts_with('y') {
                install_pip_dependencies(project_root)
            } else {
                eprintln!("   ⚠️  Skipping installation. The application may fail to start.");
                Ok(())
            }
        }
    }
}

fn install_npm_dependencies(project_root: &std::path::Path) -> anyhow::Result<()> {
    // Check if npm exists
    if Command::new("npm").arg("--version").stdout(std::process::Stdio::null()).status().is_err() {
        return Err(anyhow!("'npm' command not found. Please ensure Node.js is installed."));
    }

    println!("   ⚙️  Running 'npm install' in {:?}...", project_root);
    let status = Command::new("npm")
        .arg("install")
        .arg("--no-fund")
        .arg("--no-audit")
        .current_dir(project_root)
        .status()?;

    if status.success() {
        println!("   ✅ Node.js dependencies installed successfully.");
        Ok(())
    } else {
        Err(anyhow!("'npm install' failed with status: {}", status))
    }
}

fn install_pip_dependencies(project_root: &std::path::Path) -> anyhow::Result<()> {
    // Check if pip/python exists
    if Command::new("pip").arg("--version").stdout(std::process::Stdio::null()).status().is_err() {
        return Err(anyhow!("'pip' command not found. Please ensure Python is installed."));
    }

    println!("   ⚙️  Running 'pip install -r requirements.txt' in {:?}...", project_root);
    let status = Command::new("pip")
        .arg("install")
        .arg("-r")
        .arg("requirements.txt")
        .current_dir(project_root)
        .status()?;

    if status.success() {
        println!("   ✅ Python dependencies installed successfully.");
        Ok(())
    } else {
        Err(anyhow!("'pip install' failed with status: {}", status))
    }
}

/// Handler for /.openseal/identity endpoint
/// Returns the runtime's identity (A-hash) without requiring the internal app to be running
async fn identity_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // Simple identity response without requiring Wax challenge
    // This is a public, read-only endpoint for discovery
    let identity_response = serde_json::json!({
        "service": "OpenSeal Runtime Identity",
        "version": "0.2.0",
        "identity": {
            "a_hash": state.project_identity.root_hash.to_hex().to_string(),
            "file_count": state.project_identity.file_count,
        },
        "status": "sealed"
    });

    (StatusCode::OK, axum::Json(identity_response)).into_response()
}

async fn handler(State(state): State<Arc<AppState>>, req: Request<Body>) -> impl IntoResponse {
    let client = reqwest::Client::new();
    
    // 2. Dynamic Trajectory: Extract Wax (Challenge/Context) from Header
    // Caller-driven Verification: The verification logic relies on the caller providing the challenge
    let wax_header = req.headers().get("X-OpenSeal-Wax");
    
    let wax_hex = match wax_header {
        Some(val) => val.to_str().unwrap_or_default().to_string(), // In production, handle unwrap better
        None => {
             return (StatusCode::BAD_REQUEST, "Missing Required Header: X-OpenSeal-Wax").into_response();
        }
    };

    // Prepare A-hash
    // Prepare Blinded A-hash
    let a_hash = compute_a_hash(&state.project_identity.root_hash, &wax_hex);
    let a_hash_hex = a_hash.to_hex().to_string();

    // 3. Execution Interception (Call Boundary)
    // Construct the internal request
    let path = req.uri().path().to_string();
    let query = req.uri().query().map(|q| format!("?{}", q)).unwrap_or_default();
    let target_uri = format!("{}{}{}", state.target_url, path, query);
    
    let method = req.method().clone();
    let mut headers = req.headers().clone();

    // Inject Wax into headers for the internal app (Transparency)
    // The internal app *can* use this, but OpenSeal enforces the B-hash regardless.
    headers.insert("X-OpenSeal-Wax", HeaderValue::from_str(&wax_hex).unwrap());

    // Extract body to forward
    let body_bytes = axum::body::to_bytes(req.into_body(), usize::MAX).await.unwrap_or_default();

    // Call the Internal Logic (The Case)
    let response_result = client
        .request(method, &target_uri)
        .headers(headers) // Forward headers with Wax
        .body(body_bytes)
        .send()
        .await;

    match response_result {
        Ok(resp) => {
            // 4. Result Capture (Egress Interception)
            let _status = resp.status();
            let resp_bytes = resp.bytes().await.unwrap_or_default();

            // 5b. Standardization (Canonicalization Attempt)
            // To ensure verify tool can reproduce the hash, we must use the SAME serialization.
            // We parse the raw body to Value, then re-serialize to string.
            // This aligns with verify_seal() logic in core.
            let original_body_str = String::from_utf8_lossy(&resp_bytes);
            let result_json: serde_json::Value = serde_json::from_str(&original_body_str)
                .unwrap_or(serde_json::Value::String(original_body_str.to_string()));
            
            // Re-serialize for hashing (Standardized)
            let standardized_result_str = serde_json::to_string(&result_json).unwrap_or_default();
            let standardized_bytes = standardized_result_str.as_bytes();

            // 5. Atomic Sealing (B-hash generation)
            // B = b_G(Result, A, Wax)
            let b_hash = compute_b_hash(&a_hash, &wax_hex, standardized_bytes);
            let b_hash_hex = b_hash.to_hex().to_string();

            // 6. Optional Sign the Seal
            // Signature = Sign(Wax || A || B || SHA256(Result))
            let result_hash = blake3::hash(standardized_bytes).to_hex().to_string();
            
            let sign_payload = format!("{}{}{}{}", wax_hex, a_hash_hex, b_hash_hex, result_hash);
            let sig = state.signing_key.sign(sign_payload.as_bytes());
            let pub_key_hex = hex::encode(state.signing_key.verifying_key().to_bytes());
            
            // let signature = Some(hex::encode(sig.to_bytes()));

            // 6b. Detect Seal Mode
            let seal_mode = openseal_core::SealMode::from_env();
            
            let seal = match seal_mode {
                openseal_core::SealMode::Development => {
                    // Full Seal with all debugging information
                    openseal_core::Seal {
                        signature: hex::encode(sig.to_bytes()),
                        // wax is known to caller, no need to return
                        pub_key: Some(pub_key_hex),
                        a_hash: Some(a_hash_hex),
                        b_hash: Some(b_hash_hex),
                    }
                },
                openseal_core::SealMode::Production => {
                    // Signature-only for maximum security
                    openseal_core::Seal {
                        signature: hex::encode(sig.to_bytes()),
                        pub_key: Some(pub_key_hex), // Required for verification
                        a_hash: Some(a_hash_hex),   // Identity identifier (Public)
                        b_hash: Some(b_hash_hex),   // Binding identifier (Public, opaque)
                    }
                }
            };

            // 7. Merge & Return (State Transition Response)
            let original_body_str = String::from_utf8_lossy(&resp_bytes);
            
            let result_json: serde_json::Value = serde_json::from_str(&original_body_str)
                .unwrap_or(serde_json::Value::String(original_body_str.to_string()));

            let final_response = serde_json::json!({
                "result": result_json,
                "openseal": seal
            });
            
            (StatusCode::OK, axum::Json(final_response)).into_response()
        }
        Err(e) => {
            let error_msg = format!("Internal Application Error: {}", e);
            (StatusCode::BAD_GATEWAY, error_msg).into_response()
        }
    }
}
