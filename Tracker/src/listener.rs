use crate::{
    announce_utils::{parse_announce, URLParams},
    constants::*,
    errors::listener_error::ListenerError,
    logger::LogMsg,
    thread_pool::ThreadPool,
    tracker::{Tracker, TrackerInterface},
};
use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
        Arc, Mutex,
    },
};

/// Listens for incoming connections and handles them.
pub struct Listener {
    pub listener: TcpListener,
    pub port: u16,
    pub ip: String,
    pub logger_sender: Arc<Mutex<Sender<LogMsg>>>,
    shutdown_bool: Arc<AtomicBool>,
}

impl Listener {
    pub fn new(
        port: u16,
        ip: String,
        logger_sender: Sender<LogMsg>,
        shutdown_bool: Arc<AtomicBool>,
    ) -> Result<Listener, ListenerError> {
        Ok(Listener {
            listener: TcpListener::bind(format!("{}:{}", ip, port).as_str())?,
            port,
            ip,
            logger_sender: Arc::new(Mutex::new(logger_sender)),
            shutdown_bool,
        })
    }

    /// Starts to listen to incoming connections.
    pub fn listen(self, tracker: Arc<Mutex<Tracker>>) -> Result<(), ListenerError> {
        self.logger_sender.lock()?.send(LogMsg::Info(
            "Listener started waiting for requests...".to_string(),
        ))?;

        self.listener.set_nonblocking(true)?;
        let pool = ThreadPool::new(8);

        while !self.shutdown_bool.load(Ordering::Relaxed) {
            match self.listener.accept() {
                Ok((stream, _addr)) => {
                    let logger = self.logger_sender.clone();
                    let tracker_copy = tracker.clone();
                    pool.execute(move || {
                        let _code = handle_connection(stream, logger, tracker_copy);
                    });
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(std::time::Duration::from_secs(ACCEPT_SLEEP_TIME));
                }
                Err(e) => {
                    self.logger_sender
                        .lock()?
                        .send(LogMsg::Info(format!("Error accepting connection: {}", e)))?;
                }
            }
        }
        Ok(())
    }
}

/// Starts the process of incoming connection, reading the values if /announce was requested.
fn handle_connection(
    mut stream: TcpStream,
    logger_sender: Arc<Mutex<Sender<LogMsg>>>,
    tracker: Arc<Mutex<Tracker>>,
) -> Result<(), ListenerError> {
    let mut buffer = [0; 1024];
    let _ = stream.read(&mut buffer)?;
    let mut response: Option<Vec<u8>> = None;
    let (status, filename) = if buffer.starts_with(GET_STATS) {
        logger_sender
            .lock()?
            .send(LogMsg::Info("Stats requested".to_string()))?;
        ("HTTP/1.1 200 OK\r\n", "src/pages/stats.html")
    } else if buffer.starts_with(GET_STATS_JS) {
        logger_sender
            .lock()?
            .send(LogMsg::Info("JS file requested".to_string()))?;
        ("HTTP/1.1 200 OK\r\n", "src/pages/stats.js")
    } else if buffer.starts_with(GET_ANNOUNCE) {
        let mut announce_dic = parse_announce(String::from_utf8(buffer.to_vec())?)?;

        let peer_ip = stream.peer_addr()?.ip().to_string();
        announce_dic.insert("ip".to_string(), URLParams::String(peer_ip));
        let peer_list = tracker
            .lock()?
            .handle_announce(announce_dic, logger_sender.clone())?;

        response = Some(
            format!(
                "{}Content-Length: {}\r\n\r\n",
                "HTTP/1.1 200 OK\r\n",
                peer_list.len()
            )
            .as_bytes()
            .to_vec(),
        );
        response.as_mut().unwrap().extend(peer_list);

        logger_sender
            .lock()?
            .send(LogMsg::Info("Announce requested".to_string()))?;
        ("HTTP/1.1 200 OK\r\n", "_")
    } else if buffer.starts_with(GET_STATS_DATA) {
        logger_sender
            .lock()?
            .send(LogMsg::Info("Stats data requested".to_string()))?;
        let data = tracker.lock()?.get_stats_data()?;
        response = Some(
            format!(
                "{}Content-Length: {}\r\n\r\n{}",
                "HTTP/1.1 200 OK\r\n",
                data.len(),
                data
            )
            .as_bytes()
            .to_vec(),
        );
        ("HTTP/1.1 200 OK\r\n", "_")
    } else {
        logger_sender
            .lock()?
            .send(LogMsg::Info("Page Not Found".to_string()))?;
        ("HTTP/1.1 404 NOT FOUND\r\n", "src/pages/not_found.html")
    };

    if response.is_none() {
        let content = fs::read_to_string(filename)?;
        response = Some(
            format!(
                "{}Content-Length: {}\r\n\r\n{}",
                status,
                content.len(),
                content
            )
            .as_bytes()
            .to_vec(),
        );
    }

    let _ = stream.write(response.unwrap().as_ref())?;
    stream.flush()?;
    logger_sender.lock()?.send(LogMsg::Info(format!(
        "Response Status: {}",
        status.split("\r\n").next().unwrap()
    )))?;
    Ok(())
}
