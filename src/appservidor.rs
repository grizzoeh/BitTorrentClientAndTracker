use crabrave::constants::*;

use crabrave::utils::create_id;
use crabrave::utils::vecu8_to_u32;
use sha1::{Digest, Sha1};
use std::fs::File;
use std::io::{self, Read, Write};
use std::net::TcpStream;

#[derive(Eq, PartialEq, Debug, Clone)]
struct Metainfo {
    announce: String,
    info: Info,
    info_hash: Vec<u8>,
}

#[derive(Eq, PartialEq, Debug, Clone)]
struct Info {
    piece_length: u32,
    pieces: Vec<Vec<u8>>,
    name: String,
    length: u32,
    //files: Vec<File>,
}

#[derive(Eq, PartialEq, Debug)]

struct PeerMessage {
    id: PeerMessageId,
    length: u32,
    payload: Vec<u8>,
}
impl PeerMessage {
    fn request(piece_idx: u32, offset: u32, length: u32) -> PeerMessage {
        let mut vec_message = REQUEST_MESSAGE.to_vec();
        vec_message.extend(&piece_idx.to_be_bytes());
        vec_message.extend(&offset.to_be_bytes());
        vec_message.extend(&length.to_be_bytes());
        PeerMessage {
            id: PeerMessageId::Request,
            length: vec_message.len() as u32,
            payload: vec_message, ////////OJO
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone)]
enum PeerMessageId {
    Unchoke,
    Bitfield,
    Interested,
    NotInterested,
    Have,
    Request,
    Piece,
    Cancel,
    Port,
}
impl PeerMessageId {
    fn from_u8(value: u8) -> Self {
        match value {
            0 => PeerMessageId::Unchoke,
            1 => PeerMessageId::Bitfield,
            2 => PeerMessageId::Interested,
            3 => PeerMessageId::NotInterested,
            4 => PeerMessageId::Have,
            5 => PeerMessageId::Request,
            6 => PeerMessageId::Piece,
            7 => PeerMessageId::Cancel,
            8 => PeerMessageId::Port,
            _ => panic!("Invalid PeerMessageId"),
        }
    }
}

pub fn string_to_vec_u8(string: &str) -> &[u8] {
    string.as_bytes()
}

fn get_metainfo(pieces: Vec<Vec<u8>>, info_hash: Vec<u8>) -> Metainfo {
    let announce: String = "127.0.0.1".to_string();

    let info: Info = Info {
        piece_length: 8,
        pieces,
        name: "target.txt".to_string(),
        length: 24, // 3 pieces of 8 bytes each
                    //files: <Vec<File>>::new(),
    };

    Metainfo {
        announce,
        info,
        info_hash,
    }
}

fn sha1_of(vec: &[u8]) -> Vec<u8> {
    let mut hasher = Sha1::new();
    hasher.update(vec);
    hasher.finalize().to_vec()
}

// Returns a vector of the pieces each piece
fn init_pieces() -> (Vec<Vec<u8>>, Vec<u8>) {
    let mut pieces: Vec<Vec<u8>> = Vec::new();
    let mut joined_pieces: Vec<u8> = Vec::new();

    let mut file_0 = File::create("src/appservidor_pieces/0.txt").unwrap();
    let mut buf_0: Vec<u8> = Vec::new();
    for i in 0..8 {
        buf_0.push(i as u8);
        joined_pieces.push(i as u8);
    }
    pieces.push(buf_0.clone());

    file_0.write_all(buf_0.as_slice()).unwrap();

    let mut file_1 = File::create("src/appservidor_pieces/1.txt").unwrap();
    let mut buf_1: Vec<u8> = Vec::new();
    for i in 0..8 {
        buf_1.push((8 - i) as u8);
        joined_pieces.push((8 - i) as u8)
    }
    pieces.push(buf_1.clone());
    file_1.write_all(buf_1.as_slice()).unwrap();

    let mut file_2 = File::create("src/appservidor_pieces/2.txt").unwrap();
    let mut buf_2: Vec<u8> = Vec::new();
    for _ in 0..8 {
        buf_2.push(3_u8);
        joined_pieces.push(3_u8);
    }
    pieces.push(buf_2.clone());
    file_2.write_all(buf_2.as_slice()).unwrap();

    (
        pieces,
        [
            241, 252, 220, 20, 98, 211, 101, 48, 245, 38, 193, 217, 64, 46, 236, 145, 0, 183, 186,
            24,
        ]
        .to_vec(),
    )
}

// fn create_handshake_message(info_hash: &[u8], peer_id: &str) -> Vec<u8> {
//     let mut handshake_message = Vec::new();
//     handshake_message.extend_from_slice(&[19]);
//     handshake_message.extend_from_slice(b"BitTorrent protocol");
//     handshake_message.extend_from_slice(&[0u8; 8]);
//     handshake_message.extend_from_slice(info_hash);
//     handshake_message.extend_from_slice(&string_to_vec_u8(peer_id));
//     handshake_message
// }

fn send_message(stream: &mut TcpStream, _message: &PeerMessage) -> Result<(), io::Error> {
    // let mut bytes = Vec::with_capacity((message.length + 4) as usize);
    // bytes.extend_from_slice(&message.length.to_be_bytes());
    // bytes.extend_from_slice(&(message.id.clone() as u8).to_be_bytes());
    // bytes.extend_from_slice(&message.payload);

    let mut vec_message = REQUEST_MESSAGE.to_vec();
    vec_message.extend([0].to_vec());
    vec_message.extend(&INITIAL_OFFSET.to_be_bytes());
    vec_message.extend(&CHUNK_SIZE.to_be_bytes());

    stream
        .write_all(&[0, 0, 0, 13, 6, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 64, 0])
        .unwrap();
    Ok(())
}

fn wait_for_message(stream: &mut TcpStream) -> Result<PeerMessage, io::Error> {
    let mut message_length = [0, 0, 0, 0];
    stream.read_exact(&mut message_length).unwrap();
    println!("message_length: {:?}", message_length);

    let message_length_aux = vecu8_to_u32(&message_length);
    println!("Client: received message of length {}", message_length_aux);

    let mut message_id = [0];

    stream.read_exact(&mut message_id).unwrap();

    let mut payload: Vec<u8> = vec![0; (message_length_aux - 1) as usize];
    stream.read_exact(&mut payload).unwrap();

    let msg = PeerMessage {
        id: PeerMessageId::from_u8(message_id[0]),
        length: message_length_aux,
        payload,
    };

    Ok(msg)
}

fn init_connection(stream: &mut TcpStream, _meta: &Metainfo, _peer_id: &str) {
    // let handshake_message: Vec<u8> = create_handshake_message(&meta.info_hash, &peer_id);
    let handshake_message = [
        19, 66, 105, 116, 84, 111, 114, 114, 101, 110, 116, 32, 112, 114, 111, 116, 111, 99, 111,
        108, 0, 0, 0, 0, 0, 0, 0, 0, 44, 107, 104, 88, 214, 29, 169, 84, 61, 66, 49, 167, 29, 180,
        177, 201, 38, 75, 6, 133, 100, 101, 102, 97, 117, 108, 116, 95, 105, 100,
    ];
    stream.write_all(&handshake_message).unwrap();
    let mut handshake_response = [0u8; 68];
    stream.read_exact(&mut handshake_response).unwrap();

    println!("Client: handshake and initial messages successful");
    let unchoke_message: PeerMessage = wait_for_message(stream).unwrap();

    if unchoke_message.id == PeerMessageId::Unchoke {
        println!("Client received unchoke message from server");
    } else {
        println!("Error: Client should have received an unchoke message");
    }

    println!("Client waiting for bitfield message...");
    let bitfield_message: PeerMessage = wait_for_message(stream).unwrap();

    if bitfield_message.id == PeerMessageId::Bitfield {
        println!(
            "Client received bitfield from server: {:?}",
            bitfield_message
        );
    } else {
        println!("Error: should have received bitfield message form server");
    }
}

fn ask_for_pieces(stream: &mut TcpStream, meta: &Metainfo) -> Vec<Vec<u8>> {
    let mut pieces: Vec<Vec<u8>> = Vec::new();

    println!("Client about to start asking server for pieces");

    // Once handshake is done, we can start requesting pieces
    // We start asking for piece 0, which has the size of the asked block
    let first_request = PeerMessage::request(0, 0, meta.info.piece_length as u32);
    println!("Client: About to send {:?} to server", first_request);
    send_message(stream, &first_request).unwrap();

    println!("Client sent message {:?} to server", first_request);

    let first_response: PeerMessage = wait_for_message(stream).unwrap();
    if first_response.id == PeerMessageId::Piece {
        println!("Received piece 0 from server");
        pieces.push(first_response.payload[8..].to_vec());
    } else {
        println!("Peer message: {:?}", first_response);
        println!("Got invalid message asking for piece 0");
    }

    // asks for piece 1
    send_message(
        stream,
        &PeerMessage::request(1, 0, meta.info.piece_length as u32),
    )
    .unwrap();
    let first_response: PeerMessage = wait_for_message(stream).unwrap();
    if first_response.id == PeerMessageId::Piece {
        println!("Received piece 1 from server");
        pieces.push(first_response.payload[8..].to_vec());
    } else {
        println!("Got invalid message asking for piece 1");
    }

    // ask for piece 2
    send_message(
        stream,
        &PeerMessage::request(2, 0, meta.info.piece_length as u32),
    )
    .unwrap();
    let third_response: PeerMessage = wait_for_message(stream).unwrap();
    if third_response.id == PeerMessageId::Piece {
        println!("Received piece 2 from server");
        pieces.push(third_response.payload[8..].to_vec());
    } else {
        println!("Got invalid message asking for piece 2");
    }

    pieces
}

fn piece_is_valid(data: &Vec<u8>, expected: Vec<u8>) -> bool {
    println!("Expected: {:?}\nActual: {:?}\n\n", expected, data);
    sha1_of(data) == sha1_of(&expected)
}

fn run_test_client_server() {
    //upload manager
    let peer_id = create_id();
    let (pieces, info_hash) = init_pieces();
    let expected_pieces = pieces.clone();
    let meta: Metainfo = get_metainfo(pieces, info_hash);
    let meta_clone = meta;
    let peer_id_clone = peer_id;

    std::thread::sleep(std::time::Duration::from_secs(3));
    println!("About to connect to server from client");
    let mut stream: TcpStream = TcpStream::connect("127.0.0.1:1476").unwrap();

    init_connection(&mut stream, &meta_clone, &peer_id_clone);
    let pieces_data: Vec<Vec<u8>> = ask_for_pieces(&mut stream, &meta_clone);

    for (index, piece_data) in pieces_data.iter().enumerate() {
        let expected_piece = expected_pieces.get(index).unwrap();
        if piece_is_valid(piece_data, expected_piece.to_vec()) {
            println!("Piece {} was validated correctly", index);
        } else {
            println!("Piece {} hash is invalid", index);
        }
    }
}

fn main() {
    run_test_client_server();
}
