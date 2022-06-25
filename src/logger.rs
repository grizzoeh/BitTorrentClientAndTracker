use crate::errors::logger_error::LoggerError;
use std::fs::File;
use std::io::Write;
use std::sync::mpsc::Receiver;

pub struct Logger {
    pub file: File,
    pub receiver: Receiver<String>,
}

impl Logger {
    pub fn new(path: String, receiver: Receiver<String>) -> Result<Logger, LoggerError> {
        let file = File::create(path).unwrap();
        Ok(Logger { file, receiver })
    }
    pub fn log(&mut self, message: &str) {
        let _ = match self.file.write(format!("{}\n", message).as_bytes()) {
            Ok(_) => return,
            Err(e) => self.log_error(e.to_string().as_str()),
        };
    }

    fn log_error(&mut self, message: &str) {
        if self
            .file
            .write(format!("{}\n", message).as_bytes())
            .is_err()
        {}
    }

    pub fn start(&mut self) {
        loop {
            let msg = self.receiver.recv().unwrap();
            let _ = self.log(msg.as_str());
        }
    }
}
