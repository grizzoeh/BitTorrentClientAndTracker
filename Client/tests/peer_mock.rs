use std::fs::File;
use std::io::Read;
use std::time::Duration;

use crabrave::{
    errors::communication_method_error::CommunicationMethodError,
    utilities::constants::RESERVED_SPACE_LEN,
};
use crabrave::{
    peer_entities::communication_method::CommunicationMethod, utilities::constants::PSTR,
};
use crabrave::{test_files::test_helper::MockTcpStream, utilities::constants::CHUNK_SIZE};

#[derive(Debug)]
pub struct CommunicationMock1 {
    pub(crate) stream: Option<MockTcpStream>,
    is_connected: bool,
    counter: usize,
    file: File,
}

impl CommunicationMethod for CommunicationMock1 {
    fn create() -> Box<(dyn CommunicationMethod + Send)> {
        Box::new(CommunicationMock1 {
            stream: None,
            is_connected: false,
            counter: 0,
            file: File::open("tests/test_files/integration_test_piece_src.txt".to_string())
                .unwrap(),
        })
    }
    fn connect(&mut self, _ip: &str, _port: u16) -> Result<(), CommunicationMethodError> {
        let stream = Some(MockTcpStream::new(vec![]));
        self.stream = stream;
        self.is_connected = true;
        Ok(())
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), CommunicationMethodError> {
        let prev_counter = self.counter;
        self.counter += 1;

        match prev_counter {
            0 => {
                //handshake
                let data = vec![PSTR.len() as u8];
                buf.copy_from_slice(&data);
            }
            1 => {
                let data = PSTR.as_bytes();
                buf.copy_from_slice(data);
            }
            2 => {
                let data = vec![0; RESERVED_SPACE_LEN as usize];
                buf.copy_from_slice(&data);
            }
            3 => {
                let data = vec![5; 20];
                buf.copy_from_slice(&data);
            }
            4 => {
                let data = "default1234567891023".as_bytes();
                buf.copy_from_slice(data);
            }
            5 => {
                //bitfield
                let data: Vec<u8> = vec![0, 0, 0, 11];
                buf.copy_from_slice(&data);
            }
            6 => {
                let data: [u8; 1] = [5];
                buf.copy_from_slice(&data);
            }
            7 => {
                let data: [u8; 10] = [255, 255, 255, 255, 255, 255, 255, 255, 255, 255];
                buf.copy_from_slice(&data);
            }
            8 => {
                //unchoke
                let data: Vec<u8> = vec![0, 0, 0, 1];
                buf.copy_from_slice(&data);
            }
            9 => {
                let data: Vec<u8> = vec![1];
                buf.copy_from_slice(&data);
            }
            10 => {
                //piece
                let data: Vec<u8> = vec![0, 0, 64, 9].to_vec();
                buf.copy_from_slice(&data);
            }
            11 => {
                let data: Vec<u8> = vec![7]; // piece message id
                buf.copy_from_slice(&data);
            }
            12 => {
                let data: Vec<u8> = vec![0, 0, 0, 0]; // index
                buf.copy_from_slice(&data);
            }
            13 => {
                let data: [u8; 4] = (CHUNK_SIZE * 0).to_be_bytes(); // offset
                buf.copy_from_slice(&data);
            }
            14 => {
                let _r = self.file.read_exact(buf); // block
            }
            15 => {
                //piece
                let data: Vec<u8> = vec![0, 0, 64, 9].to_vec();
                buf.copy_from_slice(&data);
            }
            16 => {
                let data: Vec<u8> = vec![7]; // piece message id
                buf.copy_from_slice(&data);
            }
            17 => {
                let data: Vec<u8> = vec![0, 0, 0, 0]; // index
                buf.copy_from_slice(&data);
            }
            18 => {
                let data: [u8; 4] = (CHUNK_SIZE * 1).to_be_bytes(); // offset
                buf.copy_from_slice(&data);
            }
            19 => {
                let _r = self.file.read_exact(buf); // block
            }
            20 => {
                //piece
                let data: Vec<u8> = vec![0, 0, 64, 9].to_vec();
                buf.copy_from_slice(&data);
            }
            21 => {
                let data: Vec<u8> = vec![7]; // piece message id
                buf.copy_from_slice(&data);
            }
            22 => {
                let data: Vec<u8> = vec![0, 0, 0, 0]; // index
                buf.copy_from_slice(&data);
            }
            23 => {
                let data: [u8; 4] = (CHUNK_SIZE * 2).to_be_bytes(); // offset
                buf.copy_from_slice(&data);
            }
            24 => {
                let _r = self.file.read_exact(buf); // block
            }
            25 => {
                //piece
                let data: Vec<u8> = vec![0, 0, 64, 9].to_vec();
                buf.copy_from_slice(&data);
            }
            26 => {
                let data: Vec<u8> = vec![7]; // piece message id
                buf.copy_from_slice(&data);
            }
            27 => {
                let data: Vec<u8> = vec![0, 0, 0, 0]; // index
                buf.copy_from_slice(&data);
            }
            28 => {
                let data: [u8; 4] = (CHUNK_SIZE * 3).to_be_bytes(); // offset
                buf.copy_from_slice(&data);
            }
            29 => {
                let _r = self.file.read_exact(buf); // block
            }
            30 => {
                //piece
                let data: Vec<u8> = vec![0, 0, 64, 9].to_vec();
                buf.copy_from_slice(&data);
            }
            31 => {
                let data: Vec<u8> = vec![7]; // piece message id
                buf.copy_from_slice(&data);
            }
            32 => {
                let data: Vec<u8> = vec![0, 0, 0, 0]; // index
                buf.copy_from_slice(&data);
            }
            33 => {
                let data: [u8; 4] = (CHUNK_SIZE * 4).to_be_bytes(); // offset
                buf.copy_from_slice(&data);
            }
            34 => {
                let _r = self.file.read_exact(buf); // block
            }
            35 => {
                //piece
                let data: Vec<u8> = vec![0, 0, 64, 9].to_vec();
                buf.copy_from_slice(&data);
            }
            36 => {
                let data: Vec<u8> = vec![7]; // piece message id
                buf.copy_from_slice(&data);
            }
            37 => {
                let data: Vec<u8> = vec![0, 0, 0, 0]; // index
                buf.copy_from_slice(&data);
            }
            38 => {
                let data: [u8; 4] = (CHUNK_SIZE * 5).to_be_bytes(); // offset
                buf.copy_from_slice(&data);
            }
            39 => {
                let _r = self.file.read_exact(buf); // block
            }
            40 => {
                //piece
                let data: Vec<u8> = vec![0, 0, 64, 9].to_vec();
                buf.copy_from_slice(&data);
            }
            41 => {
                let data: Vec<u8> = vec![7]; // piece message id
                buf.copy_from_slice(&data);
            }
            42 => {
                let data: Vec<u8> = vec![0, 0, 0, 0]; // index
                buf.copy_from_slice(&data);
            }
            43 => {
                let data: [u8; 4] = (CHUNK_SIZE * 6).to_be_bytes(); // offset
                buf.copy_from_slice(&data);
            }
            44 => {
                let _r = self.file.read_exact(buf); // block
            }
            45 => {
                //piece
                let data: Vec<u8> = vec![0, 0, 64, 9].to_vec();
                buf.copy_from_slice(&data);
            }
            46 => {
                let data: Vec<u8> = vec![7]; // piece message id
                buf.copy_from_slice(&data);
            }
            47 => {
                let data: Vec<u8> = vec![0, 0, 0, 0]; // index
                buf.copy_from_slice(&data);
            }
            48 => {
                let data: [u8; 4] = (CHUNK_SIZE * 7).to_be_bytes(); // offset
                buf.copy_from_slice(&data);
            }
            49 => {
                let _r = self.file.read_exact(buf); // block
            }
            50 => {
                //piece
                let data: Vec<u8> = vec![0, 0, 64, 9].to_vec();
                buf.copy_from_slice(&data);
            }
            51 => {
                let data: Vec<u8> = vec![7]; // piece message id
                buf.copy_from_slice(&data);
            }
            52 => {
                let data: Vec<u8> = vec![0, 0, 0, 0]; // index
                buf.copy_from_slice(&data);
            }
            53 => {
                let data: [u8; 4] = (CHUNK_SIZE * 8).to_be_bytes(); // offset
                buf.copy_from_slice(&data);
            }
            54 => {
                let _r = self.file.read_exact(buf); // block
            }
            55 => {
                //piece
                let data: Vec<u8> = vec![0, 0, 64, 9].to_vec();
                buf.copy_from_slice(&data);
            }
            56 => {
                let data: Vec<u8> = vec![7]; // piece message id
                buf.copy_from_slice(&data);
            }
            57 => {
                let data: Vec<u8> = vec![0, 0, 0, 0]; // index
                buf.copy_from_slice(&data);
            }
            58 => {
                let data: [u8; 4] = (CHUNK_SIZE * 9).to_be_bytes(); // offset
                buf.copy_from_slice(&data);
            }
            59 => {
                let _r = self.file.read_exact(buf); // block
            }
            60 => {
                //piece
                let data: Vec<u8> = vec![0, 0, 64, 9].to_vec();
                buf.copy_from_slice(&data);
            }
            61 => {
                let data: Vec<u8> = vec![7]; // piece message id
                buf.copy_from_slice(&data);
            }
            62 => {
                let data: Vec<u8> = vec![0, 0, 0, 0]; // index
                buf.copy_from_slice(&data);
            }
            63 => {
                let data: [u8; 4] = (CHUNK_SIZE * 10).to_be_bytes(); // offset
                buf.copy_from_slice(&data);
            }
            64 => {
                let _r = self.file.read_exact(buf); // block
            }
            65 => {
                //piece
                let data: Vec<u8> = vec![0, 0, 64, 9].to_vec();
                buf.copy_from_slice(&data);
            }
            66 => {
                let data: Vec<u8> = vec![7]; // piece message id
                buf.copy_from_slice(&data);
            }
            67 => {
                let data: Vec<u8> = vec![0, 0, 0, 0]; // index
                buf.copy_from_slice(&data);
            }
            68 => {
                let data: [u8; 4] = (CHUNK_SIZE * 11).to_be_bytes(); // offset
                buf.copy_from_slice(&data);
            }
            69 => {
                let _r = self.file.read_exact(buf); // block
            }
            70 => {
                //piece
                let data: Vec<u8> = vec![0, 0, 64, 9].to_vec();
                buf.copy_from_slice(&data);
            }
            71 => {
                let data: Vec<u8> = vec![7]; // piece message id
                buf.copy_from_slice(&data);
            }
            72 => {
                let data: Vec<u8> = vec![0, 0, 0, 0]; // index
                buf.copy_from_slice(&data);
            }
            73 => {
                let data: [u8; 4] = (CHUNK_SIZE * 12).to_be_bytes(); // offset
                buf.copy_from_slice(&data);
            }
            74 => {
                let _r = self.file.read_exact(buf); // block
            }
            75 => {
                //piece
                let data: Vec<u8> = vec![0, 0, 64, 9].to_vec();
                buf.copy_from_slice(&data);
            }
            76 => {
                let data: Vec<u8> = vec![7]; // piece message id
                buf.copy_from_slice(&data);
            }
            77 => {
                let data: Vec<u8> = vec![0, 0, 0, 0]; // index
                buf.copy_from_slice(&data);
            }
            78 => {
                let data: [u8; 4] = (CHUNK_SIZE * 13).to_be_bytes(); // offset
                buf.copy_from_slice(&data);
            }
            79 => {
                let _r = self.file.read_exact(buf); // block
            }
            80 => {
                //piece
                let data: Vec<u8> = vec![0, 0, 64, 9].to_vec();
                buf.copy_from_slice(&data);
            }
            81 => {
                let data: Vec<u8> = vec![7]; // piece message id
                buf.copy_from_slice(&data);
            }
            82 => {
                let data: Vec<u8> = vec![0, 0, 0, 0]; // index
                buf.copy_from_slice(&data);
            }
            83 => {
                let data: [u8; 4] = (CHUNK_SIZE * 14).to_be_bytes(); // offset
                buf.copy_from_slice(&data);
            }
            84 => {
                let _r = self.file.read_exact(buf); // block
            }
            85 => {
                //piece
                let data: Vec<u8> = vec![0, 0, 64, 9].to_vec();
                buf.copy_from_slice(&data);
            }
            86 => {
                let data: Vec<u8> = vec![7]; // piece message id
                buf.copy_from_slice(&data);
            }
            87 => {
                let data: Vec<u8> = vec![0, 0, 0, 0]; // index
                buf.copy_from_slice(&data);
            }
            88 => {
                let data: [u8; 4] = (CHUNK_SIZE * 15).to_be_bytes(); // offset
                buf.copy_from_slice(&data);
            }
            89 => {
                let _r = self.file.read_exact(buf); // block
            }
            _ => {
                //keep alive
                let data = vec![0, 0, 0, 0];
                buf.copy_from_slice(&data);
            }
        }
        Ok(())
    }

    fn write_all(&mut self, _buf: &[u8]) -> Result<(), CommunicationMethodError> {
        Ok(())
    }

    fn set_read_timeout(&mut self, _dur: Option<Duration>) -> Result<(), CommunicationMethodError> {
        Ok(())
    }

    fn peer_addr(&self) -> Result<std::net::SocketAddr, CommunicationMethodError> {
        Ok(std::net::SocketAddr::new(
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),
            0,
        ))
    }
    fn is_connected(&self) -> bool {
        self.stream.as_ref().is_some()
    }

    fn disconnect(&mut self) -> bool {
        self.stream = None;
        self.is_connected = false;
        // self.file.;
        true
    }
}
