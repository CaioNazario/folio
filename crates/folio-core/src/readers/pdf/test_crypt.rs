//! Test-only: the standard security handler, revision 2 (RC4 with a
//! 40-bit key, PDF 32000 7.6.3), enough to write PDFs that need a
//! password.

use md5::{Digest, Md5};

/// Pads passwords to 32 bytes (PDF 32000, 7.6.3.3).
const PADDING: [u8; 32] = [
    0x28, 0xBF, 0x4E, 0x5E, 0x4E, 0x75, 0x8A, 0x41, 0x64, 0x00, 0x4E, 0x56, 0xFF, 0xFA, 0x01, 0x08,
    0x2E, 0x2E, 0x00, 0xB6, 0xD0, 0x68, 0x3E, 0x80, 0x2F, 0x0C, 0xA9, 0xFE, 0x64, 0x53, 0x69, 0x7A,
];
/// The first entry of the file identifier.
const FILE_ID: &[u8; 16] = b"folio-test-file!";
/// Every permission granted.
const PERMISSIONS: i32 = -4;
/// Revision 2 keys are 40 bits.
const KEY_LEN: usize = 5;

/// The keys and entries of a PDF encrypted with a user and an owner
/// password.
pub struct Security {
    key: Vec<u8>,
    owner_entry: Vec<u8>,
}

impl Security {
    pub fn new(user: &str, owner: &str) -> Self {
        let mut owner_entry = padded(user).to_vec();
        rc4(&md5(&[&padded(owner)])[..KEY_LEN], &mut owner_entry);
        let permissions = PERMISSIONS.to_le_bytes();
        let key = md5(&[&padded(user), &owner_entry, &permissions, FILE_ID])[..KEY_LEN].to_vec();
        Self { key, owner_entry }
    }

    /// The key that encrypts the strings and streams of object `number`.
    pub fn object_key(&self, number: usize) -> Vec<u8> {
        let number = u32::try_from(number).unwrap().to_le_bytes();
        md5(&[&self.key, &number[..3], &[0, 0]])[..KEY_LEN + 5].to_vec()
    }

    /// The encryption dictionary.
    pub fn dict(&self) -> String {
        let mut user_entry = PADDING.to_vec();
        rc4(&self.key, &mut user_entry);
        let (o, u) = (hex(&self.owner_entry), hex(&user_entry));
        format!("<< /Filter /Standard /V 1 /R 2 /Length 40 /O <{o}> /U <{u}> /P {PERMISSIONS} >>")
    }

    /// The entries the trailer needs, with the encryption dictionary at
    /// object `number`.
    pub fn trailer(number: usize) -> String {
        format!("/Encrypt {number} 0 R /ID [<{0}> <{0}>]", hex(FILE_ID))
    }
}

/// Encrypts or decrypts `data` in place with RC4.
pub fn rc4(key: &[u8], data: &mut [u8]) {
    let mut state: Vec<u8> = (0..=255).collect();
    let mut j = 0u8;
    for i in 0..256 {
        j = j.wrapping_add(state[i]).wrapping_add(key[i % key.len()]);
        state.swap(i, usize::from(j));
    }
    let (mut i, mut j) = (0u8, 0u8);
    for byte in data {
        i = i.wrapping_add(1);
        j = j.wrapping_add(state[usize::from(i)]);
        state.swap(usize::from(i), usize::from(j));
        *byte ^= state[usize::from(state[usize::from(i)].wrapping_add(state[usize::from(j)]))];
    }
}

/// `password` cut or padded to 32 bytes.
fn padded(password: &str) -> [u8; 32] {
    let mut padded = PADDING;
    let bytes = &password.as_bytes()[..password.len().min(32)];
    padded[..bytes.len()].copy_from_slice(bytes);
    padded[bytes.len()..].copy_from_slice(&PADDING[..32 - bytes.len()]);
    padded
}

fn md5(parts: &[&[u8]]) -> Vec<u8> {
    let mut hasher = Md5::new();
    for part in parts {
        hasher.update(part);
    }
    hasher.finalize().to_vec()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02X}")).collect()
}
