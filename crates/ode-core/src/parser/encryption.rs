use md5::{Md5, Digest};

/// Standard PDF padding string (32 bytes) per PDF spec Table 3.19
const PDF_PADDING: [u8; 32] = [
    0x28, 0xBF, 0x4E, 0x5E, 0x4E, 0x75, 0x8A, 0x41,
    0x64, 0x00, 0x4E, 0x56, 0xFF, 0xFA, 0x01, 0x08,
    0x2E, 0x2E, 0x00, 0xB6, 0xD0, 0x68, 0x3E, 0x80,
    0x2F, 0x0C, 0xA9, 0xFE, 0x64, 0x53, 0x69, 0x7A,
];

/// Compute the file encryption key (Algorithm 2 from PDF spec 7.6.3.3).
pub fn compute_encryption_key(
    password: &[u8],
    o_value: &[u8],
    p_value: i32,
    file_id: &[u8],
    key_length: usize,
    revision: u32,
) -> Vec<u8> {
    // Step 1: Pad password to 32 bytes
    let mut padded = [0u8; 32];
    let copy_len = password.len().min(32);
    padded[..copy_len].copy_from_slice(&password[..copy_len]);
    if copy_len < 32 {
        padded[copy_len..].copy_from_slice(&PDF_PADDING[..32 - copy_len]);
    }

    // Steps 2-6: MD5(padded_password + /O + /P_le32 + /ID[0])
    let mut hasher = Md5::new();
    hasher.update(&padded);
    hasher.update(o_value);
    hasher.update(&(p_value as u32).to_le_bytes());
    hasher.update(file_id);
    let mut hash = hasher.finalize().to_vec();

    // Step 8: For R >= 3, iterate MD5 50 times on the first key_length bytes
    if revision >= 3 {
        for _ in 0..50 {
            let mut h = Md5::new();
            h.update(&hash[..key_length]);
            hash = h.finalize().to_vec();
        }
    }

    hash[..key_length].to_vec()
}

/// Compute the per-object encryption key (Algorithm 1 from PDF spec 7.6.3.4).
pub fn compute_object_key(
    encryption_key: &[u8],
    obj_id: u64,
    gen: u16,
) -> Vec<u8> {
    let mut data = Vec::with_capacity(encryption_key.len() + 5);
    data.extend_from_slice(encryption_key);
    // Append 3 low-order bytes of object number (little-endian)
    data.push((obj_id & 0xFF) as u8);
    data.push(((obj_id >> 8) & 0xFF) as u8);
    data.push(((obj_id >> 16) & 0xFF) as u8);
    // Append 2 low-order bytes of generation number (little-endian)
    data.push((gen & 0xFF) as u8);
    data.push(((gen >> 8) & 0xFF) as u8);

    let hash = Md5::digest(&data);
    let key_len = (encryption_key.len() + 5).min(16);
    hash[..key_len].to_vec()
}

/// RC4 encrypt/decrypt (symmetric — same function for both directions).
pub fn rc4_crypt(key: &[u8], data: &[u8]) -> Vec<u8> {
    if key.is_empty() || data.is_empty() {
        return data.to_vec();
    }

    // Key Scheduling Algorithm (KSA)
    let mut s: [u8; 256] = [0; 256];
    for i in 0..256 {
        s[i] = i as u8;
    }
    let mut j: usize = 0;
    for i in 0..256 {
        j = (j + s[i] as usize + key[i % key.len()] as usize) % 256;
        s.swap(i, j);
    }

    // Pseudo-Random Generation Algorithm (PRGA)
    let mut result = Vec::with_capacity(data.len());
    let mut i: usize = 0;
    let mut j: usize = 0;
    for &byte in data {
        i = (i + 1) % 256;
        j = (j + s[i] as usize) % 256;
        s.swap(i, j);
        let k = s[(s[i] as usize + s[j] as usize) % 256];
        result.push(byte ^ k);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rc4_roundtrip() {
        let key = b"testkey";
        let plaintext = b"Hello, World!";
        let encrypted = rc4_crypt(key, plaintext);
        assert_ne!(&encrypted, plaintext);
        let decrypted = rc4_crypt(key, &encrypted);
        assert_eq!(&decrypted, plaintext);
    }

    #[test]
    fn test_rc4_known_vector() {
        // RC4 test vector: key="Key", plaintext="Plaintext"
        let key = b"Key";
        let plaintext = b"Plaintext";
        let encrypted = rc4_crypt(key, plaintext);
        // Known RC4 output for this key/plaintext pair
        let expected = [0xBB, 0xF3, 0x16, 0xE8, 0xD9, 0x40, 0xAF, 0x0A, 0xD3];
        assert_eq!(encrypted, expected);
    }

    #[test]
    fn test_empty_password_key_derivation() {
        // With an empty password, the padding bytes should be used
        let o_value = vec![0u8; 32];
        let p_value = -1;
        let file_id = vec![0u8; 16];
        let key = compute_encryption_key(b"", &o_value, p_value, &file_id, 5, 2);
        assert_eq!(key.len(), 5);
    }

    #[test]
    fn test_object_key_length() {
        let enc_key = vec![0x01, 0x02, 0x03, 0x04, 0x05];
        let obj_key = compute_object_key(&enc_key, 10, 0);
        // key_length = min(5 + 5, 16) = 10
        assert_eq!(obj_key.len(), 10);
    }
}
