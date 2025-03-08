use std::time::UNIX_EPOCH;

use anyhow::bail;
use ed25519_dalek::{Signature, VerifyingKey};

use crate::sign_algos::AlgoSign;

/// A representation of a proof of authenticity as is needed for retrieving and
/// deleting messages.
pub struct AuthProof {
    algorithm: AlgoSign,
    signature: Vec<u8>,
    pub pub_key: Vec<u8>,
    timestamp: u64,
    pub current_time: u64,
}

impl AuthProof {
    /// Constructs a new AuthProof from a buffer of bytes that could for example
    /// stem from a request body.
    pub fn new(buffer: &[u8]) -> Result<AuthProof, anyhow::Error> {
        let buf_len = buffer.len();
        if buf_len == 0 {
            bail!("Invalid Message")
        }
        let current_time = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut head = 0;

        let algorithm = match AlgoSign::from_repr(buffer[0]) {
            Some(algo) => algo,
            None => bail!(format!(
                "The specified signing algorithm is not supported. Currently supported are: \n{}\n",
                AlgoSign::list()
            )),
        };
        let expected_len = 1 + algorithm.get_key_len() + algorithm.get_signature_len() + 8;
        if buf_len != expected_len {
            bail!("Malformed message. Expected {expected_len} Bytes, but got {buf_len} Bytes.")
        }
        head += 1;
        let signature = buffer[head..=algorithm.get_signature_len()].to_vec();
        head += algorithm.get_signature_len();
        let pub_key = buffer[head..head + algorithm.get_key_len()].to_vec();
        head += algorithm.get_key_len();
        let mut time_bytes = [0u8; 8];
        time_bytes.copy_from_slice(&buffer[head..head + 8]);
        let timestamp = u64::from_be_bytes(time_bytes);
        Ok(Self {
            algorithm,
            signature,
            pub_key,
            timestamp,
            current_time,
        })
    }
    pub fn verify(&self) -> Result<bool, anyhow::Error> {
        match self.algorithm {
            AlgoSign::ED25519 => Ok(self.verify_ed25519()?),
        }
    }

    fn verify_ed25519(&self) -> Result<bool, anyhow::Error> {
        let mut encoded_pub_key = [0u8; 32];
        let mut encoded_sig = [0u8; 64];
        encoded_pub_key.copy_from_slice(self.pub_key.as_slice());
        encoded_sig.copy_from_slice(self.signature.as_slice());

        let message: Vec<u8> = [encoded_pub_key.as_slice(), &self.timestamp.to_be_bytes()].concat();
        let verifying_key = VerifyingKey::from_bytes(&encoded_pub_key)?;
        let signature = Signature::from_bytes(&encoded_sig);

        if self.timestamp > self.current_time {
            // allow timestamp to be upto five seconds in the future
            if self.timestamp - self.current_time > 5 {
                return Ok(false);
            }
        } else if self.current_time - self.timestamp > 5 {
            return Ok(false);
        }

        Ok(verifying_key
            .verify_strict(message.as_slice(), &signature)
            .is_ok())
    }
}
