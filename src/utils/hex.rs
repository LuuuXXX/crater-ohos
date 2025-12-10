pub fn encode<T: AsRef<[u8]>>(data: T) -> String {
    data.as_ref()
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect()
}

pub fn decode(s: &str) -> Result<Vec<u8>, anyhow::Error> {
    if !s.len().is_multiple_of(2) {
        anyhow::bail!("hex string has odd length");
    }

    (0..s.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&s[i..i + 2], 16)
                .map_err(|e| anyhow::anyhow!("invalid hex string: {}", e))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode() {
        assert_eq!(encode(b"hello"), "68656c6c6f");
        assert_eq!(encode(b""), "");
        assert_eq!(encode(&[0xff, 0x00, 0xaa]), "ff00aa");
    }

    #[test]
    fn test_decode() {
        assert_eq!(decode("68656c6c6f").unwrap(), b"hello");
        assert_eq!(decode("").unwrap(), b"");
        assert_eq!(decode("ff00aa").unwrap(), vec![0xff, 0x00, 0xaa]);
    }

    #[test]
    fn test_decode_invalid() {
        assert!(decode("abc").is_err()); // odd length
        assert!(decode("zz").is_err()); // invalid hex
    }
}
