use crate::bdecoder::Decodification;
use std::collections::{BTreeMap, HashMap};
use std::convert::From;
use std::string::ToString;

pub enum BencoderTypes {
    String(String),
    List(Vec<String>),
    Integer(i64),
    Dictionary(HashMap<String, String>),
    Decodification(Decodification),
}

pub fn bencode(obj: &BencoderTypes) -> Vec<u8> {
    match obj {
        BencoderTypes::String(obj) => encode_byte_string(obj),
        BencoderTypes::Integer(obj) => encode_int(obj),
        BencoderTypes::List(obj) => encode_list(obj),
        BencoderTypes::Dictionary(obj) => encode_dic(obj),
        BencoderTypes::Decodification(obj) => encode_decodification(obj),
    }
}

fn encode_decodification(obj: &Decodification) -> Vec<u8> {
    match obj {
        Decodification::String(obj) => encode_deco_byte_string(obj),
        Decodification::Int(obj) => encode_int(obj),
        Decodification::List(obj) => encode_deco_list(obj),
        Decodification::Dic(obj) => encode_deco_dic(obj),
    }
}

fn encode_deco_byte_string(obj: &[u8]) -> Vec<u8> {
    let mut encoded: Vec<u8> = vec![];
    let mut len = Vec::from(obj.len().to_string().as_bytes());
    encoded.append(&mut len);
    encoded.push(b':');
    encoded.append(&mut obj.to_owned());
    encoded
}

fn encode_deco_list(list: &[Decodification]) -> Vec<u8> {
    let mut vec = vec![b'l'];
    for elem in list {
        vec.extend_from_slice(&encode_decodification(elem));
    }
    vec.push(b'e');
    vec
}

fn encode_deco_dic(dict: &HashMap<Vec<u8>, Decodification>) -> Vec<u8> {
    let mut vec = vec![b'd'];
    let mut keys = dict.keys().cloned().collect::<Vec<Vec<u8>>>();
    keys.sort();
    for key in keys {
        vec.extend_from_slice(&encode_deco_byte_string(&key));
        vec.extend_from_slice(&encode_decodification(&dict[&key]));
    }
    vec.push(b'e');
    vec
}

fn encode_int(int: &i64) -> Vec<u8> {
    let mut vec = vec![b'i'];
    vec.extend_from_slice(int.to_string().as_bytes());
    vec.push(b'e');
    vec
}

fn encode_dic(hm: &HashMap<String, String>) -> Vec<u8> {
    let mut vec = vec![b'd'];
    let ordered: BTreeMap<_, _> = hm.iter().collect();
    for (key, value) in ordered.iter() {
        let value_aux = BencoderTypes::String(String::from(*value));
        vec.extend_from_slice(&encode_byte_string(key));
        vec.extend_from_slice(&bencode(&value_aux));
    }
    vec.push(b'e');
    vec
}

fn encode_list(list: &[String]) -> Vec<u8> {
    let mut vec = vec![b'l'];
    for elem in list {
        let elem_aux = BencoderTypes::String(String::from(elem));
        vec.extend_from_slice(&bencode(&elem_aux));
    }
    vec.push(b'e');
    vec
}

