use crate::errors::logger_error::LoggerError;
use std::fs::File;
use std::io::Write;
use std::sync::mpsc::Receiver;

pub struct Logger {
    pub file: File,
    pub receiver: Receiver<LogMsg>,
}
#[derive(Debug, PartialEq, Clone)]
pub enum LogMsg {
    Info(String),
    End,
}

impl Logger {
    pub fn new(path: String, receiver: Receiver<LogMsg>) -> Result<Logger, LoggerError> {
        let file = File::create(path)?;

        Ok(Logger { file, receiver })
    }

    pub fn start(&mut self) -> Result<(), LoggerError> {
        loop {
            match self.receiver.recv() {
                Ok(LogMsg::Info(message)) => self.log(&message)?,
                Ok(LogMsg::End) => {
                    println!("Logger: End of transmission");
                    return Ok(());
                }
                Err(e) => {
                    self.log(&e.to_string())?;
                    return Err(LoggerError::from(e));
                }
            }
        }
    }

    pub fn log(&mut self, message: &str) -> Result<(), LoggerError> {
        self.file.write_all(format!("{message}\n").as_bytes())?;
        self.file.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{io::Read, sync::mpsc::channel, thread::spawn};

    use super::*;
    use std::sync::mpsc::Sender;
    // #[test]
    // fn test_logging_successfully() {
    //     // init logger
    //     let path = "src/logger_test_files/logs.txt".to_string();
    //     let (sender, receiver): (Sender<LogMsg>, Receiver<LogMsg>) = channel();
    //     let mut logger = Logger::new(path.clone(), receiver).unwrap();
    //     let _logger_handler = spawn(move || logger.start());
    //     // send message through sender and read it on the path
    //     let msg = "testing logger 123...".to_string();
    //     sender.send(LogMsg::Info(msg.clone())).unwrap();
    //     sender.send(LogMsg::End).unwrap();
    //     println!("join handle returned:{:?}", _logger_handler.join());
    //     let mut file = File::open(path).unwrap();
    //     let mut buf = Vec::new();
    //     file.read_to_end(&mut buf).unwrap();
    //     assert_eq!(String::from_utf8(buf).unwrap(), "testing logger 123...\n");
    // }
    #[test]
    fn test_logging_with_error() {
        // init logger
        let path = "src/logger_test_files/logs.txt".to_string();
        let (sender, receiver): (Sender<LogMsg>, Receiver<LogMsg>) = channel();
        let mut logger = Logger::new(path.clone(), receiver).unwrap();
        let _logger_handler = spawn(move || logger.start());
        // send message through sender and read it on the path
        drop(sender); // drop sender to cause error
        println!("join handle returned:{:?}", _logger_handler.join());
        let mut file = File::open(path).unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();
        assert_eq!(
            String::from_utf8(buf).unwrap(),
            "receiving on a closed channel\n"
        );
    }

    #[test]
    fn test_exiting_logger() {
        // init logger
        let path = "src/logger_test_files/logs.txt".to_string();
        let (sender, receiver): (Sender<LogMsg>, Receiver<LogMsg>) = channel();
        let mut logger = Logger::new(path.clone(), receiver).unwrap();
        let _logger_handler = spawn(move || logger.start());
        sender.send(LogMsg::End).unwrap(); // send end message to exit logger
        assert!(_logger_handler.join().is_ok());
    }
}
