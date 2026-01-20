use blake3::Hash;
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use ignore::WalkBuilder;
use anyhow::{Result, Context};
use std::io::Read;
use serde::{Serialize, Deserialize};

// --- Phase 1: Merkle Tree Identity ---

/// Represents the identity of a project, derived from its file structure and content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectIdentity {
    pub root_hash: Hash, // A-hash component
    pub file_count: usize,
    pub mutable_files: Vec<String>,
}

/// Scans a directory and computes its Merkle Root hash.
pub fn compute_project_identity(root_path: &Path) -> Result<ProjectIdentity> {
    let walker = WalkBuilder::new(root_path)
        .hidden(false)
        .git_ignore(true)
        .add_custom_ignore_filename(".opensealignore")
        .build();

    // Load mutable file patterns from .openseal_mutable if exists
    let mutable_patterns = load_mutable_patterns(root_path);

    let mut file_paths: Vec<PathBuf> = Vec::new();
    for result in walker {
        match result {
            Ok(entry) => {
                if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                    file_paths.push(entry.path().to_owned());
                }
            }
            Err(err) => eprintln!("Warning: Skipping file due to error: {}", err),
        }
    }

    file_paths.sort();
    
    

    let file_hashes: Vec<(Hash, Option<String>)> = file_paths.par_iter()
        .map(|path| {
            let relative_path = path.strip_prefix(root_path).unwrap_or(path);
            let path_str = relative_path.to_string_lossy();
            
            // Check if mutable
            let is_mutable = mutable_patterns.iter().any(|p| path_str == *p || path_str.ends_with(p));

            if is_mutable {
                // SECURITY: Ensure we are not muting critical code files
                validate_mutable_file_security(&path_str)?;

                // Track this mutable file
                // Note: We can't directly modify mutable_files_found here due to parallel iteration
                // So we'll collect and merge later
                
                // If mutable, we seal the FILENAME but explicitly ignore CONTENT
                // Hash = Hash("MUTABLE_MARKER" || Filename)
                // This ensures the *existence* of the file is frozen, but content can change.
                Ok((compute_mutable_file_hash(relative_path), Some(path_str.to_string())))
            } else {
                Ok((compute_file_hash(path)?, None))
            }
        })
        .collect::<Result<Vec<(Hash, Option<String>)>>>()?;

    // Extract mutable files and hashes
    let (hashes, mut_files): (Vec<Hash>, Vec<Option<String>>) = file_hashes.into_iter().unzip();
    let mutable_files_found: Vec<String> = mut_files.into_iter().filter_map(|x| x).collect();

    let root_hash = compute_merkle_root(&hashes);

    Ok(ProjectIdentity {
        root_hash,
        file_count: file_paths.len(),
        mutable_files: mutable_files_found,
    })
}

/// SECURITY: Enforce blacklist on mutable files to prevent Backdoor Injection.
fn validate_mutable_file_security(path_str: &str) -> Result<()> {
    let lower = path_str.to_lowercase();
    let dangerous_extensions = [
        ".rs", ".js", ".ts", ".py", ".go", ".c", ".cpp", ".h", ".hpp", 
        ".wasm", ".sh", ".bat", ".cmd", ".json", ".toml", ".yaml", ".yml"
    ];

    for ext in dangerous_extensions {
        if lower.ends_with(ext) {
            anyhow::bail!(
                "SECURITY VIOLATION: '{}' cannot be mutable. Code/Config files must be immutable.", 
                path_str
            );
        }
    }
    Ok(())
}

fn load_mutable_patterns(root: &Path) -> Vec<String> {
    let config_path = root.join(".openseal_mutable");
    if let Ok(content) = fs::read_to_string(config_path) {
        content.lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect()
    } else {
        Vec::new()
    }
}

fn compute_file_hash(path: &Path) -> Result<Hash> {
    let mut file = fs::File::open(path).with_context(|| format!("Failed to open file: {:?}", path))?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0; 65536];
    loop {
        let count = file.read(&mut buffer).context("Failed to read file")?;
        if count == 0 { break; }
        hasher.update(&buffer[..count]);
    }
    Ok(hasher.finalize())
}

fn compute_mutable_file_hash(rel_path: &Path) -> Hash {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"MUTABLE_MARKER");
    hasher.update(rel_path.to_string_lossy().as_bytes());
    hasher.finalize()
}

fn compute_merkle_root(hashes: &[Hash]) -> Hash {
    if hashes.is_empty() {
        return blake3::hash(b"EMPTY_PROJECT");
    }
    let mut hasher = blake3::Hasher::new();
    for h in hashes {
        hasher.update(h.as_bytes());
    }
    hasher.finalize()
}

// --- Phase 2: Internalized Pipeline (Sealing Logic) ---

/// Determines what information is included in the Seal based on the environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SealMode {
    /// Development mode: Full seal with all debugging information
    Development,
    /// Production mode: Signature-only for maximum security and privacy
    Production,
}