fn encode_byte_string(slice: &str) -> Vec<u8> {
    let mut vec = Vec::from(slice.len().to_string().as_bytes());
    vec.push(b':');
    vec.extend_from_slice(slice.as_bytes());
    vec
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bdecoder::bdecode;
    use sha1::{Digest, Sha1};

    #[test]
    fn string_lower_case() {
        let string = String::from("spam");
        let string_aux = BencoderTypes::String(String::from(string));
        assert_eq!(
            String::from("4:spam").as_bytes(),
            bencode(&string_aux).as_slice()
        );
    }

    #[test]
    fn string_upper_case() {
        let string = String::from("LEN");
        let string_aux = BencoderTypes::String(String::from(string));
        assert_eq!(
            (String::from("3:LEN")).as_bytes(),
            bencode(&string_aux).as_slice()
        );
    }

    #[test]
    fn string_lower_and_upper_case() {
        let string = String::from("MeDiuM");
        let string_aux = BencoderTypes::String(String::from(string));
        assert_eq!(
            String::from("6:MeDiuM").as_bytes(),
            bencode(&string_aux).as_slice()
        );
    }

    #[test]
    fn empty_string() {
        let string = String::from("");
        let string_aux = BencoderTypes::String(String::from(string));
        assert_eq!(
            String::from("0:").as_bytes(),
            bencode(&string_aux).as_slice()
        );
    }

    #[test]
    fn positive_int() {
        let int = 52;
        let int_aux = BencoderTypes::Integer(int);
        assert_eq!(
            String::from("i52e").as_bytes(),
            bencode(&int_aux).as_slice()
        );
    }

    #[test]
    fn negative_int() {
        let int = -45;
        let int_aux = BencoderTypes::Integer(int);
        assert_eq!(
            String::from("i-45e").as_bytes(),
            bencode(&int_aux).as_slice()
        );
    }

    #[test]
    fn big_int() {
        let int = 5200000000120312;
        let int_aux = BencoderTypes::Integer(int);
        assert_eq!(
            String::from("i5200000000120312e").as_bytes(),
            bencode(&int_aux).as_slice()
        );
    }

    #[test]
    fn string_list() {
        let mut list1 = Vec::new();
        list1.push(String::from("MeDiuM"));
        list1.push(String::from("spam"));
        let list1_aux = BencoderTypes::List(list1);

        let mut test_list = Vec::new();
        test_list.push(b'l');
        test_list.push(b'6');
        test_list.push(b':');
        test_list.push(b'M');
        test_list.push(b'e');
        test_list.push(b'D');
        test_list.push(b'i');
        test_list.push(b'u');
        test_list.push(b'M');
        test_list.push(b'4');
        test_list.push(b':');
        test_list.push(b's');
        test_list.push(b'p');
        test_list.push(b'a');
        test_list.push(b'm');
        test_list.push(b'e');

        assert_eq!(test_list, bencode(&list1_aux).as_slice());
    }

    #[test]
    fn string_dictionary() {
        let dict = HashMap::from([
            (String::from("Mercury"), String::from("mer")),
            (String::from("Venus"), String::from("ven")),
            (String::from("Earth"), String::from("ear")),
        ]);
        let dict_aux = BencoderTypes::Dictionary(dict);

        // The order depends on the hashing algorithm
        let mut test_list = Vec::new();
        test_list.push(b'd');

        test_list.push(b'5');
        test_list.push(b':');
        test_list.push(b'E');
        test_list.push(b'a');
        test_list.push(b'r');
        test_list.push(b't');
        test_list.push(b'h');
        test_list.push(b'3');
        test_list.push(b':');
        test_list.push(b'e');
        test_list.push(b'a');
        test_list.push(b'r');

        test_list.push(b'7');
        test_list.push(b':');
        test_list.push(b'M');
        test_list.push(b'e');
        test_list.push(b'r');
        test_list.push(b'c');
        test_list.push(b'u');
        test_list.push(b'r');
        test_list.push(b'y');
        test_list.push(b'3');
        test_list.push(b':');
        test_list.push(b'm');
        test_list.push(b'e');
        test_list.push(b'r');

        test_list.push(b'5');
        test_list.push(b':');
        test_list.push(b'V');
        test_list.push(b'e');
        test_list.push(b'n');
        test_list.push(b'u');
        test_list.push(b's');
        test_list.push(b'3');
        test_list.push(b':');
        test_list.push(b'v');
        test_list.push(b'e');
        test_list.push(b'n');

        test_list.push(b'e');

        assert_eq!(test_list, bencode(&dict_aux).as_slice());
    }

    #[test]
    fn encode_deco_byte_string() {
        let right = "8: AnElAp;".as_bytes().to_vec();
        let decoded = bdecode("8: AnElAp;".as_bytes()).unwrap();
        let decoded_aux = BencoderTypes::Decodification(decoded);
        let encoded = bencode(&decoded_aux);
        assert_eq!(right, encoded);
    }

    #[test]
    fn encode_deco_list() {
        let right = b"l2:si3:sal4:ojos2:aee";
        let decoded = bdecode(right).unwrap();
        let decoded_aux = BencoderTypes::Decodification(decoded);
        let encoded = bencode(&decoded_aux);
        assert_eq!(
            String::from_utf8(right.to_vec()),
            String::from_utf8(encoded)
        );
    }

    #[test]
    fn encode_deco_dic() {
        let right = b"d8:intervali456e3:olai1ee";
        let decoded = bdecode(right).unwrap();
        let decoded_aux = BencoderTypes::Decodification(decoded);
        let encoded = bencode(&decoded_aux);
        assert_eq!(right, encoded.as_slice());
    }
    #[test]
    fn encode_deco_tracker_response() {
        let right = b"d8:completei4e10:incompletei0e8:intervali1800e5:peersld2:ip12:91.189.95.217:peer id20:T03I--00L0fMrxsYDws64:porti6892eeee";
        let decoded = bdecode(right).unwrap();
        let decoded_aux = BencoderTypes::Decodification(decoded);
        let encoded = bencode(&decoded_aux);
        assert_eq!(
            String::from_utf8(encoded.clone()),
            String::from_utf8(right.to_vec())
        );

        let mut hasher = Sha1::new();
        hasher.update(encoded);
        let hash = hasher.finalize()[..].to_vec();
        let mut hasher = Sha1::new();
        hasher.update(right.clone());
        let right = hasher.finalize()[..].to_vec();
        assert_eq!(hash, right)
    }
}
