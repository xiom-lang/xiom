// XIOM Package Manager -- ed25519 signing + trust model (Stage 5 supply chain).
//
// Audit/plan requirement: "ed25519 signatures + trust model" -- the registry
// client verified sha256 digests but both the digest and the artifact came
// from the same (unauthenticated) index, so a compromised registry could
// serve a matching pair. Signatures bind the artifact bytes to a KEY:
//
//   * `xiom pkg keygen`      -> ~/.xiom/keys/default.key (hex secret)
//   * `xiom pkg trust --registry URL --key HEX` pins a registry key and
//                              writes ~/.xiom/trusted_keys.json
//   * `xiom pkg sign FILE`   -> FILE.sig (hex)
//   * `xiom pkg verify FILE SIG [--key HEX]`
//
// install_from_registry enforces the signature when the registry is TRUSTED
// (a key is pinned) and the index publishes one; untrusted registries keep
// the existing sha256+lockfile checks and get a TOFU hint.

use std::collections::BTreeMap;
use std::path::PathBuf;

/// A signing keypair. The secret is kept as the 32-byte seed in hex; the
/// public key is derived on demand. `Debug` redacts the secret.
#[derive(Clone)]
pub struct KeyPair {
    signing: ed25519_dalek::SigningKey,
}

impl std::fmt::Debug for KeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyPair")
            .field("public", &self.public_hex())
            .field("secret", &"<redacted>")
            .finish()
    }
}

impl KeyPair {
    /// Generate a fresh keypair from OS randomness.
    pub fn generate() -> Result<KeyPair, String> {
        let mut seed = [0u8; 32];
        getrandom::getrandom(&mut seed).map_err(|e| format!("OS randomness unavailable: {e}"))?;
        Ok(KeyPair { signing: ed25519_dalek::SigningKey::from_bytes(&seed) })
    }

    pub fn from_secret_hex(hex: &str) -> Result<KeyPair, String> {
        let bytes = hex_decode(hex.trim()).ok_or_else(|| "secret key is not valid hex".to_string())?;
        if bytes.len() != 32 {
            return Err(format!("secret key must be 32 bytes (64 hex chars), got {}", bytes.len()));
        }
        let mut seed = [0u8; 32];
        seed.copy_from_slice(&bytes);
        Ok(KeyPair { signing: ed25519_dalek::SigningKey::from_bytes(&seed) })
    }

    pub fn secret_hex(&self) -> String {
        hex_encode(self.signing.to_bytes().as_ref())
    }

    pub fn public_hex(&self) -> String {
        hex_encode(self.signing.verifying_key().as_bytes())
    }

    /// Sign bytes; returns the 64-byte signature as lowercase hex.
    pub fn sign(&self, data: &[u8]) -> String {
        use ed25519_dalek::Signer;
        hex_encode(&self.signing.sign(data).to_bytes())
    }
}

/// Verify a hex signature over `data` with a hex public key.
pub fn verify(public_hex: &str, data: &[u8], signature_hex: &str) -> Result<(), String> {
    let pk = hex_decode(public_hex.trim()).ok_or_else(|| "public key is not valid hex".to_string())?;
    let pk: [u8; 32] = pk.try_into().map_err(|v: Vec<u8>| format!("public key must be 32 bytes, got {}", v.len()))?;
    let verifying = ed25519_dalek::VerifyingKey::from_bytes(&pk)
        .map_err(|e| format!("invalid ed25519 public key: {e}"))?;
    let sig = hex_decode(signature_hex.trim()).ok_or_else(|| "signature is not valid hex".to_string())?;
    let sig: [u8; 64] = sig.try_into().map_err(|v: Vec<u8>| format!("signature must be 64 bytes, got {}", v.len()))?;
    let sig = ed25519_dalek::Signature::from_bytes(&sig);
    verifying.verify_strict(data, &sig)
        .map_err(|_| "SIGNATURE MISMATCH: the artifact was not signed by the trusted key".to_string())
}

/// Short, human-comparable fingerprint of a public key (first 8 bytes hex,
/// grouped). Used in `keygen` output and trust errors.
pub fn fingerprint(public_hex: &str) -> String {
    let bytes = hex_decode(public_hex.trim()).unwrap_or_default();
    bytes.iter().take(8).map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(":")
}

// ============================================================================
// Trust store -- `<XIOM_HOME>/trusted_keys.json`  { "<registry>": "<pubhex>" }
// ============================================================================

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct TrustStore {
    /// registry base URL -> ed25519 public key (hex).
    keys: BTreeMap<String, String>,
    #[serde(skip)]
    path: PathBuf,
}

impl TrustStore {
    /// Default location: `$XIOM_HOME/trusted_keys.json`, falling back to
    /// `$HOME/.xiom/trusted_keys.json` (USERPROFILE on Windows).
    pub fn load() -> TrustStore {
        Self::load_from(default_xiom_home().join("trusted_keys.json"))
    }

    pub fn load_from(path: PathBuf) -> TrustStore {
        let mut store = TrustStore { keys: BTreeMap::new(), path };
        if let Ok(text) = std::fs::read_to_string(&store.path) {
            match serde_json::from_str::<TrustStore>(&text) {
                Ok(parsed) => {
                    store.keys = parsed.keys;
                }
                Err(e) => {
                    eprintln!("xiom pkg: ignoring malformed {}: {e}", store.path.display());
                }
            }
        }
        store
    }

