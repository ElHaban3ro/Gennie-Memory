//! SHA-256 for local audit fingerprints; never an authentication mechanism.
use sha2::{Digest, Sha256};
pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn known_sha256_vector() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
