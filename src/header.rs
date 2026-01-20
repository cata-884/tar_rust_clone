//TAR spec requires the archive to end with 2 empty 512 byte blocks
pub const BLOCK_SIZE: u64 = 512;

pub const NAME_OFFSET: usize = 0;
pub const NAME_LEN: usize = 100;

pub const MODE_OFFSET: usize = 100;
pub const MODE_LEN: usize = 8;

pub const UID_OFFSET: usize = 108;
pub const UID_LEN: usize = 8;

pub const GID_OFFSET: usize = 116;
pub const GID_LEN: usize = 8;

pub const SIZE_OFFSET: usize = 124;
pub const SIZE_LEN: usize = 12;

pub const MTIME_OFFSET: usize = 136;
pub const MTIME_LEN: usize = 12;

pub const CHKSUM_OFFSET: usize = 148;
pub const CHKSUM_LEN: usize = 8;

pub const TYPEFLAG_OFFSET: usize = 156;

pub const MAGIC_OFFSET: usize = 257;
pub const MAGIC_LEN: usize = 6;

pub const VERSION_OFFSET: usize = 263;
pub const VERSION_LEN: usize = 2;

pub const MAGIC_VALUE: &[u8; 6] = b"ustar\0";
pub const VERSION_VALUE: &[u8; 2] = b"00";
pub const EMPTY_SPACE: u8 = b' ';

fn write_oct(slice: &mut [u8], value: u64) {
    let len = slice.len() - 1;
    let s = format!("{:0width$o}", value, width = len);
    let bytes = s.as_bytes();

    let start = bytes.len().saturating_sub(len);

    slice[0..len].copy_from_slice(&bytes[start..]);
    slice[len] = 0;
}

pub fn create_header(path: &str, size: u64) -> [u8; 512] {
    let mut header = [0u8; 512];

    // Name
    let name_bytes = path.as_bytes();
    let name_copy_len = name_bytes.len().min(NAME_LEN - 1);
    header[NAME_OFFSET..NAME_OFFSET + name_copy_len].copy_from_slice(&name_bytes[0..name_copy_len]);

    // Mode (Permissions) - Standard 644 (rw-r--r--)
    write_oct(&mut header[MODE_OFFSET..MODE_OFFSET + MODE_LEN], 0o644);

    // UID & GID (Zero/Root for simplicity)
    write_oct(&mut header[UID_OFFSET..UID_OFFSET + UID_LEN], 0);
    write_oct(&mut header[GID_OFFSET..GID_OFFSET + GID_LEN], 0);

    // File Size
    write_oct(&mut header[SIZE_OFFSET..SIZE_OFFSET + SIZE_LEN], size);

    // Modification Time (MTime) - 0 for now (or use SystemTime in future)
    write_oct(&mut header[MTIME_OFFSET..MTIME_OFFSET + MTIME_LEN], 0);

    // Typeflag (0 = Normal File, 5 = Directory)
    // The caller logic handles directories separately usually,
    // but for this generic header, we default to '0' (file).
    header[TYPEFLAG_OFFSET] = b'0';

    //Magic & Version (ustar indicator)
    header[MAGIC_OFFSET..MAGIC_OFFSET + MAGIC_LEN].copy_from_slice(MAGIC_VALUE);
    header[VERSION_OFFSET..VERSION_OFFSET + VERSION_LEN].copy_from_slice(VERSION_VALUE);

    // Checksum Calculation
    // The checksum field is treated as if it were filled with spaces during calculation.
    let chksum_range = &mut header[CHKSUM_OFFSET..CHKSUM_OFFSET + CHKSUM_LEN];
    chksum_range.fill(EMPTY_SPACE);

    let mut sum: u32 = 0;
    for byte in &header {
        sum += *byte as u32;
    }

    // Write the calculated checksum back into the header
    // The checksum is 6 digits of octal followed by a null and a space usually,
    // or just nulls.
    let sum_str = format!("{:06o}\0", sum);
    header[CHKSUM_OFFSET..CHKSUM_OFFSET + sum_str.len()].copy_from_slice(sum_str.as_bytes());

    header
}
