use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::any,
    Router,
};
use openseal_core::{compute_a_hash, compute_b_hash, compute_project_identity, ProjectIdentity, Seal};
use rand::{rngs::OsRng, RngCore};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use ed25519_dalek::{SigningKey, Signer};

#[derive(Clone)]
struct AppState {
    target_url: String,
    project_identity: ProjectIdentity,
    signing_key: Option<SigningKey>,
}

pub async fn run_proxy_server(port: u16, target_url: String, project_root: PathBuf, use_key: bool) -> anyhow::Result<()> {
    println!("🔐 OpenSeal Runtime v0.2.0 Starting...");
    println!("   Target App: {}", target_url);
    println!("   Project Root: {:?}", project_root);

    // 1. Static Commitment: Compute A-hash at startup
    println!("   Scanning project identity...");
    let identity = compute_project_identity(&project_root)?;
    println!("   ✅ A-hash (Root): {}", identity.root_hash.to_hex());
    println!("   📄 Files Sealed: {}", identity.file_count);

    // Generate a strictly ephemeral signing key for this runtime session IF enabled
    let signing_key = if use_key {
        let mut csprng = OsRng;
        let mut key_bytes = [0u8; 32];
        csprng.fill_bytes(&mut key_bytes);
        let key = SigningKey::from_bytes(&key_bytes);
        let verifying_key = key.verifying_key();
        println!("   🔑 Public Key (Ephemeral): {}", hex::encode(verifying_key.to_bytes()));
        Some(key)
    } else {
        println!("   ⚠️  Key Generation DISABLED (Unsigned Mode)");
        None
    };

    let state = Arc::new(AppState {
        target_url,
        project_identity: identity,
        signing_key,
    });

    let app = Router::new()
        .route("/*path", any(handler))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    println!("🚀 OpenSeal Running on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn handler(State(state): State<Arc<AppState>>, req: Request<Body>) -> impl IntoResponse {
    let client = reqwest::Client::new();
    
    // 2. Dynamic Trajectory: Generate Wax (Challenge/Context)
    let mut wax_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut wax_bytes);
    let wax_hex = hex::encode(wax_bytes);

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
        .headers(headers) // Forward headers with Nonce
        .body(body_bytes)
        .send()
        .await;

    match response_result {
        Ok(resp) => {
            // 4. Result Capture (Egress Interception)
            let status = resp.status();
            let resp_bytes = resp.bytes().await.unwrap_or_default();

            // 5. Atomic Sealing (B-hash generation)
            // B = b_G(Result, A, Wax)
            let b_hash = compute_b_hash(&a_hash, &wax_hex, &resp_bytes);
            let b_hash_hex = b_hash.to_hex().to_string();

            // 6. Optional Sign the Seal
            // Signature = Sign(Wax || A || B || SHA256(Result))
            let (signature, pub_key_hex) = if let Some(key) = &state.signing_key {
                 // Calculate Result Hash for binding
                 let result_hash = blake3::hash(&resp_bytes).to_hex().to_string();
                 
                 let sign_payload = format!("{}{}{}{}", wax_hex, a_hash_hex, b_hash_hex, result_hash);
                 let sig = key.sign(sign_payload.as_bytes());
                 let pub_key = hex::encode(key.verifying_key().to_bytes());
                 
                 (Some(hex::encode(sig.to_bytes())), pub_key)
            } else {
                (None, String::from("UNSIGNED_MODE"))
            };

            let seal = Seal {
                wax: wax_hex,
                pub_key: pub_key_hex,
                a_hash: a_hash_hex,
                b_hash: b_hash_hex,
                signature,
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
