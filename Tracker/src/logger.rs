use crate::errors::logger_error::LoggerError;
use std::{fs::File, io::Write, sync::mpsc::Receiver};

/// This struct is used to log the events of the program. It receives a LogMsg
pub struct Logger {
    pub file: File,
    pub receiver: Receiver<LogMsg>,
}

/// This struct represents the message that is sent to the logger. It contains the message in string format.
#[derive(Debug, PartialEq, Clone)]
pub enum LogMsg {
    Info(String),
    End,
}

impl Logger {
    /// Creates a new logger instance.
    pub fn new(path: String, receiver: Receiver<LogMsg>) -> Result<Logger, LoggerError> {
        let file = File::create(path)?;

        Ok(Logger { file, receiver })
    }

    /// This function starts the logger listener.
    pub fn start(&mut self) -> Result<(), LoggerError> {
        loop {
            match self.receiver.recv() {
                Ok(LogMsg::Info(message)) => self.log(&message)?,
                Ok(LogMsg::End) => {
                    self.log("Logger: End of transmission")?;
                    return Ok(());
                }
                Err(e) => {
                    self.log(&e.to_string())?;
                    return Err(LoggerError::from(e));
                }
            }
        }
    }

    /// Writes the message into the log file.
    fn log(&mut self, message: &str) -> Result<(), LoggerError> {
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
    #[test]
    fn test_logging_successfully() {
        // Init logger
        let path = "src/test_files/logger_test_files/logs.txt".to_string();
        let (sender, receiver): (Sender<LogMsg>, Receiver<LogMsg>) = channel();
        let mut logger = Logger::new(path.clone(), receiver).unwrap();
        let logger_handler = spawn(move || logger.start());
        // Send message through sender and read it on the path
        let msg = "testing logger 123...".to_string();
        sender.send(LogMsg::Info(msg.clone())).unwrap();
        sender.send(LogMsg::End).unwrap();
        let _r = logger_handler.join();
        let mut file = File::open(path).unwrap();
        // Buffer with first line of file
        let mut buffer = String::new();
        file.read_to_string(&mut buffer).unwrap();

        assert_eq!(
            buffer,
            "testing logger 123...\nLogger: End of transmission\n"
        );
    }

    #[test]
    fn test_logging_with_error() {
        // Init logger
        let path = "src/test_files/logger_test_files/logs2.txt".to_string();
        let (sender, receiver): (Sender<LogMsg>, Receiver<LogMsg>) = channel();
        let mut logger = Logger::new(path.clone(), receiver).unwrap();
        let logger_handler = spawn(move || logger.start());
        // Send message through sender and read it on the path
        drop(sender); // Drop sender to cause error
        let _r = logger_handler.join();
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
        // Init logger
        let path = "src/test_files/logger_test_files/logs3.txt".to_string();
        let (sender, receiver): (Sender<LogMsg>, Receiver<LogMsg>) = channel();
        let mut logger = Logger::new(path.clone(), receiver).unwrap();
        let _logger_handler = spawn(move || logger.start());
        sender.send(LogMsg::End).unwrap(); // Send end message to exit logger
        assert!(_logger_handler.join().is_ok());
    }
}
