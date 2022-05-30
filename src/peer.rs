use std::io::prelude::*;
use std::net::TcpStream;

const PSTR: &str = "BitTorrent protocol";

#[derive(Clone)]
pub struct Peer {
    pub id: String,
    pub ip: String,
    pub port: u16,
    //pub is_connected: bool, ??  FIXME
    pub is_choked: bool,
    pub is_interested: bool,
    pub choked_me: bool,
    pub bitfield: Vec<u8>,
}

impl Peer {
    pub fn new(id: String, ip: String, port: u16) -> Peer {
        Peer {
            id,
            ip,
            port,
            is_choked: true,
            is_interested: false,
            choked_me: true,
            bitfield: vec![],
        }
    }

    pub fn handshake(&mut self, info_hash: Vec<u8>, peer_id: String) {
        // Must send <pstrlen><pstr><reserved><info_hash><peer_id>
        //println!("BITFIELD LEN= {}", 6 * 256 + 208); // = 1744 contando el primer 5
        // from 4 bytes big endian to u32

        let mut stream = match TcpStream::connect(format!("{}:{}", &self.ip, self.port)) {
            Ok(stream) => stream,
            Err(_) => return,
        };
        let mut data = vec![PSTR.len() as u8];
        data.extend(PSTR.as_bytes());
        data.extend(vec![0u8; 8]);
        data.extend(&info_hash);
        data.extend(peer_id.as_bytes());
        println!("DATA: {:?}", data);
        let _i = match stream.write_all(&data) {
            Ok(i) => i,
            Err(_) => return,
        };

        // Must receive <pstrlen><pstr><reserved><info_hash><peer_id>
        let mut response = Vec::new();
        let _i = match stream.read_to_end(&mut response) {
            Ok(i) => i,
            Err(_) => return,
        };
        // necesitamos el largo del bitfield? lo calculo para no olvidarme como era

        // let _bitfield_len = response[0] * to_power_of(2, 24) EN PROCESO DE DEBUG
        //     + response[1] * to_power_of(2, 16)
        //     + response[2] * to_power_of(2, 8)
        //     + response[3];
        // //checkeo que el peer id sea el mismo
        // if peer_id.as_bytes().to_vec() == response[data.len() - 20..data.len()] {
        //     self.bitfield = response[data.len()..].to_vec(); // me guardo el bitfield
        // }

        println!("\nPEER HANDSHAKE: {:?}\n", response[0..68].to_vec());
        println!("PEER BITFIELD: {:?}", response[68..].to_vec());
    }

    // HasPiece tells if a bitfield has a particular index set
    fn _has_piece(&mut self, index: u8) -> bool {
        let byte_index = index / 8;
        let offset = index % 8;
        self.bitfield[byte_index as usize] >> (7 - offset) & 1 != 0
    }
}
// hago esta funcion porq el formatter me pone 2**8 como 2* *8 y explota
fn _to_power_of(number: u8, exp: u8) -> u8 {
    let mut result = 1;
    for _ in 0..exp {
        result *= number;
    }
    result
}
