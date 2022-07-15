use crate::errors::communication_method_error::CommunicationMethodError;
use std::{
    io::{Read, Write},
    net::{IpAddr, Ipv4Addr, Shutdown, SocketAddr},
    time::Duration,
};

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
    fn disconnect(&mut self) -> bool;
}

pub struct TCP {
    pub(crate) stream: Option<std::net::TcpStream>,
}

impl CommunicationMethod for TCP {
    /// This function creates a new TCP communication method.
    fn create() -> Box<(dyn CommunicationMethod + Send)> {
        Box::new(TCP { stream: None })
    }

    /// Connects to a peer given using its port, ip and stream created when create() was called.
    fn connect(&mut self, ip: &str, port: u16) -> Result<(), CommunicationMethodError> {
        if ip.split('.').count() == 1 {
            //ipv6 case
            // let vec: Vec<u16>;
            // vec = ip
            //     .split(':')
            //     .collect::<Vec<&str>>()
            //     .iter()
            //     .map(|x| x.parse::<u16>().unwrap())
            //     .collect::<Vec<u16>>();
            // let socket = SocketAddr::new(
            //     IpAddr::V6(Ipv6Addr::new(
            //         vec[0], vec[1], vec[2], vec[3], vec[4], vec[5], vec[6], vec[7],
            //     )),
            //     port,
            // );
            // let stream =
            //     std::net::TcpStream::connect_timeout(&socket, std::time::Duration::from_secs(20))?;
            // self.stream = Some(stream);
            return Err(CommunicationMethodError::new(
                "IPv6 not supported yet".to_string(),
            ));
        } else {
            //ipv4 case
            let vec: Vec<u8> = ip
                .split('.')
                .collect::<Vec<&str>>()
                .iter()
                .map(|x| x.parse::<u8>().unwrap())
                .collect::<Vec<u8>>();
            let socket = SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(vec[0], vec[1], vec[2], vec[3])),
                port,
            );
            let stream =
                std::net::TcpStream::connect_timeout(&socket, std::time::Duration::from_secs(15))?;
            self.stream = Some(stream);
        }
        Ok(())
    }

    /// This function reads buf size bytes from the stream.
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

    /// This function writes all the buf content on the stream.
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

    /// This function sets the read timeout of the stream.
    fn set_read_timeout(&mut self, dur: Option<Duration>) -> Result<(), CommunicationMethodError> {
        let stream = match self.stream.as_mut() {
            Some(some) => some,
            None => {
                return Err(CommunicationMethodError::new(
                    "Stream is None, could be disconnected".to_string(),
                ))
            }
        };
        let _r = stream.set_read_timeout(dur)?;
        Ok(())
    }

    /// Returns the SocketAddres of the current stream.
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

    /// Returns true if the stream is connected, false otherwise.
    fn is_connected(&self) -> bool {
        self.stream.as_ref().is_some()
    }

    /// Close connection with the peer. Returns true if the connection was closed, false otherwise.
    fn disconnect(&mut self) -> bool {
        if let Some(stream) = self.stream.as_mut() {
            let _r = stream.shutdown(Shutdown::Both);
            true
        } else {
            false
        }
    }
}
