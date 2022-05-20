use std::collections::{BTreeMap, HashMap};
use std::convert::From;
use std::string::ToString;

/// Bencoder only receives 4 types: String, Vec<String>, Int64 & HashMap<String, String>.
pub enum BencoderTypes {
    String(String),
    List(Vec<String>),
    Integer(i64),
    Dictionary(HashMap<String, String>),
}

/// encode must receive something of type BencoderTypes
pub fn encode(obj: BencoderTypes) -> Vec<u8> {
    match obj {
        BencoderTypes::String(obj) => encode_byte_string(&obj),
        BencoderTypes::Integer(obj) => encode_int(obj),
        BencoderTypes::List(obj) => encode_list(&obj),
        BencoderTypes::Dictionary(obj) => encode_dic(&obj),
    }
}

/// Example: encode it from 123 to "i123e" as [u8].
fn encode_int(int: i64) -> Vec<u8> {
    let mut vec = vec![b'i'];
    vec.extend_from_slice(int.to_string().as_bytes()); // Push int as bytes
    vec.push(b'e');
    vec
}

/// Example: encode it from {"hola": "chau", "mesa": "silla"} to "d4:hola4:chau4:mesa5:sillae" as [u8]
fn encode_dic(hm: &HashMap<String, String>) -> Vec<u8> {
    let mut vec = vec![b'd'];
    let ordered: BTreeMap<_, _> = hm.iter().collect();
    for (key, value) in ordered.iter() {
        let value_aux = BencoderTypes::String(String::from(*value));
        vec.extend_from_slice(&encode_byte_string(key));
        vec.extend_from_slice(&encode(value_aux));
    }
    vec.push(b'e');
    vec
}

/// Example: encode it from ["hola", "chau", "mesa", "silla"] to "l4:hola4:chau4:mesa5:sillae" as [u8]
fn encode_list(list: &[String]) -> Vec<u8> {
    let mut vec = vec![b'l'];
    for elem in list {
        let elem_aux = BencoderTypes::String(String::from(elem));
        vec.extend_from_slice(&encode(elem_aux));
    }
    vec.push(b'e');
    vec
}

/// Example: encode it from "string" to "6:string" as [u8]
fn encode_byte_string(slice: &str) -> Vec<u8> {
    let mut vec = Vec::from(slice.len().to_string().as_bytes()); // Len in base ten ASCII
    vec.push(b':');
    vec.extend_from_slice(slice.as_bytes());
    vec
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_lower_case() {
        let string = String::from("spam");
        let string_aux = BencoderTypes::String(String::from(string));
        assert_eq!(
            String::from("4:spam").as_bytes(),
            encode(string_aux).as_slice()
        );
    }

    #[test]
    fn string_upper_case() {
        let string = String::from("LEN");
        let string_aux = BencoderTypes::String(String::from(string));
        assert_eq!(
            (String::from("3:LEN")).as_bytes(),
            encode(string_aux).as_slice()
        );
    }

    #[test]
    fn string_lower_and_upper_case() {
        let string = String::from("MeDiuM");
        let string_aux = BencoderTypes::String(String::from(string));
        assert_eq!(
            String::from("6:MeDiuM").as_bytes(),
            encode(string_aux).as_slice()
        );
    }

    #[test]
    fn empty_string() {
        let string = String::from("");
        let string_aux = BencoderTypes::String(String::from(string));
        assert_eq!(String::from("0:").as_bytes(), encode(string_aux).as_slice());
    }

    #[test]
    fn positive_int() {
        let int = 52;
        let int_aux = BencoderTypes::Integer(int);
        assert_eq!(String::from("i52e").as_bytes(), encode(int_aux).as_slice());
    }

    #[test]
    fn negative_int() {
        let int = -45;
        let int_aux = BencoderTypes::Integer(int);
        assert_eq!(String::from("i-45e").as_bytes(), encode(int_aux).as_slice());
    }

    #[test]
    fn big_int() {
        let int = 5200000000120312;
        let int_aux = BencoderTypes::Integer(int);
        assert_eq!(
            String::from("i5200000000120312e").as_bytes(),
            encode(int_aux).as_slice()
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

        assert_eq!(test_list, encode(list1_aux).as_slice());
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

        assert_eq!(test_list, encode(dict_aux).as_slice());
    }
}
