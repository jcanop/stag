use xxhash_rust::xxh32::xxh32;

const SEED: u32 = 42;

pub fn hash(data: &[u8]) -> u32 {
    xxh32(data, SEED)
}

pub fn validate(data: &[u8], h: u32) -> bool {
    hash(data) == h
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let h = hash(b"Test");
        assert!(validate(b"Test", h));
        assert!(!validate(b"TEST", h));
    }
}
