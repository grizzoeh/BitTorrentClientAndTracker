use std::{
    io::{Read, Write},
    net::SocketAddr,
    time::Duration,
};

use crate::errors::communication_method_error::CommunicationMethodError;

pub trait CommunicationMethod {
    fn create() -> Box<dyn CommunicationMethod + Send>
    where
        Self: Sized;
    fn connect(&mut self, ip: &str, port: u16) -> Result<(), CommunicationMethodError>;
    fn peer_addr(&self) -> Result<SocketAddr, CommunicationMethodError>;
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), CommunicationMethodError>;
    fn write_all(&mut self, buf: &[u8]) -> Result<(), CommunicationMethodError>;
    fn set_read_timeout(&mut self, dur: Option<Duration>) -> Result<(), CommunicationMethodError>;
    fn is_connected(&self) -> bool;
}

pub struct TCP {
    pub(crate) stream: Option<std::net::TcpStream>,
}

impl CommunicationMethod for TCP {
    fn create() -> Box<(dyn CommunicationMethod + Send)> {
        Box::new(TCP { stream: None })
    }
    fn connect(&mut self, ip: &str, port: u16) -> Result<(), CommunicationMethodError> {
        let stream = std::net::TcpStream::connect((ip, port))?;
        self.stream = Some(stream);
        Ok(())
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), CommunicationMethodError> {
        let stream = match self.stream.as_mut() {
            Some(some) => some,
            None => {
                return Err(CommunicationMethodError::new(
                    "Stream is None, could be disconnected".to_string(),
                ))
            }
        };
        let _r = stream.read_exact(buf);
        Ok(())
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), CommunicationMethodError> {
        let stream = match self.stream.as_mut() {
            Some(some) => some,
            None => {
                return Err(CommunicationMethodError::new(
                    "Stream is None, could be disconnected".to_string(),
                ))
            }
        };
        let _r = stream.write_all(buf);
        Ok(())
    }

    fn set_read_timeout(&mut self, dur: Option<Duration>) -> Result<(), CommunicationMethodError> {
        let stream = match self.stream.as_mut() {
            Some(some) => some,
            None => {
                return Err(CommunicationMethodError::new(
                    "Stream is None, could be disconnected".to_string(),
                ))
            }
        };
        let _r = stream.set_read_timeout(dur);
        Ok(())
    }

    fn peer_addr(&self) -> Result<SocketAddr, CommunicationMethodError> {
        let stream = match self.stream.as_ref() {
            Some(some) => some,
            None => {
                return Err(CommunicationMethodError::new(
                    "Stream is None, could be disconnected".to_string(),
                ))
            }
        };
        let _r = stream.peer_addr()?;
        Ok(_r)
    }
    fn is_connected(&self) -> bool {
        self.stream.as_ref().is_some()
    }
}
