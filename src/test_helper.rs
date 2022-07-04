use std::io::{self, Cursor, Read, Write};
use std::time::Duration;

use crate::communication_method::CommunicationMethod;
use crate::constants::{CHUNK_SIZE, PSTR, RESERVED_SPACE_LEN};
use crate::errors::communication_method_error::CommunicationMethodError;
use crate::utils::UiParams;

#[derive(Debug, Clone)]
pub struct MockTcpStream {
    pub cursor_r: Cursor<Vec<u8>>,
    pub cursor_w: Cursor<Vec<u8>>,
    pub is_connected: bool,
}

impl MockTcpStream {
    pub fn new(vec_r: Vec<u8>) -> MockTcpStream {
        MockTcpStream {
            cursor_r: Cursor::new(vec_r),
            cursor_w: Cursor::new(Vec::new()),
            is_connected: true,
        }
    }
}

impl Read for MockTcpStream {
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), io::Error> {
        self.cursor_r.read_exact(buf)
    }
    fn read(&mut self, _buf: &mut [u8]) -> Result<usize, io::Error> {
        self.cursor_r.read(_buf)
    }
}

impl Write for MockTcpStream {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        self.cursor_w.write(buf)
    }
    fn flush(&mut self) -> Result<(), io::Error> {
        self.cursor_w.flush()
    }
}

impl CommunicationMethod for MockTcpStream {
    fn create() -> Box<(dyn CommunicationMethod + Send)> {
        Box::new(MockTcpStream {
            cursor_r: Cursor::new(Vec::new()),
            cursor_w: Cursor::new(Vec::new()),
            is_connected: false,
        })
    }

    fn connect(&mut self, _ip: &str, _port: u16) -> Result<(), CommunicationMethodError> {
        self.is_connected = true;
        Ok(())
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), CommunicationMethodError> {
        let _ = self.cursor_w.write(buf);
        Ok(())
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), CommunicationMethodError> {
        let _ = self.cursor_r.read_exact(buf);
        Ok(())
    }

