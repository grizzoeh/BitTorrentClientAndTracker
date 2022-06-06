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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u32_to_vecu8() {
        let x1: u8 = ((0x12345678 >> 24) & 0xff) as u8;
        let x2: u8 = ((0x12345678 >> 16) & 0xff) as u8;
        let x3: u8 = ((0x12345678 >> 8) & 0xff) as u8;
        let x4: u8 = (0x12345678 & 0xff) as u8;
        assert_eq!(u32_to_vecu8(&0x12345678), [x1, x2, x3, x4]);
    }

    #[test]
    fn test_i64_to_vecu8() {
        let x1: u8 = ((100000000000000000 as i64 >> 56) & 0xff) as u8;
        let x2: u8 = ((100000000000000000 as i64 >> 48) & 0xff) as u8;
        let x3: u8 = ((100000000000000000 as i64 >> 40) & 0xff) as u8;
        let x4: u8 = ((100000000000000000 as i64 >> 32) & 0xff) as u8;
        let x5: u8 = ((100000000000000000 as i64 >> 24) & 0xff) as u8;
        let x6: u8 = ((100000000000000000 as i64 >> 16) & 0xff) as u8;
        let x7: u8 = ((100000000000000000 as i64 >> 8) & 0xff) as u8;
        let x8: u8 = (100000000000000000 as i64 & 0xff) as u8;
        assert_eq!(
            i64_to_vecu8(&(100000000000000000 as i64)),
            [x1, x2, x3, x4, x5, x6, x7, x8]
        );
    }

    #[test]
    fn test_vecu8_to_u32() {
        let x1: u8 = ((0x12345678 >> 24) & 0xff) as u8;
        let x2: u8 = ((0x12345678 >> 16) & 0xff) as u8;
        let x3: u8 = ((0x12345678 >> 8) & 0xff) as u8;
        let x4: u8 = (0x12345678 & 0xff) as u8;
        assert_eq!(vecu8_to_u32(&[x1, x2, x3, x4]), 0x12345678);
    }

    #[test]
    fn test_vecu8_to_u64() {
        let x1 = (100000000000000000 as u64 >> 56) & 0xff;
        let x2 = (100000000000000000 as u64 >> 48) & 0xff;
        let x3 = (100000000000000000 as u64 >> 40) & 0xff;
        let x4 = (100000000000000000 as u64 >> 32) & 0xff;
        let x5 = (100000000000000000 as u64 >> 24) & 0xff;
        let x6 = (100000000000000000 as u64 >> 16) & 0xff;
        let x7 = (100000000000000000 as u64 >> 8) & 0xff;
        let x8 = 100000000000000000 as u64 & 0xff;
        assert_eq!(
            vecu8_to_u64(&[
                x1.try_into().unwrap(),
                x2.try_into().unwrap(),
                x3.try_into().unwrap(),
                x4.try_into().unwrap(),
                x5.try_into().unwrap(),
                x6.try_into().unwrap(),
                x7.try_into().unwrap(),
                x8.try_into().unwrap()
            ]),
            100000000000000000 as u64
        );
    }

    #[test]
    fn test_create_id() {
        let id = create_id();
        assert_eq!(id.len(), 20);
        assert_eq!(id.chars().count(), 20);
        assert_eq!(id.chars().filter(|c| !c.is_digit(10)).count(), 0);
    }

    #[test]
    fn test_to_urlencoded_without_new_characters() {
        assert_eq!(
            to_urlencoded(&[
                b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n',
                b'o', b'p'
            ]),
            "abcdefghijklmnop"
        );
    }

    #[test]
    fn test_to_urlencoded_with_no_alphanumeric() {
        assert_eq!(
            to_urlencoded(&[
                b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n',
                b'o', b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', b'y', b'z', b'0', b'1',
                b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'.', b'-', b'_', b'~'
            ]),
            "abcdefghijklmnopqrstuvwxyz0123456789.-_~"
        );
    }

    #[test]
    fn test_to_urlencoded_with_strange_characters() {
        assert_eq!(
            to_urlencoded(&[
                b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n',
                b'o', b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', b'y', b'z', b'0', b'1',
                b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'.', b'-', b'_', b'~', b'!', b'@',
                b'#', b'$', b'%', b'^', b'&', b'*', b'(', b')', b'=', b'+', b'{', b'}', b'[', b']',
                b'|', b':', b';', b'<', b'>', b',', b'?', b'/', b'\\', b'\'', b'"', b'`', b' '
            ]),
            "abcdefghijklmnopqrstuvwxyz0123456789.-_~%21%40%23%24%25%5e%26%2a%28%29%3d%2b%7b%7d%5b%5d%7c%3a%3b%3c%3e%2c%3f%2f%5c%27%22%60%20"
        );
    }
}
