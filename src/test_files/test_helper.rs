use crate::{
    errors::communication_method_error::CommunicationMethodError,
    peer_entities::communication_method::CommunicationMethod,
};
use std::io::{self, Cursor, Read, Write};

/// This struct is used to mock a TCP connection.
#[derive(Debug, Clone)]
pub struct MockTcpStream {
    pub cursor_r: Cursor<Vec<u8>>,
    pub cursor_w: Cursor<Vec<u8>>,
    pub is_connected: bool,
}

impl MockTcpStream {
    /// Creates a new MockTcpStream instance.
    pub fn new(vec_r: Vec<u8>) -> MockTcpStream {
        MockTcpStream {
            cursor_r: Cursor::new(vec_r),
            cursor_w: Cursor::new(Vec::new()),
            is_connected: false,
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
    /// Creates a new MockTcpStream instance with its corresponding fields to implement CommunicationMethod.
    fn create() -> Box<(dyn CommunicationMethod + Send)> {
        Box::new(MockTcpStream {
            cursor_r: Cursor::new(Vec::new()),
            cursor_w: Cursor::new(Vec::new()),
            is_connected: false,
        })
    }

    /// Change is_connected field to true.
    fn connect(&mut self, _ip: &str, _port: u16) -> Result<(), CommunicationMethodError> {
        self.is_connected = true;
        Ok(())
    }

    /// Write all the bytes in the buffer to the stream.
    fn write_all(&mut self, buf: &[u8]) -> Result<(), CommunicationMethodError> {
        let _ = self.write(buf);
        Ok(())
    }

    /// Read buf.len() bytes from the stream.
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), CommunicationMethodError> {
        let _ = self.cursor_r.read_exact(buf);
        Ok(())
    }

    /// Sets the read timeout for the stream.
    fn set_read_timeout(
        &mut self,
        _dur: Option<std::time::Duration>,
    ) -> Result<(), CommunicationMethodError> {
        Ok(())
    }

    /// Creates a new address for a peer.
    fn peer_addr(&self) -> Result<std::net::SocketAddr, CommunicationMethodError> {
        Ok(std::net::SocketAddr::new(
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),
            0,
        ))
    }

    /// Returns if the peer is connected.
    fn is_connected(&self) -> bool {
        self.is_connected
    }

    /// Disconnects the peer.
    fn disconnect(&mut self) -> bool {
        true
    }
}
