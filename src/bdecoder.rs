use std::{collections::HashMap, convert::From, io};

#[derive(Debug, PartialEq)]
pub enum Decodification {
    Dic(HashMap<String, Decodification>),
    List(Vec<Decodification>),
    Int(i64),
    String(String),
}

#[derive(Debug)]
pub enum DecodeError {
    IOError(io::Error),
    UnexpectedEndOfBuffer,
    UnexpectedCharacter(String),
}

pub fn parse(bytes: &[u8]) -> Result<Decodification, DecodeError> {
    let (decoded, _) = parse_from(bytes, 0)?;
    Ok(decoded)
}

fn parse_from(bytes: &[u8], i: usize) -> Result<(Decodification, usize), DecodeError> {
    if bytes[i].is_ascii_digit() {
        // string case
        let decoded = decode_str(bytes, i)?;
        return Ok(decoded);
    }

    match bytes[i] {
        b'd' => {
            // Dictionary case
            let decoded = decode_dic(bytes, i)?;
            Ok(decoded)
        }
        b'i' => {
            // Integer case
            let decoded = decode_int(bytes, i)?;
            Ok(decoded)
        }
        b'l' => {
            // List case
            let decoded = decode_list(bytes, i)?;
            Ok(decoded)
        }
        _ => Err(DecodeError::UnexpectedCharacter(format!(
            "unexpected character: {}",
            bytes[i - 2] as char,
        ))),
    }
}

fn decode_int(bytes: &[u8], i: usize) -> Result<(Decodification, usize), DecodeError> {
    let mut j = i + 1;
    let mut num: i64 = 0;
    let mut is_negative = false;

    while bytes[j] != b'e' {
        if bytes[j] == b'-' {
            is_negative = true;
            j += 1;
            continue;
        }
        if !bytes[j].is_ascii_digit() && bytes[j] != b'-' {
            return Err(DecodeError::UnexpectedCharacter(String::from(
                "Not a digit",
            )));
        }
        num *= 10;
        num += (bytes[j] - b'0') as i64;
        j += 1;
    }
    if is_negative {
        num = -num;
    }

    let decoded = Decodification::Int(num);
    Ok((decoded, j + 1))
}

fn decode_str(bytes: &[u8], i: usize) -> Result<(Decodification, usize), DecodeError> {
    let mut decoded = String::new();
    let mut j = i;
    let mut len = 0;

    while bytes[j].is_ascii_digit() {
        len = len * 10 + (bytes[j] - b'0') as usize;
        j += 1;
    }
    if bytes[j] != b':' {
        return Err(DecodeError::UnexpectedCharacter(String::from(
            "expected ':' after string length",
        )));
    }
    j += 1;

    for _ in 0..len {
        decoded.push(bytes[j] as char);
        j += 1;
    }
    Ok((Decodification::String(decoded), j))
}

fn decode_dic(bencoded_dic: &[u8], i: usize) -> Result<(Decodification, usize), DecodeError> {
    let mut decoded: HashMap<String, Decodification> = HashMap::new();
    let mut j = i + 1;

    while bencoded_dic[j] != b'e' {
        let (key, key_index) = parse_from(bencoded_dic, j)?; // We are not checking if key is string
        let (value, value_index) = parse_from(bencoded_dic, key_index)?;

        if let Decodification::String(str_aux) = key {
            decoded.insert(str_aux, value);
        }

        j = value_index;
    }
    Ok((Decodification::Dic(decoded), j + 1))
}

