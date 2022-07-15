use crabrave::utilities::constants::*;
use sha1::{Digest, Sha1};
use std::{
    fs::File,
    io::{self, Read, Write},
    net::TcpStream,
};

const BITFIELD_LEN: usize = 195;
const ID_POSITION: usize = 4;

/// This function tests that the application can send pieces to another peer.
fn main() {
    let piece_length = 262144;

    std::thread::sleep(std::time::Duration::from_secs(3));
    println!("Connecting to server...");
    let mut stream: TcpStream = TcpStream::connect("127.0.0.1:1476").unwrap();

    let handshake_message = [
        19, 66, 105, 116, 84, 111, 114, 114, 101, 110, 116, 32, 112, 114, 111, 116, 111, 99, 111,
        108, 0, 0, 0, 0, 0, 24, 0, 5, 177, 17, 129, 60, 230, 15, 66, 145, 151, 52, 130, 61, 245,
        236, 32, 189, 30, 4, 231, 247, 45, 68, 69, 50, 48, 51, 115, 45, 103, 109, 33, 107, 88, 97,
        81, 88, 56, 102, 51, 52,
    ];

    println!("Sending Handshake...");

    stream.write_all(&handshake_message).unwrap(); // Send handshake message

    let mut handshake_response = [0u8; 68];

    stream.read_exact(&mut handshake_response).unwrap(); // Read handshake
    println!("Handshake response received.");

    println!("Waiting for unchoke message...");
    let unchoke_message = wait_for_message(&mut stream, UNCHOKE_LEN).unwrap(); // Wait for unchoke

    if unchoke_message.0 == 1 {
        println!("Unchoke message received");
    } else {
        println!(
            "Error: Did not receive unchoke, the message was {:?}",
            unchoke_message
        );
    }

    println!("Waiting for bitfield message...");
    let bitfield_message = wait_for_message(&mut stream, BITFIELD_LEN).unwrap(); // Wait for bitfield
    if bitfield_message.0 != BITFIELD_ID {
        println!(
            "Error: Did not receive bitfield, the message was {:?}",
            bitfield_message
        );
    } else {
        println!("Bitfield received");
    }

    println!("Sending piece requests...");
    let mut offset: u32 = INITIAL_OFFSET;
    let mut piece_data: Vec<u8> = vec![];
    while offset < piece_length {
        // Ask application for pieces
        println!("The offset is: {}", offset);

        request(&mut stream, 0, offset, CHUNK_SIZE); // Send request
        let mut msg_len = [0u8; CHUNK_LEN_LEN];
        let mut id = [0u8; MESSAGE_ID_LEN];
        let mut index = [0u8; PIECE_INDEX_LEN];
        let mut begin = [0u8; PIECE_OFFSET_LEN];

        stream.read_exact(&mut msg_len).unwrap();
        stream.read_exact(&mut id).unwrap();
        stream.read_exact(&mut index).unwrap();
        stream.read_exact(&mut begin).unwrap();

        let mut response = [0u8; CHUNK_SIZE as usize];
        stream.read_exact(&mut response).unwrap(); // Read chunk

        piece_data.extend(response);

        offset += CHUNK_SIZE;
    }

    println!("Received successfully");

    let mut piece_file =
        File::open("src/downloaded_pieces/debian-11.3.0-amd64-netinst.iso.torrent/piece_0.txt")
            .unwrap();

    let mut piece_file_data = vec![];
    piece_file.read_to_end(&mut piece_file_data).unwrap(); // Read real piece file

    assert_eq!(piece_file_data.len(), piece_data.len()); // Same length
    assert_eq!(apply_sha1(&piece_data), apply_sha1(&piece_file_data)); // Same sha1 hash

    // // PIECE 2
    // let mut offset: u32 = INITIAL_OFFSET;
    // let mut piece_data: Vec<u8> = vec![];
    // while offset < piece_length {
    //     println!("The offset is: {}", offset);

    //     request(&mut stream, 1, offset, CHUNK_SIZE);
    //     let mut msg_len = [0u8; CHUNK_LEN_LEN];
    //     let mut id = [0u8; MESSAGE_ID_LEN];
    //     let mut index = [0u8; PIECE_INDEX_LEN];
    //     let mut begin = [0u8; PIECE_OFFSET_LEN];

    //     stream.read_exact(&mut msg_len).unwrap();
    //     stream.read_exact(&mut id).unwrap();
    //     stream.read_exact(&mut index).unwrap();
    //     stream.read_exact(&mut begin).unwrap();

    //     let mut response = [0u8; CHUNK_SIZE as usize];
    //     stream.read_exact(&mut response).unwrap();

    //     piece_data.extend(response);

    //     offset += CHUNK_SIZE;
    // }

    // println!("Received piece successfully");

    // let mut piece_file =
    //     File::open("src/downloaded_pieces/debian-11.3.0-amd64-netinst.iso.torrent/piece_1.txt")
    //         .unwrap();

    // let mut piece_file_data = vec![];
    // piece_file.read_to_end(&mut piece_file_data).unwrap(); // Read real piece file

    // assert_eq!(piece_file_data.len(), piece_data.len()); // Same length
    // assert_eq!(apply_sha1(&piece_data), apply_sha1(&piece_file_data)); // Same sha1 hash

    // // PIECE 3
    // let mut offset: u32 = INITIAL_OFFSET;
    // let mut piece_data: Vec<u8> = vec![];
    // while offset < piece_length {
    //     println!("The offset is: {}", offset);

    //     request(&mut stream, 2, offset, CHUNK_SIZE);
    //     let mut msg_len = [0u8; CHUNK_LEN_LEN];
    //     let mut id = [0u8; MESSAGE_ID_LEN];
    //     let mut index = [0u8; PIECE_INDEX_LEN];
    //     let mut begin = [0u8; PIECE_OFFSET_LEN];

    //     stream.read_exact(&mut msg_len).unwrap();
    //     stream.read_exact(&mut id).unwrap();
    //     stream.read_exact(&mut index).unwrap();
    //     stream.read_exact(&mut begin).unwrap();

    //     let mut response = [0u8; CHUNK_SIZE as usize];
    //     stream.read_exact(&mut response).unwrap();

    //     piece_data.extend(response);

    //     offset += CHUNK_SIZE;
    // }

    // println!("Received piece successfully");

    // let mut piece_file =
    //     File::open("src/downloaded_pieces/debian-11.3.0-amd64-netinst.iso.torrent/piece_2.txt")
    //         .unwrap();

    // let mut piece_file_data = vec![];
    // piece_file.read_to_end(&mut piece_file_data).unwrap(); // Read real piece file

    let choke_message = [0, 0, 0, 1, 0];
    stream.write_all(&choke_message).unwrap(); // Send handshake message

    // assert_eq!(piece_file_data.len(), piece_data.len()); // Same length
    // assert_eq!(apply_sha1(&piece_data), apply_sha1(&piece_file_data)); // Same sha1 hash
}