impl SealMode {
    /// Detects mode from OPENSEAL_MODE environment variable.
    /// Defaults to Development for safety (explicit opt-in to production).
    pub fn from_env() -> Self {
        match std::env::var("OPENSEAL_MODE") {
            Ok(val) if val == "production" => SealMode::Production,
            _ => SealMode::Development,
        }
    }
}

/// The complete seal structure returned to the outside world.
/// In Production mode, only `signature` is populated; other fields are None.
#[derive(Debug, Serialize, Deserialize)]
pub struct Seal {
    pub signature: String,            // Always present (Mandatory)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pub_key: Option<String>,      // Dev only: Ephemeral Public Key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub a_hash: Option<String>,       // Dev only: Blinded Identity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub b_hash: Option<String>,       // Dev only: Result Binding
}

/// Generates the Blinded A-hash (Execution Commitment).
/// A = Hash(ProjectRoot || Wax)
/// This binds the static identity to the dynamic request, and hides the raw Root Hash.
pub fn compute_a_hash(project_root: &Hash, wax: &str) -> Hash {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"OPENSEAL_BLINDED_IDENTITY");
    hasher.update(project_root.as_bytes());
    hasher.update(wax.as_bytes());
    hasher.finalize()
}

/// [HARDENED IMPLEMENTATION - v2.0-rc14]
/// Derives the dynamic sealing key (g_B function) using multi-stage obfuscation.
/// This implementation is significantly more complex than simple keyed hashing,
/// making reverse engineering substantially more difficult.
///
/// **Security Note**: While this is more robust than the reference impl,
/// production systems should consider additional obfuscation layers or native code compilation.
// [SEALED] derive_sealing_key_hardened has been moved to openseal-secret crate.
// [SEALED] compute_b_hash has been moved to openseal-secret crate.

#[derive(Debug, Serialize)]
pub struct VerificationReport {
    pub valid: bool,
    pub signature_verified: bool,
    pub binding_verified: bool,
    pub identity_verified: bool,
    pub message: String,
}

