use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Size {
    #[serde(alias = "B")]
    Bytes(u64),
    #[serde(alias = "K")]
    Kilobytes(u64),
    #[serde(alias = "M")]
    Megabytes(u64),
    #[serde(alias = "G")]
    Gigabytes(u64),
}

impl Size {
    pub fn to_bytes(&self) -> u64 {
        match self {
            Size::Bytes(n) => *n,
            Size::Kilobytes(n) => n * 1024,
            Size::Megabytes(n) => n * 1024 * 1024,
            Size::Gigabytes(n) => n * 1024 * 1024 * 1024,
        }
    }
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Size::Bytes(n) => write!(f, "{}B", n),
            Size::Kilobytes(n) => write!(f, "{}K", n),
            Size::Megabytes(n) => write!(f, "{}M", n),
            Size::Gigabytes(n) => write!(f, "{}G", n),
        }
    }
}

impl std::str::FromStr for Size {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            anyhow::bail!("empty size string");
        }

        let (num_str, suffix) = if s.ends_with('B') || s.ends_with('b') {
            (&s[..s.len() - 1], "B")
        } else if s.ends_with('K') || s.ends_with('k') {
            (&s[..s.len() - 1], "K")
        } else if s.ends_with('M') || s.ends_with('m') {
            (&s[..s.len() - 1], "M")
        } else if s.ends_with('G') || s.ends_with('g') {
            (&s[..s.len() - 1], "G")
        } else {
            (s, "B")
        };

        let num: u64 = num_str
            .parse()
            .map_err(|_| anyhow::anyhow!("invalid number in size: {}", s))?;

        Ok(match suffix {
            "B" => Size::Bytes(num),
            "K" => Size::Kilobytes(num),
            "M" => Size::Megabytes(num),
            "G" => Size::Gigabytes(num),
            _ => unreachable!(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_to_bytes() {
        assert_eq!(Size::Bytes(100).to_bytes(), 100);
        assert_eq!(Size::Kilobytes(2).to_bytes(), 2048);
        assert_eq!(Size::Megabytes(1).to_bytes(), 1048576);
        assert_eq!(Size::Gigabytes(1).to_bytes(), 1073741824);
    }

    #[test]
    fn test_size_from_str() {
        assert_eq!("100".parse::<Size>().unwrap(), Size::Bytes(100));
        assert_eq!("2K".parse::<Size>().unwrap(), Size::Kilobytes(2));
        assert_eq!("2k".parse::<Size>().unwrap(), Size::Kilobytes(2));
        assert_eq!("1M".parse::<Size>().unwrap(), Size::Megabytes(1));
        assert_eq!("1G".parse::<Size>().unwrap(), Size::Gigabytes(1));
    }
}