fn decode_list(bencoded_list: &[u8], i: usize) -> Result<(Decodification, usize), DecodeError> {
    let mut decoded: Vec<Decodification> = Vec::new();
    let mut j = i + 1;

    while bencoded_list[j] != b'e' {
        let (parsed, index) = parse_from(bencoded_list, j)?;
        decoded.push(parsed);
        j = index;
    }
    Ok((Decodification::List(decoded), j + 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_positive_int() {
        let bencoded = b"i1e";
        let decoded = parse(bencoded);
        assert_eq!(decoded.unwrap(), Decodification::Int(1));
    }

    #[test]
    fn test_decode_negative_int() {
        let bencoded = b"i-12e";
        let decoded = parse(bencoded);
        assert_eq!(decoded.unwrap(), Decodification::Int(-12));
    }

    #[test]
    fn test_decode_big_int() {
        let bencoded = b"i12123124124124e";
        let decoded = parse(bencoded);
        assert_eq!(decoded.unwrap(), Decodification::Int(12123124124124));
    }

    #[test]
    fn test_decode_string() {
        let bencoded = b"6:espejo";
        let decoded = parse(bencoded);
        assert_eq!(
            decoded.unwrap(),
            Decodification::String(String::from("espejo"))
        );
    }

    #[test]
    fn test_decode_string_with_spaces() {
        let bencoded = b"3: aa";
        let decoded = parse(bencoded);
        assert_eq!(
            decoded.unwrap(),
            Decodification::String(String::from(" aa"))
        );
    }

    #[test]
    fn test_decode_string_with_spaces_and_colons() {
        let bencoded = b"7: aa: bb";
        let decoded = parse(bencoded);
        assert_eq!(
            decoded.unwrap(),
            Decodification::String(String::from(" aa: bb"))
        );
    }

    #[test]
    fn test_decode_empty_string() {
        let bencoded = b"0:";
        let decoded = parse(bencoded);
        assert_eq!(decoded.unwrap(), Decodification::String(String::from("")));
    }

    #[test]
    fn test_decode_list_with_strings() {
        let bencoded = b"l2:si3:sal4:ojos2:aee";
        let decoded = parse(bencoded);
        assert_eq!(
            decoded.unwrap(),
            Decodification::List(vec![
                Decodification::String(String::from("si")),
                Decodification::String(String::from("sal")),
                Decodification::String(String::from("ojos")),
                Decodification::String(String::from("ae")),
            ])
        );
    }

    #[test]
    fn test_decode_list_with_ints() {
        let bencoded = b"li1ei22ei31ei-441ei5000ee";
        let decoded = parse(bencoded);
        assert_eq!(
            decoded.unwrap(),
            Decodification::List(vec![
                Decodification::Int(1),
                Decodification::Int(22),
                Decodification::Int(31),
                Decodification::Int(-441),
                Decodification::Int(5000),
            ])
        );
    }

    #[test]
    fn test_decode_dic_with_strings() {
        let bencoded = b"d3:key4:hola6:holaaa5:joseae";
        let decoded = parse(bencoded);
        assert_eq!(
            decoded.unwrap(),
            Decodification::Dic(HashMap::from([
                (
                    String::from("holaaa"),
                    Decodification::String(String::from("josea"))
                ),
                (
                    String::from("key"),
                    Decodification::String(String::from("hola"))
                ),
            ]))
        );
    }

    #[test]
    fn test_decode_tracker_response() {
        let bencoded = b"d8:intervali456e8:completei23e10:incompletei112e5:peersld4:porti3000e2:ip6:holajoed4:porti3001e2:ip6:chaujoeee";
        let decoded = parse(bencoded);
        assert_eq!(
            decoded.unwrap(),
            Decodification::Dic(HashMap::from([
                (String::from("interval"), Decodification::Int(456)),
                (String::from("complete"), Decodification::Int(23)),
                (String::from("incomplete"), Decodification::Int(112)),
                (
                    String::from("peers"),
                    Decodification::List(vec![
                        Decodification::Dic(HashMap::from([
                            (String::from("port"), Decodification::Int(3000)),
                            (
                                String::from("ip"),
                                Decodification::String(String::from("holajo"))
                            ),
                        ])),
                        Decodification::Dic(HashMap::from([
                            (String::from("port"), Decodification::Int(3001)),
                            (
                                String::from("ip"),
                                Decodification::String(String::from("chaujo"))
                            ),
                        ])),
                    ])
                ),
            ]))
        );
    }
}
