use super::error::ArchiveError;

/// Known media capacity presets (base logical budget before connection factor).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaKind {
    Cd,
    Dvd,
    Bd25,
    Custom(u64),
}

impl MediaKind {
    pub fn parse(name: &str) -> Result<Self, ArchiveError> {
        match name.to_ascii_lowercase().as_str() {
            "cd" => Ok(Self::Cd),
            "dvd" => Ok(Self::Dvd),
            "bd25" | "bluray" | "bd" => Ok(Self::Bd25),
            _ => Err(ArchiveError::InvalidMedia(name.to_string())),
        }
    }

    pub fn capacity_bytes(self) -> u64 {
        const MIB: u64 = 1024 * 1024;
        match self {
            Self::Cd => 700 * MIB,
            Self::Dvd => 4482 * MIB,
            Self::Bd25 => 23866 * MIB,
            Self::Custom(n) => n,
        }
    }
}

/// How the destination is attached; used to derate the effective budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionKind {
    #[default]
    Local,
    Usb2,
    Usb3,
    Thunderbolt,
    Network,
}

impl ConnectionKind {
    pub fn parse(name: &str) -> Result<Self, ArchiveError> {
        match name.to_ascii_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "usb2" => Ok(Self::Usb2),
            "usb3" => Ok(Self::Usb3),
            "thunderbolt" | "tb" => Ok(Self::Thunderbolt),
            "network" | "net" | "smb" | "nfs" => Ok(Self::Network),
            _ => Err(ArchiveError::InvalidConnection(name.to_string())),
        }
    }

    /// Safety / overhead factor applied to media capacity.
    pub fn factor(self) -> f64 {
        match self {
            Self::Local => 1.00,
            Self::Thunderbolt => 0.99,
            Self::Usb3 => 0.98,
            Self::Usb2 => 0.95,
            Self::Network => 0.90,
        }
    }
}

/// Compute effective pack budget from media capacity and connection.
pub fn effective_budget(media: MediaKind, connection: ConnectionKind) -> u64 {
    let base = media.capacity_bytes() as f64;
    let scaled = (base * connection.factor()).floor() as u64;
    let reserve = mapping_reserve(scaled);
    scaled.saturating_sub(reserve).max(1)
}

/// Bytes reserved for mapping / FS overhead when deriving a usable budget.
pub fn mapping_reserve(budget: u64) -> u64 {
    const MIB: u64 = 1024 * 1024;
    let one_percent = budget / 100;
    one_percent.min(MIB).max(if budget > 4096 { 4096 } else { 0 })
}

/// Parse human sizes: `123`, `700M`, `4.7G`, `1Gi` (binary powers for K/M/G/T).
pub fn parse_size(input: &str) -> Result<u64, ArchiveError> {
    let s = input.trim();
    if s.is_empty() {
        return Err(ArchiveError::InvalidSize(input.to_string()));
    }

    let (num_part, suffix) = split_size_suffix(s);
    let value: f64 = num_part
        .parse()
        .map_err(|_| ArchiveError::InvalidSize(input.to_string()))?;
    if value < 0.0 || !value.is_finite() {
        return Err(ArchiveError::InvalidSize(input.to_string()));
    }

    let mult = match suffix.as_str() {
        "" => 1.0_f64,
        "k" | "kb" | "ki" | "kib" => 1024.0,
        "m" | "mb" | "mi" | "mib" => 1024.0 * 1024.0,
        "g" | "gb" | "gi" | "gib" => 1024.0 * 1024.0 * 1024.0,
        "t" | "tb" | "ti" | "tib" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => return Err(ArchiveError::InvalidSize(input.to_string())),
    };

    let bytes = value * mult;
    if bytes > u64::MAX as f64 {
        return Err(ArchiveError::InvalidSize(input.to_string()));
    }
    Ok(bytes.floor() as u64)
}

fn split_size_suffix(s: &str) -> (&str, String) {
    let bytes = s.as_bytes();
    let mut i = bytes.len();
    while i > 0 && bytes[i - 1].is_ascii_alphabetic() {
        i -= 1;
    }
    (&s[..i], s[i..].to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_human_sizes() {
        assert_eq!(parse_size("1024").unwrap(), 1024);
        assert_eq!(parse_size("1K").unwrap(), 1024);
        assert_eq!(parse_size("700M").unwrap(), 700 * 1024 * 1024);
        assert_eq!(parse_size("1G").unwrap(), 1024 * 1024 * 1024);
    }

    #[test]
    fn connection_derates_budget() {
        let local = effective_budget(MediaKind::Custom(10_000), ConnectionKind::Local);
        let net = effective_budget(MediaKind::Custom(10_000), ConnectionKind::Network);
        assert!(net < local);
        assert!(local > 0 && net > 0);
    }
}