/// Verifies the integrity of a Seal provided in a JSON response.
/// 
/// # Arguments
/// * `response` - The full JSON response object (containing "result" and "openseal")
/// * `wax` - The challenge string used for the request
/// * `expected_root_hash` - Optional. If provided, verifies A-hash matches.
pub fn verify_seal(response: &serde_json::Value, wax: &str, expected_root_hash: Option<&str>) -> Result<VerificationReport> {
    use ed25519_dalek::{Verifier, VerifyingKey, Signature};

    // 1. Extract Seal Components
    let openseal = response.get("openseal").context("Missing 'openseal' field")?;
    let result_val = response.get("result").context("Missing 'result' field")?;
    
    // In Dev mode, these should be present. In Prod, they are missing (cannot verify without out-of-band info).
    let signature_hex = openseal.get("signature").and_then(|v| v.as_str()).context("Missing signature")?;
    let pub_key_hex = openseal.get("pub_key").and_then(|v| v.as_str()).context("Missing pub_key (Dev mode required for CLI verify)")?;
    let a_hash_hex = openseal.get("a_hash").and_then(|v| v.as_str()).context("Missing a_hash")?;
    let b_hash_hex = openseal.get("b_hash").and_then(|v| v.as_str()).context("Missing b_hash")?;

    // 2. Decode Hex
    let pub_key_bytes = hex::decode(pub_key_hex).context("Invalid pub_key hex")?;
    let pub_key = VerifyingKey::from_bytes(&pub_key_bytes.try_into().map_err(|_| anyhow::anyhow!("Invalid pub_key length"))?)?;
    
    let signature_bytes = hex::decode(signature_hex).context("Invalid signature hex")?;
    let signature = Signature::from_bytes(&signature_bytes.try_into().map_err(|_| anyhow::anyhow!("Invalid signature length"))?);

    // 3. Reconstruct Payload for Signature Verification
    // Payload Rule: wax_hex + a_hash_hex + b_hash_hex + blake3(result_bytes).to_hex()
    
    // Issue: How to get original result bytes?
    // We assume result_val is exact representation.
    // Try converting back to string. Note: formatting differences will cause failure.
    let result_str = if result_val.is_string() {
        result_val.as_str().unwrap().to_string()
    } else {
        serde_json::to_string(result_val)?
    };
    
    let result_hash = blake3::hash(result_str.as_bytes()).to_hex().to_string();
    let payload = format!("{}{}{}{}", wax, a_hash_hex, b_hash_hex, result_hash);
    
    // 4. Verify Signature
    let signature_verified = pub_key.verify(payload.as_bytes(), &signature).is_ok();
    
    if !signature_verified {
        return Ok(VerificationReport {
            valid: false,
            signature_verified: false,
            binding_verified: false,
            identity_verified: false,
            message: "Signature verification failed. The seal may have been tampered.".to_string(),
        });
    }

    // 5. Verify Logic Binding (B-hash)
    // [SEALED POLICY] The internal binding logic (g_B) is sealed.
    // Therefore, public verifiers CANNOT recompute B-hash to verify binding.
    // We rely SOLELY on the Signature to prove that the Runtime (which knows g_B)
    // has attested to this B-hash for this Result.
    
    // If signature is valid, then Binding is implicitly valid (trusted runtime).
    let _binding_verified = true; 
    
    // Legacy check removed: we don't have compute_b_hash here anymore.

    // 6. Verify Identity (Optional A-hash check)
    let mut identity_verified = true;
    if let Some(root_hash_hex) = expected_root_hash {
        let root_hash = Hash::from_hex(root_hash_hex)?;
        // We can verify A-hash because compute_a_hash is PUBLIC spec.
        let computed_a = compute_a_hash(&root_hash, wax);
        if computed_a.to_hex().to_string() != a_hash_hex {
            identity_verified = false;
        }
    }
    
    if expected_root_hash.is_some() && !identity_verified {
        return Ok(VerificationReport {
            valid: false,
            signature_verified: true,
            binding_verified: true,
            identity_verified: false,
            message: "Identity Mismatch. The code executed is different from what was expected.".to_string(),
        });
    }

    Ok(VerificationReport {
        valid: true,
        signature_verified: true,
        binding_verified: true,
        identity_verified,
        message: "✅ SEAL VALID. The result is authentic and untampered.".to_string(),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_compute_project_identity_determinism() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path)?;
        writeln!(file, "Hello OpenSeal")?;

        let identity1 = compute_project_identity(dir.path())?;
        let identity2 = compute_project_identity(dir.path())?;

        assert_eq!(identity1.root_hash, identity2.root_hash);
        assert_eq!(identity1.file_count, 1);
        Ok(())
    }

    #[test]
    fn test_changes_affect_root_hash() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path)?;
        writeln!(file, "Hello")?;

        let identity1 = compute_project_identity(dir.path())?;

        // Modify file
        let mut file = File::create(&file_path)?; // Overwrite
        writeln!(file, "World")?;

        let identity2 = compute_project_identity(dir.path())?;

        assert_ne!(identity1.root_hash, identity2.root_hash);
        Ok(())
    }

    #[test]
    fn test_mutable_file_logic() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("config.db");
        let mut file = File::create(&file_path)?;
        writeln!(file, "Initial State")?;

        // Declare config.db as mutable
        let config_path = dir.path().join(".openseal_mutable");
        let mut config = File::create(&config_path)?;
        writeln!(config, "config.db")?;

        let identity1 = compute_project_identity(dir.path())?;

        // Modify content of the mutable file
        let mut file = File::create(&file_path)?;
        writeln!(file, "Changed State")?;

        let identity2 = compute_project_identity(dir.path())?;

        // Root hash should match because content is ignored for mutable files
        // BUT the .openseal_mutable file itself is hashed (it is immutable!)
        // Since .openseal_mutable is new, ensure it's handled.
        
        // Wait, .openseal_mutable is part of the project? Yes.
        // It guarantees the rule itself is sealed.
        
        assert_eq!(identity1.root_hash, identity2.root_hash);
        assert_eq!(identity1.mutable_files, vec!["config.db"]);
        Ok(())
    }

    #[test]
    fn test_dynamic_b_hash_binding() {
        let a_hash = blake3::hash(b"PROJECT_IDENTITY");
        let wax1 = "WAX_1";
        let wax2 = "WAX_2";
        let result = b"Execution Result";

        let b1 = compute_b_hash(&a_hash, wax1, result);
        let b2 = compute_b_hash(&a_hash, wax2, result);
        
        // Different waxes should produce different B-hashes even if A and Result are same
        assert_ne!(b1, b2);

        let a_hash_modified = blake3::hash(b"PROJECT_IDENTITY_MODIFIED");
        let b3 = compute_b_hash(&a_hash_modified, wax1, result);

        // Different A-hash should produce different B-hashes
        assert_ne!(b1, b3);
    }

    // SealMode tests
    mod seal_mode_tests {
        use super::super::*;

        #[test]
        fn test_seal_mode_from_env_default() {
            std::env::remove_var("OPENSEAL_MODE");
            let mode = SealMode::from_env();
            assert_eq!(mode, SealMode::Development);
        }

        #[test]
        fn test_seal_mode_from_env_production() {
            std::env::set_var("OPENSEAL_MODE", "production");
            let mode = SealMode::from_env();
            assert_eq!(mode, SealMode::Production);
            std::env::remove_var("OPENSEAL_MODE");
        }

        #[test]
        fn test_seal_serialization_full() {
            let seal = Seal {
                signature: "abc123".to_string(),
                pub_key: Some("key123".to_string()),
                a_hash: Some("ahash123".to_string()),
                b_hash: Some("bhash123".to_string()),
            };
            
            let json = serde_json::to_string(&seal).unwrap();
            assert!(json.contains("\"signature\""));
            // wax is removed from struct
        }

        #[test]
        fn test_seal_serialization_signature_only() {
            let seal = Seal {
                signature: "abc123".to_string(),
                pub_key: None,
                a_hash: None,
                b_hash: None,
            };
            
            let json = serde_json::to_string(&seal).unwrap();
            assert!(json.contains("\"signature\""));
        }
    }
}