    pub fn get(&self, registry: &str) -> Option<&String> {
        self.keys.get(&normalize_registry(registry))
    }

    pub fn entries(&self) -> impl Iterator<Item = (&String, &String)> {
        self.keys.iter()
    }

    /// Pin (or replace) the key for a registry and persist.
    pub fn pin(&mut self, registry: &str, public_hex: &str) -> Result<(), String> {
        // Validate before persisting: a typo'd key must not silently disable
        // the fail-closed install check.
        verify(public_hex, b"pin-check", &"00".repeat(64)) // expected mismatch, but parse errors surface
            .err()
            .filter(|e| !e.contains("SIGNATURE MISMATCH"))
            .map_or(Ok(()), |e| Err(format!("refusing to pin invalid key: {e}")))?;
        self.keys.insert(normalize_registry(registry), public_hex.trim().to_lowercase());
        self.save()
    }

    pub fn save(&self) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
        }
        let text = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&self.path, text + "\n")
            .map_err(|e| format!("cannot write {}: {e}", self.path.display()))
    }
}

/// `<XIOM_HOME>` or `$HOME/.xiom`.
pub fn default_xiom_home() -> PathBuf {
    if let Ok(home) = std::env::var("XIOM_HOME") {
        if !home.trim().is_empty() {
            return PathBuf::from(home);
        }
    }
    let base = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(base).join(".xiom")
}

/// Registry URLs compare case-insensitively and ignore a trailing slash.
fn normalize_registry(registry: &str) -> String {
    registry.trim().trim_end_matches('/').to_lowercase()
}

// ============================================================================
// Hex helpers (no extra dependency)
// ============================================================================

pub fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn hex_decode(text: &str) -> Option<Vec<u8>> {
    let t = text.trim();
    if t.len() % 2 != 0 {
        return None;
    }
    let mut out = Vec::with_capacity(t.len() / 2);
    let bytes = t.as_bytes();
    for pair in bytes.chunks(2) {
        let hi = (pair[0] as char).to_digit(16)?;
        let lo = (pair[1] as char).to_digit(16)?;
        out.push((hi * 16 + lo) as u8);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_verify_round_trip_and_tamper() {
        let kp = KeyPair::generate().expect("keygen");
        let data = b"package-tarball-bytes";
        let sig = kp.sign(data);
        assert_eq!(sig.len(), 128, "64-byte signature in hex");
        assert!(verify(&kp.public_hex(), data, &sig).is_ok());
        // Tampered payload
        let err = verify(&kp.public_hex(), b"package-tarball-bytez", &sig).unwrap_err();
        assert!(err.contains("SIGNATURE MISMATCH"), "{err}");
        // Wrong key
        let other = KeyPair::generate().expect("keygen");
        assert!(verify(&other.public_hex(), data, &sig).is_err());
        // Uppercase hex inputs are accepted
        assert!(verify(&kp.public_hex().to_uppercase(), data, &sig.to_uppercase()).is_ok());
    }

    #[test]
    fn secret_round_trip_and_bad_inputs() {
        let kp = KeyPair::generate().expect("keygen");
        let restored = KeyPair::from_secret_hex(&kp.secret_hex()).expect("restore");
        assert_eq!(restored.public_hex(), kp.public_hex());
        assert!(KeyPair::from_secret_hex("zz").is_err());
        assert!(KeyPair::from_secret_hex("abcd").unwrap_err().contains("32 bytes"));
        assert!(verify("not-hex", b"x", "00").is_err());
        assert!(verify(&kp.public_hex(), b"x", "00").unwrap_err().contains("64 bytes"));
    }

    #[test]
    fn trust_store_pins_and_reloads() {
        let base = std::env::temp_dir().join(format!(
            "xiom_trust_{}_{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0)
        ));
        let path = base.join("trusted_keys.json");
        let mut store = TrustStore::load_from(path.clone());
        let kp = KeyPair::generate().expect("keygen");
        store.pin("https://registry.xiom.dev/", &kp.public_hex()).expect("pin");
        assert_eq!(store.get("https://registry.xiom.dev").map(|s| s.as_str()), Some(kp.public_hex().as_str()));
        // Reload from disk
        let reloaded = TrustStore::load_from(path);
        assert_eq!(reloaded.get("https://REGISTRY.xiom.dev").map(|s| s.as_str()), Some(kp.public_hex().as_str()));
        // Invalid keys are refused
        let mut store2 = TrustStore::load_from(base.join("other.json"));
        assert!(store2.pin("https://x", "not-hex").is_err());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn hex_and_fingerprint() {
        assert_eq!(hex_encode(&[0x0a, 0xff]), "0aff");
        assert_eq!(hex_decode("0AFF"), Some(vec![0x0a, 0xff]));
        assert_eq!(hex_decode("abc"), None);
        let fp = fingerprint(&"ab".repeat(32));
        assert_eq!(fp, "ab:ab:ab:ab:ab:ab:ab:ab");
    }
}
