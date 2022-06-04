use chrono::offset::Utc;
use chrono::DateTime;
use std::process;
use std::time::SystemTime;

pub fn u32_to_vecu8(number: &u32) -> [u8; 4] {
    let x1: u8 = ((number >> 24) & 0xff) as u8;
    let x2: u8 = ((number >> 16) & 0xff) as u8;
    let x3: u8 = ((number >> 8) & 0xff) as u8;
    let x4: u8 = (number & 0xff) as u8;
    [x1, x2, x3, x4]
}

pub fn i64_to_vecu8(number: &i64) -> [u8; 8] {
    let x1: u8 = ((number >> 56) & 0xff) as u8;
    let x2: u8 = ((number >> 48) & 0xff) as u8;
    let x3: u8 = ((number >> 40) & 0xff) as u8;
    let x4: u8 = ((number >> 32) & 0xff) as u8;
    let x5: u8 = ((number >> 24) & 0xff) as u8;
    let x6: u8 = ((number >> 16) & 0xff) as u8;
    let x7: u8 = ((number >> 8) & 0xff) as u8;
    let x8: u8 = (number & 0xff) as u8;
    [x1, x2, x3, x4, x5, x6, x7, x8]
}

pub fn vecu8_to_u32(vec: &[u8]) -> u32 {
    let x1: u32 = (vec[0] as u32) << 24;
    let x2: u32 = (vec[1] as u32) << 16;
    let x3: u32 = (vec[2] as u32) << 8;
    let x4: u32 = vec[3] as u32;
    x1 | x2 | x3 | x4
}

pub fn vecu8_to_u64(vec: &[u8]) -> u64 {
    let x1: u64 = (vec[0] as u64) << 56;
    let x2: u64 = (vec[1] as u64) << 48;
    let x3: u64 = (vec[2] as u64) << 40;
    let x4: u64 = (vec[3] as u64) << 32;
    let x5: u64 = (vec[4] as u64) << 24;
    let x6: u64 = (vec[5] as u64) << 16;
    let x7: u64 = (vec[6] as u64) << 8;
    let x8: u64 = vec[7] as u64;
    x1 | x2 | x3 | x4 | x5 | x6 | x7 | x8
}

pub fn create_id() -> String {
    let system_time = SystemTime::now();
    let datetime: DateTime<Utc> = system_time.into();
    let mut id = format!("{}{}54321", process::id(), (datetime.timestamp() as u64));
    if id.len() != 20 {
        id.push('0');
    }
    id
}

pub fn to_urlencoded(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| {
            if b.is_ascii_alphanumeric() || *b == b'.' || *b == b'-' || *b == b'_' || *b == b'~' {
                String::from(*b as char)
            } else {
                format!("%{:02x}", *b)
            }
        })
        .collect()
}
