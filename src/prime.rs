//! Implements the Keccak-prime function.

use crate::{
    expansion::{expand, INPUT_HASH_SIZE, NONCE_SIZE},
    keccak::Keccak,
    Hasher,
};
use ::vdf::{InvalidIterations, PietrzakVDFParams, VDFParams, VDF};
use std::error::Error;
use std::fmt;

/// Keccak-prime error.
#[derive(Debug)]
pub enum KeccakPrimeError {
    /// Opaque AES function failure.
    AesError(aes_gcm_siv::aead::Error),

    /// An error return indicating an invalid number of VDF iterations.  The string is a
    /// human-readable message describing the valid iterations.  It should not be
    /// interpreted by programs.
    VdfInvalidIterations(InvalidIterations),
}

impl From<aes_gcm_siv::aead::Error> for KeccakPrimeError {
    fn from(e: aes_gcm_siv::aead::Error) -> Self {
        Self::AesError(e)
    }
}

impl From<InvalidIterations> for KeccakPrimeError {
    fn from(e: InvalidIterations) -> Self {
        Self::VdfInvalidIterations(e)
    }
}

impl fmt::Display for KeccakPrimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeccakPrimeError::AesError(e) => write!(f, "AES error: {}", e),
            KeccakPrimeError::VdfInvalidIterations(e) => {
                write!(f, "VDF invalid iterations: {:?}", e)
            }
        }
    }
}

impl Error for KeccakPrimeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            KeccakPrimeError::AesError(_err) => None, // aes_gcm_siv::Error doesn't implement the Error trait
            KeccakPrimeError::VdfInvalidIterations(_err) => None, // InvalidIterations doesn't implement the Error trait
        }
    }
}

/// Keccak-prime function.
///
/// ### Arguments
/// - `prev_hash`: previous block hash.
/// - `root_hash`: Merkle root hash.
/// - `nonce`: block nonce.
/// - `penalty`: applied penalty (regulates a number of extra Keccak permutations).
/// - `delay`: delay parameter used in the VDF function.
/// - `vdf_iterations`: a number of VDF iterations.
pub fn prime(
    prev_hash: [u8; INPUT_HASH_SIZE],
    root_hash: [u8; INPUT_HASH_SIZE],
    nonce: [u8; NONCE_SIZE],
    penalty: usize,
    delay: u64,
    vdf_iterations: usize,
) -> Result<[u8; INPUT_HASH_SIZE], KeccakPrimeError> {
    // Expand the block.
    let block = expand(prev_hash, root_hash, nonce)?;

    // Execute a chain of VDFs.
    let mut vdf_output = block;
    for _i in 0..vdf_iterations {
        let pietrzak_vdf = PietrzakVDFParams(2048).new();
        vdf_output = pietrzak_vdf.solve(&vdf_output, delay)?;
    }

    // Construct a Keccak function with rate=1088 and capacity=512.
    let mut keccak = Keccak::new(1088 / 8);
    keccak.update(&vdf_output);
    Ok(keccak.finalize_with_penalty(penalty))
}

#[cfg(test)]
mod tests {
    use super::prime;
    use crate::expansion::{INPUT_HASH_SIZE, NONCE_SIZE};

    #[test]
    fn keccak_prime_test() {
        let prev_hash = [1u8; INPUT_HASH_SIZE];
        let root_hash = [2u8; INPUT_HASH_SIZE];
        let nonce = [3u8; NONCE_SIZE];

        dbg!(prime(prev_hash, root_hash, nonce, 100, 100, 10)
            .expect("Failed to execute Keccak-prime"));
    }
}