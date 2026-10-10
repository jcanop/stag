///  Embed Format:
///
///  00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F
/// +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+  --+--
/// |     VERSION MAJOR     |     VERSION MINOR     |    |
/// +-----------------------+-----------------------+    |
/// |       HASH ALG        |         ENC ALG       |    |
/// +-----------------------+-----------------------+    |  HEADER
/// |                  DATA LENGTH                  |    |
/// |                                               |    |
/// +-----------------------+-----------------------+  --+--
/// |                                               |
/// /                      HASH                     /
/// |                                               |
/// +-----------------------+-----------------------+
/// |                                               |
/// /                      DATA                     /
/// |                                               |
/// +-----------------------+-----------------------+

use crate::crypto::{ self, EncryptionAlgorithm, HashAlgorithm };
use std::fmt::{ self, Display };
use thiserror::Error;

// --- Constants ---
const HEADER_LEN: usize = 2 + 1 + 1 + 4;

/// Enumeration of the parsing errors.
#[derive(Error, Debug, PartialEq)]
pub enum ParsingError {
    #[error("Missing data")]
    MissingData,
    #[error("Incorrect hash")]
    IncorrectHash,
    #[error("Unsupported version: `{0}`")]
    UnsupportedVersion(String),
    #[error("Unknown encryption algorithm")]
    UnknownEncryptionAlgorithmError(#[from] crypto::UnknownEncryptionAlgorithmError),
    #[error("Unknown hash algorithm")]
    UnknownHashAlgorithmError(#[from] crypto::UnknownHashAlgorithmError),
}

/// Enumeration of the wire errors.
#[derive(Error, Debug, PartialEq)]
pub enum WireError {
    #[error("Input data is too big")]
    InputTooBig,
    #[error("Parsing error")]
    Parsing(#[from] ParsingError),
}

/// Version of the embed format.
#[derive(Debug, Clone, PartialEq)]
pub struct Version {
    major: u8,
    minor: u8,
}

impl Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

impl From<&[u8;2]> for Version {
    fn from(item: &[u8;2]) -> Self {
        Self { major: item[0], minor: item[1] }
    }
}

impl From<Version> for [u8;2] {
    fn from(item: Version) -> Self {
        let a: [u8; 2] = [item.major, item.minor];
        a
    }
}

/// Header of the embed format.
#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    pub version: Version,
    pub hash_algorithm: HashAlgorithm,
    pub enc_algorithm: EncryptionAlgorithm,
    pub data_len: u32,
}

impl From<Header> for Vec<u8> {
    fn from(header: Header) -> Self {
        let mut v: Vec<u8> = Vec::with_capacity(HEADER_LEN);
        v.push(header.version.major);
        v.push(header.version.minor);
        v.push(header.hash_algorithm.into());
        v.push(header.enc_algorithm.into());
        v.extend_from_slice(&header.data_len.to_be_bytes());
        v
    }
}

impl TryFrom<&[u8; HEADER_LEN]> for Header {
    type Error = ParsingError;

    fn try_from(data: &[u8; HEADER_LEN]) -> Result<Self, Self::Error> {
        let version = Version { major: data[0], minor: data[1] };
        if version.major != 1 || version.minor != 0 {
            return Err(ParsingError::UnsupportedVersion(format!("{}", &version)));
        }
        let hash_algorithm: HashAlgorithm = data[2].try_into()?;
        let enc_algorithm: EncryptionAlgorithm = data[3].try_into()?;
        let bytes: [u8; 4] = data[4..].try_into().unwrap();
        let data_len = u32::from_be_bytes(bytes);
        Ok(Header { version, hash_algorithm, enc_algorithm, data_len })
    }
}

/// Embedded Data struct.
#[derive(Debug, Clone, PartialEq)]
pub struct EmbeddedData {
    pub header: Header,
    pub hash: Option<Vec<u8>>,
    pub data: Vec<u8>,
}

impl EmbeddedData {
    pub fn new(hash_algorithm: HashAlgorithm, enc_algorithm: EncryptionAlgorithm, data: Vec<u8>) -> Result<Self, WireError> {
        // --- Validate input length ---
        let data_len;
        if let Ok(n) = u32::try_from(data.len()) {
            data_len = n;
        } else {
            return Err(WireError::InputTooBig);
        }

        // --- Create the header ---
        let header = Header {
            version: Version { major: 1, minor: 0 },
            hash_algorithm,
            enc_algorithm,
            data_len,
        };

        // --- Calculate the hash ---
        let hash:Option<Vec<u8>> = match header.hash_algorithm {
            HashAlgorithm::None => None,
            HashAlgorithm::XXH32 => Some(crypto::xxh32::hash(&data).to_be_bytes().to_vec()),
        };

        // --- Result ---
        Ok(EmbeddedData { header, hash, data })
    }

    pub fn len(&self) -> usize {
        let hash_len = match self.hash {
            Some(ref e) => e.len(),
            None => 0,
        };
        HEADER_LEN + hash_len + self.data.len()
    }
}

impl From<EmbeddedData> for Vec<u8> {
    fn from(mut e: EmbeddedData) -> Self {
        let mut bytes: Vec<u8> = e.header.clone().into();
        if let Some(mut b) = e.hash {
            bytes.append(&mut b);
        }
        bytes.append(&mut e.data);
        bytes
    }
}

impl TryFrom<Vec<u8>> for EmbeddedData {
    type Error = WireError;

    fn try_from(mut v: Vec<u8>) -> Result<Self, Self::Error>  {
        if v.len() < HEADER_LEN {
            return Err(WireError::Parsing(ParsingError::MissingData));
        }
        let header_bytes: &[u8; HEADER_LEN] = &v[..HEADER_LEN].try_into().unwrap();
        let header: Header = header_bytes.try_into()?;
        let hash_len = header.hash_algorithm.len();
        if v.len() < HEADER_LEN + hash_len {
            return Err(WireError::Parsing(ParsingError::MissingData));
        }
        let hash: Option<Vec<u8>> = if hash_len == 0 {
            None
        } else {
            Some((&v[HEADER_LEN..HEADER_LEN + hash_len]).to_vec())
        };
        let i = HEADER_LEN + hash_len;
        let j = i + header.data_len as usize;
        let data: Vec<u8> = v.drain(i..j).collect();
        let h:Option<Vec<u8>> = match header.hash_algorithm {
            HashAlgorithm::None => None,
            HashAlgorithm::XXH32 => Some(crypto::xxh32::hash(&data).to_be_bytes().to_vec()),
        };
        if h != hash {
            return Err(WireError::Parsing(ParsingError::IncorrectHash));
        }
        Ok(EmbeddedData { header, hash, data })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        let version = Version { major: 1, minor: 5 };
        let bytes: [u8;2] = version.clone().into();
        let expected: [u8;2] = [1, 5];
        assert_eq!(expected, bytes);

        let parsed = Version::from(&bytes);
        assert_eq!(version, parsed);
    }

    #[test]
    fn test_header() {
        let header = Header {
            version: Version { major: 1, minor: 0 },
            hash_algorithm: HashAlgorithm::XXH32,
            enc_algorithm: EncryptionAlgorithm::Argon2,
            data_len: 42,
        };
        let expected: [u8;HEADER_LEN] = [ 1, 0, 1, 1, 0, 0, 0, 42];
        let bytes: Vec<u8> = header.clone().into();
        assert_eq!(expected, &bytes[..]);

        let parsed: Header = (&expected).try_into().unwrap();
        assert_eq!(header, parsed);

        let mut bytes = expected.clone();
        bytes[0] = 42;
        let result = Header::try_from(&bytes);
        assert!(matches!(result, Err(ParsingError::UnsupportedVersion(_))));
    }

    #[test]
    fn test_embedded_data_plain() {
        let hash_alg = HashAlgorithm::XXH32;
        let enc_alg = EncryptionAlgorithm::None;
        let data = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let hash = crypto::xxh32::hash(&data);
        let ed = EmbeddedData::new(hash_alg, enc_alg, data).unwrap();
        assert_eq!(Some(u32::to_be_bytes(hash).to_vec()), ed.hash);
        let bytes: Vec<u8> = ed.clone().into();

        let parsed = bytes.try_into().unwrap();
        assert_eq!(ed, parsed);
    }

    #[test]
    fn test_embedded_data_invalid_hash() {
        let hash_alg = HashAlgorithm::XXH32;
        let enc_alg = EncryptionAlgorithm::None;
        let data = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let ed = EmbeddedData::new(hash_alg, enc_alg, data).unwrap();
        let mut bytes: Vec<u8> = ed.clone().into();
        bytes[HEADER_LEN + hash_alg.len()] = 1;
        let result = TryInto::<EmbeddedData>::try_into(bytes);
        assert!(matches!(result, Err(WireError::Parsing(ParsingError::IncorrectHash))));
    }

    #[test]
    fn test_embedded_data_enc() {
        let hash_alg = HashAlgorithm::None;
        let enc_alg = EncryptionAlgorithm::Argon2;
        let data = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let ed = EmbeddedData::new(hash_alg, enc_alg, data).unwrap();
        let bytes: Vec<u8> = ed.clone().into();

        let parsed = bytes.try_into().unwrap();
        assert_eq!(ed, parsed);
    }

    #[test]
    fn test_embedded_data_limits() {
        let bytes = vec![1, 0, 0, 0, 0, 0, 0];
        let result = TryInto::<EmbeddedData>::try_into(bytes);
        assert!(matches!(result, Err(WireError::Parsing(ParsingError::MissingData))));
        let bytes = vec![2, 3, 0, 0, 0, 0, 0, 0];
        let result = TryInto::<EmbeddedData>::try_into(bytes);
        assert!(matches!(result, Err(WireError::Parsing(ParsingError::UnsupportedVersion(_)))));
        let bytes = vec![1, 0, 0, 0, 0, 0, 0, 0];
        let ed = TryInto::<EmbeddedData>::try_into(bytes).unwrap();
        assert_eq!(ed.hash, None);
        assert_eq!(ed.data, Vec::new());
    }
}
