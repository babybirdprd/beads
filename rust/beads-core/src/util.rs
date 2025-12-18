use sha2::{Digest, Sha256};
use chrono::{DateTime, Utc};

const BASE36_ALPHABET: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";

fn encode_base36(data: &[u8], length: usize) -> String {
    // Interpret data as big-endian integer.
    // We assume data fits in u128 (16 bytes).
    // Go implementation handles arbitrary length, but passes hash[:numBytes].
    // For length=6, numBytes=4 (32 bits). u128 is plenty.

    let mut num = 0u128;
    for &b in data {
        num = (num << 8) | (b as u128);
    }

    let mut chars = Vec::new();
    let base = 36u128;

    // Handle 0 explicitly? loop handles it if num starts at 0, loop doesn't run, padding adds '0's. Correct.

    while num > 0 {
        let rem = (num % base) as usize;
        num /= base;
        chars.push(BASE36_ALPHABET[rem] as char);
    }

    // Reverse (since we extracted LSD first)
    chars.reverse();

    // Pad with '0' to the left (prefix)
    // Go: "if len(str) < length { str = strings.Repeat("0", length-len(str)) + str }"
    // So "000abc"

    let mut padded_chars = Vec::new();
    let needed = if length > chars.len() { length - chars.len() } else { 0 };
    for _ in 0..needed {
        padded_chars.push('0');
    }
    padded_chars.extend(chars);

    // Truncate to exact length if needed (keep least significant digits / suffix)
    // Go: "if len(str) > length { str = str[len(str)-length:] }"
    // So "123abc" (length 3) -> "abc"

    if padded_chars.len() > length {
        let start = padded_chars.len() - length;
        padded_chars[start..].iter().collect()
    } else {
        padded_chars.into_iter().collect()
    }
}

pub fn generate_hash_id(title: &str, description: &str, created_at: DateTime<Utc>, creator: &str) -> String {
    // Go: content := fmt.Sprintf("%s|%s|%s|%d|%d", title, description, creator, timestamp.UnixNano(), nonce)
    // We use nonce = 0
    // We use length = 6 (default fallback in Go)

    let nonce = 0;
    let length = 6;
    let timestamp_nano = created_at.timestamp_nanos_opt().unwrap_or(0);

    let content = format!("{}|{}|{}|{}|{}", title, description, creator, timestamp_nano, nonce);

    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize(); // [u8; 32]

    // Go logic for numBytes:
    // case 6: numBytes = 4
    let num_bytes = 4;

    let hash_slice = &result[..num_bytes];
    encode_base36(hash_slice, length)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_encode_base36() {
        // Test 0 -> "000000"
        assert_eq!(encode_base36(&[0], 6), "000000");

        // Test max u32 -> "1z141z3" -> truncate to 6 -> "z141z3"
        // u32::MAX = 4294967295
        // 4294967295 in base 36 is 1z141z3
        let bytes = u32::MAX.to_be_bytes();
        assert_eq!(encode_base36(&bytes, 6), "z141z3");

        // Test known value
        // 123456 -> "002n9c"
        // 2*36^3 + 23*36^2 + 9*36 + 12 = 93312 + 29808 + 324 + 12 = 123456
        let val = 123456u32;
        assert_eq!(encode_base36(&val.to_be_bytes(), 6), "002n9c");
    }

    #[test]
    fn test_generate_hash_id_stability() {
        // Ensure deterministic output for same input
        let date = Utc.timestamp_opt(1600000000, 0).unwrap();
        let id1 = generate_hash_id("Title", "Desc", date, "User");
        let id2 = generate_hash_id("Title", "Desc", date, "User");
        assert_eq!(id1, id2);
        assert_eq!(id1.len(), 6);

        // Ensure different output for different input
        let id3 = generate_hash_id("Title2", "Desc", date, "User");
        assert_ne!(id1, id3);
    }
}