/// Returns the SHA1 hash of the given vec.
fn apply_sha1(vec: &[u8]) -> Vec<u8> {
    let mut hasher = Sha1::new();
    hasher.update(vec);
    hasher.finalize().to_vec()
}

/// Reads n bytes from the stream and returns (message_id, ).
fn wait_for_message(stream: &mut TcpStream, len: usize) -> Result<(u8, Vec<u8>), io::Error> {
    let msg_len_vec = read_n_bytes(stream, len)?;
    let msg_id = msg_len_vec[ID_POSITION];

    Ok((msg_id, msg_len_vec))
}

/// Reads n bytes from the stream.
fn read_n_bytes(stream: &mut TcpStream, n: usize) -> Result<Vec<u8>, io::Error> {
    let mut buffer = vec![0; n];
    stream.read_exact(buffer.as_mut())?;
    Ok(buffer)
}

/// This function sends a request to the application.
fn request(stream: &mut TcpStream, piece_idx: u32, offset: u32, length: u32) {
    let mut vec_message = REQUEST_MESSAGE.to_vec();
    vec_message.extend(&piece_idx.to_be_bytes());
    vec_message.extend(&offset.to_be_bytes());
    vec_message.extend(&length.to_be_bytes());
    println!("Requesting {:?}", vec_message);
    let _ = stream.write_all(vec_message.as_slice());
}
