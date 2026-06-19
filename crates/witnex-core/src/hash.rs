//! SHA-256 helpers for constructing [`Digest`] commitments.
//!
//! Every commitment in an [`ExecutionTrace`](crate::ExecutionTrace) is the
//! SHA-256 digest of some value's byte encoding. This module provides the one
//! place those digests are computed, so the hashing is consistent across the
//! runtime.

use sha2::{Digest as _, Sha256};

use crate::Digest;

impl Digest {
    /// Compute the SHA-256 digest of `bytes`.
    ///
    /// ```
    /// use witnex_core::Digest;
    ///
    /// // SHA-256("abc")
    /// assert_eq!(
    ///     Digest::sha256("abc").to_string(),
    ///     "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
    /// );
    /// ```
    pub fn sha256(bytes: impl AsRef<[u8]>) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(bytes.as_ref());
        let mut buf = [0u8; Self::LEN];
        buf.copy_from_slice(&hasher.finalize());
        Digest(buf)
    }
}

#[cfg(test)]
mod tests {
    use crate::Digest;

    #[test]
    fn sha256_empty_matches_known_vector() {
        assert_eq!(
            Digest::sha256("").to_string(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        );
    }

    #[test]
    fn sha256_abc_matches_known_vector() {
        assert_eq!(
            Digest::sha256("abc").to_string(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        );
    }

    #[test]
    fn sha256_is_deterministic_and_input_sensitive() {
        assert_eq!(Digest::sha256("witnex"), Digest::sha256("witnex"));
        assert_ne!(Digest::sha256("witnex"), Digest::sha256("Witnex"));
    }
}
