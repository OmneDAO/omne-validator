//! Utility functions and helpers


use std::path::PathBuf;

/// Expand tilde (~) in file paths to home directory
pub fn expand_tilde(path: &PathBuf) -> PathBuf {
    if path.starts_with("~") {
        if let Some(home) = dirs::home_dir() {
            return home.join(path.strip_prefix("~").unwrap());
        }
    }
    path.clone()
}

/// Format duration in human-readable format
pub fn format_duration(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else if secs < 86400 {
        let hours = secs / 3600;
        let minutes = (secs % 3600) / 60;
        format!("{}h {}m", hours, minutes)
    } else {
        let days = secs / 86400;
        let hours = (secs % 86400) / 3600;
        format!("{}d {}h", days, hours)
    }
}

/// Format bytes in human-readable format
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit_index = 0;

    while value >= 1024.0 && unit_index < UNITS.len() - 1 {
        value /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", value, UNITS[unit_index])
    }
}

/// Validate Omne address format
pub fn validate_omne_address(address: &str) -> bool {
    // Validate Omne native address format (omne1...)
    if address.starts_with("omne1") {
        // Should be 43 characters total (omne1 + 38 characters)
        if address.len() != 43 {
            return false;
        }
        
        // Validate base32-like encoding (excluding 0, O, I, L)
        const OMNE_ALPHABET: &str = "123456789abcdefghjkmnpqrstuvwxyz";
        return address[5..].chars().all(|c| OMNE_ALPHABET.contains(c));
    }
    
    // Legacy hex address validation for backward compatibility
    if address.starts_with("0x") {
        if address.len() != 42 {
            return false;
        }
        return address[2..].chars().all(|c| c.is_ascii_hexdigit());
    }
    
    false
}

/// Generate a secure random string
pub fn generate_random_string(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Calculate uptime percentage
pub fn calculate_uptime(total_time: std::time::Duration, downtime: std::time::Duration) -> f64 {
    if total_time.is_zero() {
        return 100.0;
    }
    
    let uptime = total_time.saturating_sub(downtime);
    (uptime.as_secs_f64() / total_time.as_secs_f64()) * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(std::time::Duration::from_secs(30)), "30s");
        assert_eq!(format_duration(std::time::Duration::from_secs(90)), "1m 30s");
        assert_eq!(format_duration(std::time::Duration::from_secs(3661)), "1h 1m");
        assert_eq!(format_duration(std::time::Duration::from_secs(90061)), "1d 1h");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
    }

    #[test]
    fn test_validate_omne_address() {
        assert!(validate_omne_address("0x1234567890abcdef1234567890abcdef12345678"));
        assert!(!validate_omne_address("1234567890abcdef1234567890abcdef12345678"));
        assert!(!validate_omne_address("0x1234567890abcdef1234567890abcdef123456"));
        assert!(!validate_omne_address("0x1234567890abcdef1234567890abcdef1234567g"));
    }

    #[test]
    fn test_calculate_uptime() {
        let total = std::time::Duration::from_secs(3600); // 1 hour
        let downtime = std::time::Duration::from_secs(360); // 6 minutes
        assert_eq!(calculate_uptime(total, downtime), 90.0);
        
        assert_eq!(calculate_uptime(std::time::Duration::ZERO, downtime), 100.0);
    }
}
