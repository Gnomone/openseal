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
use anyhow::anyhow;

#[derive(Clone)]
struct AppState {
    target_url: String,
    project_identity: ProjectIdentity,
    signing_key: SigningKey,
}

pub async fn run_proxy_server(port: u16, target_url: String, project_root: PathBuf) -> anyhow::Result<()> {
    println!("🔐 OpenSeal Runtime v{} Starting...", env!("CARGO_PKG_VERSION"));
    println!("   Target App: {}", target_url);
    println!("   Project Root: {:?}", project_root);

    // 1. Static Commitment: Compute A-hash at startup
    println!("   Scanning project identity...");
    let live_identity = compute_project_identity(&project_root)?;
    println!("   ✅ Live A-hash: {}", live_identity.root_hash.to_hex());
    println!("   📄 Files Sealed: {}", live_identity.file_count);
    
    // 2. Load Expected Hash from openseal.json
    let manifest_path = project_root.join("openseal.json");
    if !manifest_path.exists() {
        eprintln!("\n⚠️  WARNING: openseal.json not found in project root");
        eprintln!("   Integrity verification skipped.");
        eprintln!("   This is acceptable for development, but NOT recommended for production.\n");
    } else {
        // Load and parse manifest
        let manifest_content = match std::fs::read_to_string(&manifest_path) {
            Ok(content) => content,
            Err(e) => return Err(anyhow!("Failed to read openseal.json: {}", e)),
        };
        
        let manifest: serde_json::Value = match serde_json::from_str(&manifest_content) {
            Ok(m) => m,
            Err(e) => return Err(anyhow!("Failed to parse openseal.json: {}", e)),
        };
        
        // Extract expected hash
        let expected_hash_array = manifest["identity"]["root_hash"]
            .as_array()
            .ok_or_else(|| anyhow!("Invalid openseal.json: missing identity.root_hash"))?;
        
        let expected_hash_bytes: Vec<u8> = expected_hash_array
            .iter()
            .filter_map(|v: &serde_json::Value| v.as_u64().map(|n| n as u8))
            .collect();
        
        if expected_hash_bytes.len() != 32 {
            return Err(anyhow!("Invalid hash length in openseal.json: expected 32 bytes, got {}", expected_hash_bytes.len()));
        }
        
        // 3. Verify Integrity
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

    // Generate a strictly ephemeral signing key for this runtime session (Mandatory in v2.0)
    let mut csprng = OsRng;
    let mut key_bytes = [0u8; 32];
    csprng.fill_bytes(&mut key_bytes);
    let key = SigningKey::from_bytes(&key_bytes);
    let verifying_key = key.verifying_key();
    println!("   🔑 Public Key (Ephemeral): {}", hex::encode(verifying_key.to_bytes()));

    let state = Arc::new(AppState {
        target_url,
        project_identity: live_identity,
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
