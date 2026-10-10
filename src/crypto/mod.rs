pub mod argon2;
pub mod xxh32;

use std::error::Error;
use std::fmt::{ self, Display };

/// Error indicating an unknown hash algorithm.
///
/// Attributes:
/// 1. Algorithm ID code.
#[derive(Debug, PartialEq)]
pub struct UnknownHashAlgorithmError(u8);

impl Error for UnknownHashAlgorithmError {}

impl Display for UnknownHashAlgorithmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unknown hash algorithm: {}", self.0)
    }
}

/// Error indicating an unknown encryption algorithm
///
/// Attributes:
/// 1. Encyption ID code.
#[derive(Debug, PartialEq)]
pub struct UnknownEncryptionAlgorithmError(u8);

impl Display for UnknownEncryptionAlgorithmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unknown encryption algorithm: {}", self.0)
    }
}

impl Error for UnknownEncryptionAlgorithmError {}

/// Encryption algorithm enumeration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EncryptionAlgorithm {
    None,
    Argon2,
}

impl TryFrom<u8> for EncryptionAlgorithm {
    type Error = UnknownEncryptionAlgorithmError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Argon2),
            _ => Err(UnknownEncryptionAlgorithmError(value)),
        }
    }
}

impl From<EncryptionAlgorithm> for u8 {
    fn from(value: EncryptionAlgorithm) -> u8 {
        match value {
            EncryptionAlgorithm::None => 0,
            EncryptionAlgorithm::Argon2 => 1,
        }
    }
}

/// Hash algorithm enumeration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HashAlgorithm {
    None,
    XXH32,
}

impl HashAlgorithm {
    pub const fn len(&self) -> usize {
        match self {
            Self::None => 0,
            Self::XXH32 => 4,
        }
    }
}

impl TryFrom<u8> for HashAlgorithm {
    type Error = UnknownHashAlgorithmError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::XXH32),
            _ => Err(UnknownHashAlgorithmError(value)),
        }
    }
}

impl From<HashAlgorithm> for u8 {
    fn from(value: HashAlgorithm) -> u8 {
        match value {
            HashAlgorithm::None => 0,
            HashAlgorithm::XXH32 => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_algorithm() {
        let value = EncryptionAlgorithm::try_from(0);
        assert_eq!(Ok(EncryptionAlgorithm::None), value);
        let value = EncryptionAlgorithm::try_from(1);
        assert_eq!(Ok(EncryptionAlgorithm::Argon2), value);
        let value = EncryptionAlgorithm::try_from(3);
        assert_eq!(Err(UnknownEncryptionAlgorithmError(3)), value);

        assert_eq!(0u8, EncryptionAlgorithm::None.into());
        assert_eq!(1u8, EncryptionAlgorithm::Argon2.into());
    }

    #[test]
    fn test_hash_algorithm() {
        let value = HashAlgorithm::try_from(0);
        assert_eq!(Ok(HashAlgorithm::None), value);
        let value = HashAlgorithm::try_from(1);
        assert_eq!(Ok(HashAlgorithm::XXH32), value);
        let value = HashAlgorithm::try_from(2);
        assert_eq!(Err(UnknownHashAlgorithmError(2)), value);

        assert_eq!(0u8, HashAlgorithm::None.into());
        assert_eq!(1u8, HashAlgorithm::XXH32.into());
    }

    #[test]
    fn test_hash_algorithm_len() {
        assert_eq!(0, HashAlgorithm::None.len());
        assert_eq!(4, HashAlgorithm::XXH32.len());
    }
}