    fn set_read_timeout(
        &mut self,
        _dur: Option<std::time::Duration>,
    ) -> Result<(), CommunicationMethodError> {
        Ok(())
    }
    fn peer_addr(&self) -> Result<std::net::SocketAddr, CommunicationMethodError> {
        Ok(std::net::SocketAddr::new(
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),
            0,
        ))
    }
    fn is_connected(&self) -> bool {
        self.is_connected
    }
}

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
                // copy data to buf
                buf[0..data.len()].copy_from_slice(&data);
            }
            1 => {
                let mut data = PSTR.as_bytes().to_vec();
                data.extend(vec![0; RESERVED_SPACE_LEN as usize]);
                buf[0..data.len()].copy_from_slice(&data);
            }

            2 => {
                let data = vec![5; 20];
                buf[0..data.len()].copy_from_slice(&data);
            }
            3 => {
                let data = "default".as_bytes();
                buf[0..data.len()].copy_from_slice(data);
            }
            4 => {
                //bitfield
                let data: Vec<u8> = vec![0, 0, 0, 2]; // largo 2, id 5, un 1 porq tiene la 1
                buf[0..data.len()].copy_from_slice(&data);
            }
            5 => {
                let data: [u8; 1] = [5];
                buf[0..data.len()].copy_from_slice(&data);
            }
            6 => {
                let data: [u8; 1] = [1];
                buf[0..data.len()].copy_from_slice(&data);
            }
            7 => {
                //unchoke
                let data: Vec<u8> = vec![0, 0, 0, 1];
                buf[0..data.len()].copy_from_slice(&data);
            }
            8 => {
                //choke
                let data: Vec<u8> = vec![1];
                buf[0..data.len()].copy_from_slice(&data);
            }
            9 => {
                //piece
                let data: Vec<u8> = vec![0, 0, 64, 9].to_vec();
                buf[0..data.len()].copy_from_slice(&data);
            }
            10 => {
                let data: Vec<u8> = vec![7]; // piece message id
                buf[0..data.len()].copy_from_slice(&data);
            }
            11 => {
                let data: Vec<u8> = vec![0, 0, 0, 0]; // index
                buf[0..data.len()].copy_from_slice(&data);
            }
            12 => {
                let data: Vec<u8> = vec![0, 0, 0, 0]; // offset
                buf[0..data.len()].copy_from_slice(&data);
            }
            13 => {
                let data: Vec<u8> = vec![0; CHUNK_SIZE as usize]; // block
                buf[0..data.len()].copy_from_slice(&data);
            }
            _ => {
                //keep alive
                let data = vec![0, 0, 0, 0];
                println!("LLAMÓ DE MÁS A PEER DE INTEGRACION");
                buf[0..data.len()].copy_from_slice(&data);
            }
        }
        self.counter += 1;
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
                // copy data to buf
                buf[0..data.len()].copy_from_slice(&data);
            }
            1 => {
                let mut data = PSTR.as_bytes().to_vec();
                data.extend(vec![0; RESERVED_SPACE_LEN as usize]);
                buf[0..data.len()].copy_from_slice(&data);
            }

            2 => {
                let data = vec![5; 20];
                buf[0..data.len()].copy_from_slice(&data);
            }
            3 => {
                let data = "default".as_bytes();
                buf[0..data.len()].copy_from_slice(data);
            }
            4 => {
                //bitfield
                let data: Vec<u8> = vec![0, 0, 0, 2]; // largo 2, id 5, un 2 porq tiene la 2
                buf[0..data.len()].copy_from_slice(&data);
            }
            5 => {
                let data: [u8; 1] = [5];
                buf[0..data.len()].copy_from_slice(&data);
            }
            6 => {
                let data: [u8; 1] = [2];
                buf[0..data.len()].copy_from_slice(&data);
            }
            7 => {
                //unchoke
                let data: Vec<u8> = vec![0, 0, 0, 1];
                buf[0..data.len()].copy_from_slice(&data);
            }
            8 => {
                //choke
                let data: Vec<u8> = vec![1];
                buf[0..data.len()].copy_from_slice(&data);
            }
            9 => {
                //piece
                let data: Vec<u8> = vec![0, 0, 64, 9].to_vec();
                buf[0..data.len()].copy_from_slice(&data);
            }
            10 => {
                let data: Vec<u8> = vec![7]; // piece message id
                buf[0..data.len()].copy_from_slice(&data);
            }
            11 => {
                let data: Vec<u8> = vec![0, 0, 0, 0]; // index
                buf[0..data.len()].copy_from_slice(&data);
            }
            12 => {
                let data: Vec<u8> = vec![0, 0, 0, 0]; // offset
                buf[0..data.len()].copy_from_slice(&data);
            }
            13 => {
                let data: Vec<u8> = vec![1; CHUNK_SIZE as usize]; // block
                buf[0..data.len()].copy_from_slice(&data);
            }
            _ => {
                //keep alive
                let data = vec![0, 0, 0, 0];
                println!("LLAMÓ DE MÁS A PEER DE INTEGRACION");
                buf[0..data.len()].copy_from_slice(&data);
            }
        }
        self.counter += 1;
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
                // copy data to buf
                buf[0..data.len()].copy_from_slice(&data);
            }
            1 => {
                let mut data = PSTR.as_bytes().to_vec();
                data.extend(vec![0; RESERVED_SPACE_LEN as usize]);
                buf[0..data.len()].copy_from_slice(&data);
            }

            2 => {
                let data = vec![5; 20];
                buf[0..data.len()].copy_from_slice(&data);
            }
            3 => {
                let data = "default".as_bytes();
                buf[0..data.len()].copy_from_slice(data);
            }
            4 => {
                //bitfield
                let data: Vec<u8> = vec![0, 0, 0, 2]; // largo 2, id 5, un 4 porq tiene la 3
                buf[0..data.len()].copy_from_slice(&data);
            }
            5 => {
                let data: [u8; 1] = [5];
                buf[0..data.len()].copy_from_slice(&data);
            }
            6 => {
                let data: [u8; 1] = [4];
                buf[0..data.len()].copy_from_slice(&data);
            }
            7 => {
                //unchoke
                let data: Vec<u8> = vec![0, 0, 0, 1];
                buf[0..data.len()].copy_from_slice(&data);
            }
            8 => {
                //choke
                let data: Vec<u8> = vec![1];
                buf[0..data.len()].copy_from_slice(&data);
            }
            9 => {
                //piece
                let data: Vec<u8> = vec![0, 0, 64, 9].to_vec();
                buf[0..data.len()].copy_from_slice(&data);
            }
            10 => {
                let data: Vec<u8> = vec![7]; // piece message id
                buf[0..data.len()].copy_from_slice(&data);
            }
            11 => {
                let data: Vec<u8> = vec![0, 0, 0, 0]; // index
                buf[0..data.len()].copy_from_slice(&data);
            }
            12 => {
                let data: Vec<u8> = vec![0, 0, 0, 0]; // offset
                buf[0..data.len()].copy_from_slice(&data);
            }
            13 => {
                let data: Vec<u8> = vec![2; CHUNK_SIZE as usize]; // block
                buf[0..data.len()].copy_from_slice(&data);
            }
            _ => {
                //keep alive
                let data = vec![0, 0, 0, 0];
                println!("LLAMÓ DE MÁS A PEER DE INTEGRACION");
                buf[0..data.len()].copy_from_slice(&data);
            }
        }
        self.counter += 1;
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

fn _ui_listener_aux(receiver_ui: glib::Receiver<Vec<(usize, UiParams, String)>>) {
    let _r = receiver_ui.attach(None, move |msg| {
        let code = &msg[0].0;
        if *code == 121 {
            glib::Continue(false)
        } else {
            glib::Continue(true)
        }
    });
}
