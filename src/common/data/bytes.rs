use serde::{Deserialize, Serialize};

use std::fmt::{Debug, Error, Formatter, LowerHex};

use talk::crypto::primitives::hash::{Hash, HASH_LENGTH};

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub(crate) struct Bytes(pub [u8; HASH_LENGTH]);

impl From<Hash> for Bytes {
    fn from(digest: Hash) -> Bytes {
        Bytes(digest.to_bytes())
    }
}

impl From<Bytes> for Hash {
    fn from(val: Bytes) -> Self {
        Hash::from_bytes(val.0)
    }
}

impl LowerHex for Bytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        for byte in &self.0 {
            write!(f, "{:x}", byte)?;
        }

        Ok(())
    }
}

impl Debug for Bytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Bytes({:x})", self)?;
        Ok(())
    }
}

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}