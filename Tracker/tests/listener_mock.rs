use std::{
    fs,
    sync::{mpsc::Sender, Arc, Mutex},
};
use tracker::{
    announce_utils::parse_announce,
    announce_utils::URLParams,
    errors::listener_error::ListenerError,
    logger::LogMsg,
    tracker::{Tracker, TrackerInterface},
};

pub struct Listener {
    pub port: u16,
    pub ip: String,
    pub logger_sender: Arc<Mutex<Sender<LogMsg>>>,
}

impl Listener {
    pub fn new(port: u16, ip: String, logger_sender: Sender<LogMsg>) -> Listener {
        Listener {
            port,
            ip,
            logger_sender: Arc::new(Mutex::new(logger_sender)),
        }
    }

    pub fn listen(self, announce_url: String, tracker: Arc<Mutex<Tracker>>) {
        self.logger_sender
            .lock()
            .unwrap()
            .send(LogMsg::Info(
                "Listener started waiting for requests...".to_string(),
            ))
            .unwrap();

        let logger = self.logger_sender.clone();
        let tracker_copy = tracker.clone();
        let _code = handle_announce(announce_url, self.ip, logger, tracker_copy);
    }
}

pub fn handle_announce(
    announce_url: String,
    peer_ip: String,
    logger_sender: Arc<Mutex<Sender<LogMsg>>>,
    tracker: Arc<Mutex<Tracker>>,
) -> Result<(), ListenerError> {
    let mut _response: Option<String> = None;
    // PARA PROBAR -> "GET /announce?info_hash=%b1%11%81%3c%e6%0fB%91%974%82%3d%f5%ec%20%bd%1e%04%e7%f7&peer_id=556816578366975432pasaq 10&port=443&uploaded=0&downloaded=0&left=0&event=started&numwant=100 HTTP/1.1\r\n".to_string()
    let mut announce_dic = parse_announce(announce_url)?;
    announce_dic.insert("ip".to_string(), URLParams::String(peer_ip));
    let peer_list = tracker
        .lock()
        .unwrap()
        .handle_announce(announce_dic, logger_sender.clone())
        .unwrap();

    _response = Some(format!(
        "{}Content-Length: {}\r\n\r\n{:?}",
        "HTTP/1.1 200 OK\r\n",
        peer_list.len(),
        peer_list
    ));

    logger_sender
        .lock()
        .unwrap()
        .send(LogMsg::Info("Announce requested".to_string()))
        .unwrap();
    let (status, filename) = ("HTTP/1.1 200 OK\r\n", "_");

    if _response.is_none() {
        let content = fs::read_to_string(filename)?;
        _response = Some(format!(
            "{}Content-Length: {}\r\n\r\n{}",
            status,
            content.len(),
            content
        ));
    }

    logger_sender
        .lock()
        .unwrap()
        .send(LogMsg::Info(format!(
            "Response Status: {}",
            status.split("\r\n").next().unwrap()
        )))
        .unwrap();
    Ok(())
}
