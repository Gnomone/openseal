use clap::{Parser, Subcommand};
use std::path::Path;
use std::fs;
use anyhow::{Result, anyhow, Context};
use std::process::Command;
use hex;
use ed25519_dalek::{VerifyingKey, Signature, Verifier};
use std::convert::TryInto;
use serde_json;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build OpenSeal Identity from Docker Image
    Build {
        /// Docker image (must include digest: user/api@sha256:...)
        #[arg(short, long)]
        image: String,
    },
    /// Run the OpenSeal-wrapped container
    Run {
        /// Docker image to run (with digest)
        #[arg(short, long)]
        image: String,

        /// Port to expose (OpenSeal Proxy)
        #[arg(short, long, default_value = "8080")]
        port: u16,

        /// Allow network access to specific domains
        #[arg(long)]
        allow_network: Vec<String>,
    },
    /// Verify a sealed response
    Verify {
        /// JSON response file path
        #[arg(long, short)]
        response: String,

        /// Wax challenge string used for the request
        #[arg(long, short)]
        wax: String,

        /// Optional: Expected Root Hash (Image Digest)
        #[arg(long)]
        root_hash: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { image } => {
            println!("🐳 OpenSeal v1.0.0-alpha.1: Docker-Based Identity Builder");
            println!("   Image: {}", image);

            // 1. Extract Digest (support both formats)
            let digest = if image.contains("@sha256:") {
                // Registry format: user/api@sha256:abc...
                image.split('@').nth(1)
                    .ok_or_else(|| anyhow!("Invalid image format"))?
                    .to_string()
            } else {
                // Local development: use Image ID
                let output = Command::new("docker")
                    .args(&["inspect", &image, "--format={{.Id}}"])
                    .output()?;

                if !output.status.success() {
                    return Err(anyhow!(
                        "❌ Image '{}' not found.\n\
                         \n\
                         For production: Use digest format (user/api@sha256:...)\n\
                         For development: Ensure image exists locally",
                        image
                    ));
                }

                let id = String::from_utf8(output.stdout)?.trim().to_string();
                println!("   ⚠️  Development mode: Using Image ID instead of digest");
                id
            };

            println!("   ✅ Root Hash: {}", digest);

            // 3. Create openseal.json (v1 format)
            let openseal_json = serde_json::json!({
                "version": "1.0.0",
                "image": {
                    "reference": image,
                    "digest": digest,
                    "created_at": chrono::Utc::now().to_rfc3339()
                },
                "identity": {
                    "root_hash": digest,
                    "seal_version": "2.0"
                }
            });

            let json_path = Path::new("openseal.json");
            fs::write(json_path, serde_json::to_string_pretty(&openseal_json)?)?;

            println!("   📝 openseal.json created");
            println!();
            println!("🎉 Build complete!");
            println!("   Root Hash: {}", digest);
            println!();
            println!("⚠️  IMPORTANT: Ensure the image is pushed to a registry.");
            println!("   Other environments will fail if they can't pull this exact digest.");
        }
        Commands::Run { image, port, allow_network } => {
            println!("🚀 OpenSeal v1.0.0-alpha.1: Starting Container");
            println!("   Image: {}", image);
            println!("   Public Port: {}", port);

            // 1. Load openseal.json
            let json_path = Path::new("openseal.json");
            if !json_path.exists() {
                return Err(anyhow!("❌ openseal.json not found. Run 'openseal build' first."));
            }

            let json_content = fs::read_to_string(json_path)?;
            let json: serde_json::Value = serde_json::from_str(&json_content)?;
            
            let expected_digest = json["identity"]["root_hash"]
                .as_str()
                .ok_or_else(|| anyhow!("Invalid openseal.json"))?;

            // 2. Verify image digest matches
            let actual_digest = if image.contains("@sha256:") {
                image.split('@').nth(1)
                    .ok_or_else(|| anyhow!("Image must include digest"))?
                    .to_string()
            } else {
                // Development mode: get Image ID
                let output = Command::new("docker")
                    .args(&["inspect", &image, "--format={{.Id}}"])
                    .output()?;
                String::from_utf8(output.stdout)?.trim().to_string()
            };

            if expected_digest != actual_digest {
                return Err(anyhow!(
                    "🚨 Security Breach: Image Digest Mismatch!\n\
                     Expected: {}\n\
                     Actual:   {}\n\
                     Image has been modified since build.",
                    expected_digest, actual_digest
                ));
            }

            println!("   ✅ Digest verified: {}", actual_digest);

            // 3. Start container (Daemon Mode)
            let container_name = format!("openseal_runtime_{}", chrono::Utc::now().timestamp());
            let internal_port = 3000; // Default internal port
            let port_mapping = format!("127.0.0.1:{}:{}", internal_port, internal_port);

            let mut docker_args = vec![
                "run", "-d",
                "--name", &container_name,
                "--rm",
                "--read-only",
                "--cap-drop=ALL",
                "--security-opt=no-new-privileges",
                "-p", &port_mapping,
            ];

            // Network setup
            if allow_network.is_empty() {
                // v1.0.0-alpha.1: Use bridge for development (allows external API calls)
                // TODO: Implement strict isolation with whitelist in beta
                docker_args.push("--network=bridge");
                println!("   ⚠️  Network: bridge (development mode - no whitelist yet)");
            } else {
                println!("   📡 Network whitelist: {:?}", allow_network);
                // TODO: Implement DNS resolve + iptables
                docker_args.push("--network=bridge");
            }

            docker_args.push(&image);

            println!("   🐳 Starting container...");
            let output = Command::new("docker")
                .args(&docker_args)
                .output()?;

            if !output.status.success() {
                return Err(anyhow!(
                    "Failed to start container:\n{}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }

            let container_id = String::from_utf8(output.stdout)?.trim().to_string();
            println!("   ✅ Container started: {}", &container_id[..12]);

            // 4. Health Check (wait for port)
            println!("   ⏳ Waiting for health check...");
            use std::net::TcpStream;
            use std::time::{Duration, Instant};

            let start = Instant::now();
            let addr = format!("127.0.0.1:{}", internal_port);
            let mut connected = false;

            while start.elapsed() < Duration::from_secs(30) {
                if TcpStream::connect(&addr).is_ok() {
                    connected = true;
                    break;
                }
                std::thread::sleep(Duration::from_millis(500));
            }

            if !connected {
                // Cleanup
                let _ = Command::new("docker").args(&["stop", &container_id]).output();
                return Err(anyhow!("Health check timeout: Container failed to respond"));
            }

            println!("   ✅ Container ready");
            println!();
            println!("🔐 Starting OpenSeal Proxy on port {}...", port);
            println!("   → Forwarding to container: 127.0.0.1:{}", internal_port);
            println!();

            // 5. Create ProjectIdentity from openseal.json (v1 format)
            let root_hash_str = expected_digest;
            let root_hash_bytes = if root_hash_str.starts_with("sha256:") {
                hex::decode(&root_hash_str[7..])
                    .map_err(|e| anyhow!("Invalid digest hex: {}", e))?
            } else {
                return Err(anyhow!("Invalid digest format"));
            };

            let root_hash = blake3::Hash::from_bytes(
                root_hash_bytes.as_slice().try_into()
                    .map_err(|_| anyhow!("Invalid hash length"))?
            );

            let project_identity = openseal_core::ProjectIdentity {
                root_hash,
                file_count: 0, // Docker images don't have file count
                mutable_files: vec![], // No mutable files in v1 (containers are immutable)
            };

            // 6. Start Proxy Server (blocking)
            let target_url = format!("http://127.0.0.1:{}", internal_port);
            let project_root = std::env::current_dir()?;

            println!("📡 Proxy Server Ready!");
            println!("   Public: http://0.0.0.0:{}", port);
            println!("   Target: {}", target_url);
            println!();
            println!("💡 Test with:");
            println!("   curl -H \"X-OpenSeal-Wax: test123\" http://localhost:{}/api/v1/price/BTC", port);
            println!();

            // Spawn log streaming in background
            let container_id_clone = container_id.clone();
            tokio::spawn(async move {
                let _ = tokio::process::Command::new("docker")
                    .args(&["logs", "-f", &container_id_clone])
                    .stdout(std::process::Stdio::inherit())
                    .stderr(std::process::Stdio::inherit())
                    .spawn();
            });

            // Register Ctrl+C handler for cleanup
            let container_id_cleanup = container_id.clone();
            tokio::spawn(async move {
                tokio::signal::ctrl_c().await.ok();
                println!("\n🛑 Shutting down...");
                let _ = tokio::process::Command::new("docker")
                    .args(&["stop", &container_id_cleanup])
                    .output()
                    .await;
                println!("   ✅ Container stopped");
                std::process::exit(0);
            });

            // Start the proxy (blocking call)
            openseal_runtime::run_proxy_server(port, target_url, project_root, project_identity).await?;
        }
        Commands::Verify { response, wax, root_hash } => {
            verify_seal(&response, &wax, root_hash.as_deref())?;
        }
    }

    Ok(())
}

/// Verifies a sealed response file
fn verify_seal(response_path: &str, wax: &str, expected_root: Option<&str>) -> Result<()> {
    println!("🔍 Verifying seal...");

    // 1. Read and Parse JSON
    let content = fs::read_to_string(response_path)
        .context(format!("Failed to read response file: {}", response_path))?;
    
    let json: serde_json::Value = serde_json::from_str(&content)
        .context("Failed to parse JSON response")?;

    // 2. Extract Components
    let openseal = json.get("openseal")
        .ok_or_else(|| anyhow!("Missing 'openseal' field"))?;
    
    let result = json.get("result")
        .ok_or_else(|| anyhow!("Missing 'result' field"))?;

    let a_hash_hex = openseal.get("a_hash")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing or invalid 'a_hash'"))?;
        
    let b_hash_hex = openseal.get("b_hash")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing or invalid 'b_hash'"))?;
        
    let signature_hex = openseal.get("signature")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing or invalid 'signature'"))?;
        
    let pub_key_hex = openseal.get("pub_key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing or invalid 'pub_key'"))?;

    println!("   🔑 Public Key: {}", pub_key_hex);
    println!("   🆔 A-hash:    {}", a_hash_hex);

    // 3. Compute Result Hash (Canonicalization)
    // MUST match runtime logic: serde_json::to_string(&result)
    let result_str = serde_json::to_string(result)?;
    let result_hash = blake3::hash(result_str.as_bytes()).to_hex().to_string();

    // 4. Reconstruct Message
    // format!("{}{}{}{}", wax_hex, a_hash_hex, b_hash_hex, result_hash)
    // IMPORTANT: Verify format matches openseal-runtime/src/lib.rs sign_payload
    
    // In runtime: wax_hex is passed as string
    let message = format!("{}{}{}{}", wax, a_hash_hex, b_hash_hex, result_hash);
    
    // 5. Verify Signature
    let pub_key_bytes = hex::decode(pub_key_hex)
        .context("Failed to decode public key hex")?;
    let pub_key: [u8; 32] = pub_key_bytes.try_into()
        .map_err(|_| anyhow!("Invalid public key length"))?;
    
    let verifying_key = VerifyingKey::from_bytes(&pub_key)
        .map_err(|_| anyhow!("Invalid public key format"))?;

    let signature_vec = hex::decode(signature_hex)
        .context("Failed to decode signature hex")?;
    
    let signature_bytes: [u8; 64] = signature_vec.try_into()
        .map_err(|_| anyhow!("Invalid signature length. Expected 64 bytes."))?;
        
    let signature = Signature::from_bytes(&signature_bytes);

    match verifying_key.verify(message.as_bytes(), &signature) {
        Ok(_) => {
            println!("   ✅ Signature Verified!");
        }
        Err(e) => {
            return Err(anyhow!("❌ Signature Verification Failed: {}", e));
        }
    }

    // 6. Optional: Verify Identity (Root Hash)
    if let Some(expected) = expected_root {
        println!("🔍 Verifying identity...");
        
        let root_hash_hex = if expected.starts_with("sha256:") {
            &expected[7..]
        } else {
            expected
        };

        let root_hash_bytes = hex::decode(root_hash_hex)
            .map_err(|e| anyhow!("Invalid digest hex: {}", e))?;
            
        let root_hash = blake3::Hash::from_bytes(
            root_hash_bytes.as_slice().try_into()
                .map_err(|_| anyhow!("Invalid hash length"))?
        );

        // a_hash = Blake3(RootHash || Wax) (Using Core)
        let computed_a = openseal_core::compute_a_hash(&root_hash, wax);
        let computed_a_hex = computed_a.to_hex().to_string();
        
        if computed_a_hex != a_hash_hex {
            return Err(anyhow!(
                "❌ Identity Mismatch!\n   Expected A-hash: {}\n   Actual A-hash:   {}",
                computed_a_hex, a_hash_hex
            ));
        }
        println!("   ✅ Identity Verified (Matches Root Hash)");
    }

    Ok(())
}
