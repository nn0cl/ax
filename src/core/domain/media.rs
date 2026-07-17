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

/// Environment-oriented pack budget presets for `-s` / `--size` / `target_size`.
///
/// Accepts keywords (case-insensitive) in addition to numeric sizes (`10M`, `4G`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizePreset {
    /// 10 MiB units — low bandwidth / resume-friendly fine chunks.
    /// Primary keyword: `c2` (short); also `low-net`, `slow`.
    C2,
    /// 3.8 GiB units — stay under FAT32 single-file 4 GiB limit with margin.
    Fat32,
    /// 1 GiB units — balanced for typical flash / USB random-write behavior.
    Flash,
    /// Effectively no split (very large budget) — fast local / high-bandwidth.
    Max,
}

impl SizePreset {
    pub fn parse(name: &str) -> Option<Self> {
        match name.trim().to_ascii_lowercase().replace('_', "-").as_str() {
            // short primary: c2; longer aliases kept for clarity
            "c2" | "low-net" | "lownet" | "slow" => Some(Self::C2),
            "fat32" | "fat" | "usb-old" => Some(Self::Fat32),
            "flash" | "usb" | "stick" => Some(Self::Flash),
            "max" | "nosplit" | "no-split" | "single" => Some(Self::Max),
            _ => None,
        }
    }

    /// Budget bytes (before connection derating / mapping reserve).
    pub fn bytes(self) -> u64 {
        const MIB: u64 = 1024 * 1024;
        const GIB: u64 = 1024 * MIB;
        match self {
            Self::C2 => 10 * MIB,
            // 3.8 GiB < 4 GiB FAT32 max file size (with FS/tooling margin).
            Self::Fat32 => (3.8 * GIB as f64).floor() as u64,
            Self::Flash => GIB,
            // 16 TiB — practically one archive for normal trees.
            Self::Max => 16 * 1024 * GIB,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::C2 => "c2",
            Self::Fat32 => "fat32",
            Self::Flash => "flash",
            Self::Max => "max",
        }
    }
}

/// Parse human sizes **or** environment presets.
///
/// Numeric: `123`, `700M`, `4.7G`, `1Gi` (binary K/M/G/T).  
/// Presets: `c2` (10MiB), `fat32` (3.8GiB), `flash` (1GiB), `max` (no practical split).
pub fn parse_size(input: &str) -> Result<u64, ArchiveError> {
    let s = input.trim();
    if s.is_empty() {
        return Err(ArchiveError::InvalidSize(input.to_string()));
    }

    if let Some(preset) = SizePreset::parse(s) {
        return Ok(preset.bytes());
    }

    let (num_part, suffix) = split_size_suffix(s);
    if num_part.is_empty() {
        return Err(ArchiveError::InvalidSize(format!(
            "unknown size or preset {input:?} (try 10M, 4G, c2, fat32, flash, max)"
        )));
    }
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
        _ => {
            return Err(ArchiveError::InvalidSize(format!(
                "unknown size or preset {input:?} (try 10M, 4G, c2, fat32, flash, max)"
            )))
        }
    };

    let bytes = value * mult;
    if bytes > u64::MAX as f64 {
        return Err(ArchiveError::InvalidSize(input.to_string()));
    }
    Ok(bytes.floor() as u64)
}

/// Like [`parse_size`], but returns which preset was used (if any) for logging.
pub fn parse_size_with_preset(input: &str) -> Result<(u64, Option<SizePreset>), ArchiveError> {
    let s = input.trim();
    if let Some(p) = SizePreset::parse(s) {
        return Ok((p.bytes(), Some(p)));
    }
    Ok((parse_size(s)?, None))
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

    #[test]
    fn size_presets() {
        const MIB: u64 = 1024 * 1024;
        const GIB: u64 = 1024 * MIB;
        assert_eq!(parse_size("c2").unwrap(), 10 * MIB);
        assert_eq!(parse_size("low-net").unwrap(), 10 * MIB); // alias
        assert_eq!(parse_size("LOW_NET").unwrap(), 10 * MIB);
        assert_eq!(parse_size("fat32").unwrap(), (3.8 * GIB as f64).floor() as u64);
        assert!(parse_size("fat32").unwrap() < 4 * GIB);
        assert_eq!(parse_size("flash").unwrap(), GIB);
        assert_eq!(parse_size("max").unwrap(), 16 * 1024 * GIB);
        let (b, p) = parse_size_with_preset("c2").unwrap();
        assert_eq!(b, 10 * MIB);
        assert_eq!(p, Some(SizePreset::C2));
        assert_eq!(SizePreset::C2.as_str(), "c2");
        assert!(parse_size("not-a-size").is_err());
    }
}
