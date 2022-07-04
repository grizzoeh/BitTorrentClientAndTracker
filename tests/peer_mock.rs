use std::time::Duration;

use crabrave::{communication_method::CommunicationMethod, constants::PSTR};
use crabrave::{constants::CHUNK_SIZE, test_helper::MockTcpStream};
use crabrave::{
    constants::RESERVED_SPACE_LEN, errors::communication_method_error::CommunicationMethodError,
};

// desde aca para integración
#[derive(Debug, Clone)]
pub struct CommunicationMock1 {
    pub(crate) stream: Option<MockTcpStream>,
    is_connected: bool,
    counter: usize,
}

impl CommunicationMethod for CommunicationMock1 {
    fn create() -> Box<(dyn CommunicationMethod + Send)> {
        Box::new(CommunicationMock1 {
            stream: None,
            is_connected: false,
            counter: 0,
        })
    }
    fn connect(&mut self, _ip: &str, _port: u16) -> Result<(), CommunicationMethodError> {
        let stream = Some(MockTcpStream::new(vec![]));
        self.stream = stream;
        self.is_connected = true;
        Ok(())
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), CommunicationMethodError> {
        match self.counter {
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
                let data = "default".as_bytes();
                buf.copy_from_slice(data);
            }
            5 => {
                //bitfield
                let data: Vec<u8> = vec![0, 0, 0, 2]; // largo 2, id 5, un 7 porq tenemos 3 piezas
                buf.copy_from_slice(&data);
            }
            6 => {
                let data: [u8; 1] = [5];
                buf.copy_from_slice(&data);
            }
            7 => {
                let data: [u8; 1] = [7];
                buf.copy_from_slice(&data);
            }
            8 => {
                //unchoke
                let data: Vec<u8> = vec![0, 0, 0, 1];
                buf.copy_from_slice(&data);
            }
            9 => {
                //choke
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
                let data: Vec<u8> = vec![0, 0, 0, 0]; // offset
                buf.copy_from_slice(&data);
            }
            14 => {
                let data: Vec<u8> = vec![0; CHUNK_SIZE as usize]; // block
                buf.copy_from_slice(&data);
            }
            _ => {
                //keep alive
                let data = vec![0, 0, 0, 0];
                println!("LLAMÓ DE MÁS A PEER DE INTEGRACION");
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
}

#[derive(Debug, Clone)]
pub struct CommunicationMock2 {
    pub(crate) stream: Option<MockTcpStream>,
    is_connected: bool,
    counter: usize,
}

impl CommunicationMethod for CommunicationMock2 {
    fn create() -> Box<(dyn CommunicationMethod + Send)> {
        Box::new(CommunicationMock2 {
            stream: None,
            is_connected: false,
            counter: 0,
        })
    }
    fn connect(&mut self, _ip: &str, _port: u16) -> Result<(), CommunicationMethodError> {
        let stream = Some(MockTcpStream::new(vec![]));
        self.stream = stream;
        self.is_connected = true;
        Ok(())
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), CommunicationMethodError> {
        match self.counter {
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
                let data = "default".as_bytes();
                buf.copy_from_slice(data);
            }
            5 => {
                //bitfield
                let data: Vec<u8> = vec![0, 0, 0, 2]; // largo 2, id 5, un 7 porq tenemos 3 piezas
                buf.copy_from_slice(&data);
            }
            6 => {
                let data: [u8; 1] = [5];
                buf.copy_from_slice(&data);
            }
            7 => {
                let data: [u8; 1] = [7];
                buf.copy_from_slice(&data);
            }
            8 => {
                //unchoke
                let data: Vec<u8> = vec![0, 0, 0, 1];
                buf.copy_from_slice(&data);
            }
            9 => {
                //choke
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
                let data: Vec<u8> = vec![0, 0, 0, 0]; // offset
                buf.copy_from_slice(&data);
            }
            14 => {
                let data: Vec<u8> = vec![1; CHUNK_SIZE as usize]; // block
                buf.copy_from_slice(&data);
            }
            _ => {
                //keep alive
                let data = vec![0, 0, 0, 0];
                println!("LLAMÓ DE MÁS A PEER DE INTEGRACION");
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
}

#[derive(Debug, Clone)]
pub struct CommunicationMock3 {
    pub(crate) stream: Option<MockTcpStream>,
    is_connected: bool,
    counter: usize,
}

impl CommunicationMethod for CommunicationMock3 {
    fn create() -> Box<(dyn CommunicationMethod + Send)> {
        Box::new(CommunicationMock3 {
            stream: None,
            is_connected: false,
            counter: 0,
        })
    }
    fn connect(&mut self, _ip: &str, _port: u16) -> Result<(), CommunicationMethodError> {
        let stream = Some(MockTcpStream::new(vec![]));
        self.stream = stream;
        self.is_connected = true;
        Ok(())
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), CommunicationMethodError> {
        match self.counter {
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
                let data = "default".as_bytes();
                buf.copy_from_slice(data);
            }
            5 => {
                //bitfield
                let data: Vec<u8> = vec![0, 0, 0, 2]; // largo 2, id 5, un 7 porq tenemos 3 piezas
                buf.copy_from_slice(&data);
            }
            6 => {
                let data: [u8; 1] = [5];
                buf.copy_from_slice(&data);
            }
            7 => {
                let data: [u8; 1] = [7];
                buf.copy_from_slice(&data);
            }
            8 => {
                //unchoke
                let data: Vec<u8> = vec![0, 0, 0, 1];
                buf.copy_from_slice(&data);
            }
            9 => {
                //choke
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
                let data: Vec<u8> = vec![0, 0, 0, 0]; // offset
                buf.copy_from_slice(&data);
            }
            14 => {
                let data: Vec<u8> = vec![2; CHUNK_SIZE as usize]; // block
                buf.copy_from_slice(&data);
            }
            _ => {
                //keep alive
                let data = vec![0, 0, 0, 0];
                println!("LLAMÓ DE MÁS A PEER DE INTEGRACION");
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
}
