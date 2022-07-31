mod data_manager_mock;
#[cfg(test)]
mod listener_mock;
mod tests {
    use crate::data_manager_mock::*;
    use crate::listener_mock::*;

    use tracker::announce_utils::decode_peer_list;
    use tracker::announce_utils::response_splitter;
    use tracker::app::initialize_app;
    use tracker::bdecoder::bdecode;
    use tracker::constants::DATA_MANAGER_PATH;
    use tracker::tracker::Tracker;
    use tracker::{
        constants::{LISTENER_IP, LISTENER_PORT},
        logger::{LogMsg, Logger},
    };

    use std::fs::File;
    use std::io::Read;

    use std::io::Write;
    use std::net::TcpStream;
    use std::thread;
    use std::time::Duration;
    use std::{sync::mpsc::channel, thread::spawn};

    #[test]
    fn test_receive_tracker_response() {
        const PATH: &str = "tests/data/test_receive_tracker_response.json";
        // truncate the data manager file
        let data_manager_file = File::create(PATH).unwrap();
        drop(data_manager_file);
        // init Tracker
        let _r = spawn(move || initialize_app(PATH));
        thread::sleep(Duration::from_secs(3)); // sleep to wait for Tracker to start

        // connect 1st peer to Tracker
        let mut stream1 = TcpStream::connect(format!("{}:{}", LISTENER_IP, LISTENER_PORT)).unwrap();

        // send announce to Tracker ( 1st time to save the peer, 2nd time to get the peer list)
        let urlencoded_infohash = to_urlencoded(&[1, 2, 3]);
        let buf1 = format!("GET /announce?info_hash={}&peer_id=juan&port=8088&uploaded=0&downloaded=0&left=0&event=completed&numwant=50 HTTP/1.1\r\nHost: {}\r\n\r\n", urlencoded_infohash, "localhost:8088");

        stream1.write_all(buf1.as_bytes()).unwrap();
        thread::sleep(Duration::from_secs(5)); // sleep to wait for Tracker to receive announce

        let mut response_buf = Vec::new();
        let _ = stream1.read_to_end(&mut response_buf).unwrap();

        // connect 2nd peer to Tracker
        let mut stream2 = TcpStream::connect(format!("{}:{}", LISTENER_IP, LISTENER_PORT)).unwrap();

        let buf2 = format!("GET /announce?info_hash={}&peer_id=pepe&port=8088&uploaded=0&downloaded=0&left=0&event=completed&numwant=50 HTTP/1.1\r\nHost: {}\r\n\r\n", urlencoded_infohash, "localhost:8088");
        stream2.write_all(buf2.as_bytes()).unwrap();
        thread::sleep(Duration::from_secs(5)); // sleep to wait for Tracker to receive announce

        // receive response from Tracker
        let mut response_buf2 = Vec::new();
        let _ = stream2.read_to_end(&mut response_buf2).unwrap();
        let d: &[u8] = response_splitter(response_buf2.as_ref());

        // decode response and check if it's correct
        let response = bdecode(d).unwrap();
        let peer_list = decode_peer_list(vec![1, 2, 3], response);
        assert_eq!(peer_list.len(), 1);
        assert_eq!(peer_list[0].id, "juan");

        // write something to stdin to end the tracker
        let mut stdout = std::io::stdout();
        let mut stdout_buf = "terminando...".as_bytes().to_vec();
        stdout.write(&mut stdout_buf).unwrap();

        // check if data saved in json file is correct
        thread::sleep(Duration::from_secs(5));
        let mut data_manager = DataManager::new(DATA_MANAGER_PATH.to_string()).unwrap();
        let tracker = data_manager.init_tracker().unwrap();
        assert_eq!(
            tracker
                .lock()
                .unwrap()
                .torrents
                .get(&vec![1, 2, 3])
                .unwrap()
                .peers
                .len(),
            2
        );
        assert_eq!(
            tracker.lock().unwrap().torrents[&vec![1, 2, 3]].peers["pepe"].id,
            "pepe"
        );
    }

    #[test]
    fn integration_test_update_and_store() {
        pub const DATA_MANAGER_TEST_PATH: &str = "tests/data/tracker_data_test.json";
        pub const LOG_PATH_TEST: &str = "tests/reports/log_test.txt";

        let mut data_manager = DataManager::new(DATA_MANAGER_TEST_PATH.to_string()).unwrap();

        let tracker = data_manager.init_tracker().expect("Failed to init tracker");

        let (logger_sender, logger_receiver) = channel();
        let mut logger = Logger::new(LOG_PATH_TEST.to_string(), logger_receiver).unwrap();

        let handle_logger = spawn(move || {
            let _r = logger.start();
        });

        let listener = Listener::new(
            LISTENER_PORT,
            LISTENER_IP.to_string(),
            logger_sender.clone(),
        );

        let announce_url = "GET /announce?info_hash=%b1%11%81%3c%e6%0fB%91%974%82%3d%f5%ec%20%bd%1e%04%e7%f7&peer_id=556816578366975432pasaq 10&port=443&uploaded=0&downloaded=0&left=0&event=started&numwant=20 HTTP/1.1\r\n".to_string();

        let tracker_copy = tracker.clone();
        let _r = listener.listen(announce_url.to_string(), tracker_copy);

        data_manager.save_tracker();

        let mut contents = String::new();

        let mut file = File::open(DATA_MANAGER_TEST_PATH).unwrap();
        file.read_to_string(&mut contents).unwrap();

        let tracker: Tracker = serde_json::from_str(&contents).unwrap();

        assert!(tracker.historical_peers.len() >= 1);

        let infohash1: Vec<u8> = [
            177, 17, 129, 60, 230, 15, 66, 145, 151, 52, 130, 61, 245, 236, 32, 189, 30, 4, 231,
            247,
        ]
        .to_vec();

        assert!(tracker.historical_peers[&infohash1]["connected"].len() >= 1);

        assert_eq!(
            tracker.torrents[&infohash1].peers["556816578366975432pasaq 10"].numwant,
            20 as u32
        );
        logger_sender.send(LogMsg::End).unwrap();
        handle_logger.join().unwrap();
    }

    #[test]
    fn integration_test_completed() {
        pub const DATA_MANAGER_TEST_PATH: &str = "tests/data/tracker_data_test2.json";
        pub const LOG_PATH_TEST: &str = "tests/reports/log_test.txt";

        let mut data_manager = DataManager::new(DATA_MANAGER_TEST_PATH.to_string()).unwrap();

        let tracker = data_manager.init_tracker().expect("Failed to init tracker");

        let (logger_sender, logger_receiver) = channel();
        let mut logger = Logger::new(LOG_PATH_TEST.to_string(), logger_receiver).unwrap();

        let handle_logger = spawn(move || {
            let _r = logger.start();
        });

        let listener = Listener::new(
            LISTENER_PORT,
            LISTENER_IP.to_string(),
            logger_sender.clone(),
        );

        let announce_url = "GET /announce?info_hash=%b1%11%81%3c%e6%0fB%91%974%82%3d%f5%ec%20%bd%1e%04%e7%f7&peer_id=556816578366975432pasaq 10&port=443&uploaded=0&downloaded=100&left=0&event=completed&numwant=20 HTTP/1.1\r\n".to_string();

        let tracker_copy = tracker.clone();
        let _r = listener.listen(announce_url.to_string(), tracker_copy);

        data_manager.save_tracker();

        let mut contents = String::new();

        let mut file = File::open(DATA_MANAGER_TEST_PATH).unwrap();
        file.read_to_string(&mut contents).unwrap();

        let tracker: Tracker = serde_json::from_str(&contents).unwrap();

        assert!(tracker.historical_peers.len() >= 1);

        let infohash1: Vec<u8> = [
            177, 17, 129, 60, 230, 15, 66, 145, 151, 52, 130, 61, 245, 236, 32, 189, 30, 4, 231,
            247,
        ]
        .to_vec();

        assert!(tracker.torrents[&infohash1].peers["556816578366975432pasaq 10"].completed);

        logger_sender.send(LogMsg::End).unwrap();
        handle_logger.join().unwrap();
    }

    #[test]
    fn integration_test_stopped_disconnected() {
        pub const DATA_MANAGER_TEST_PATH: &str = "tests/data/tracker_data_test3.json";
        pub const LOG_PATH_TEST: &str = "tests/reports/log_test.txt";

        let mut data_manager = DataManager::new(DATA_MANAGER_TEST_PATH.to_string()).unwrap();

        let tracker = data_manager.init_tracker().expect("Failed to init tracker");

        let (logger_sender, logger_receiver) = channel();
        let mut logger = Logger::new(LOG_PATH_TEST.to_string(), logger_receiver).unwrap();

        let handle_logger = spawn(move || {
            let _r = logger.start();
        });
        let listener = Listener::new(
            LISTENER_PORT,
            LISTENER_IP.to_string(),
            logger_sender.clone(),
        );

        let announce_url = "GET /announce?info_hash=%b1%11%81%3c%e6%0fB%91%974%82%3d%f5%ec%20%bd%1e%04%e7%f7&peer_id=556816578366975432pasaq 10&port=443&uploaded=0&downloaded=10&left=0&event=stopped&numwant=20 HTTP/1.1\r\n".to_string();

        let tracker_copy = tracker.clone();
        let _r = listener.listen(announce_url.to_string(), tracker_copy);

        data_manager.save_tracker();

        let mut contents = String::new();

        let mut file = File::open(DATA_MANAGER_TEST_PATH).unwrap();
        file.read_to_string(&mut contents).unwrap();

        let tracker: Tracker = serde_json::from_str(&contents).unwrap();

        assert!(tracker.historical_peers.len() >= 1);

        let infohash1: Vec<u8> = [
            177, 17, 129, 60, 230, 15, 66, 145, 151, 52, 130, 61, 245, 236, 32, 189, 30, 4, 231,
            247,
        ]
        .to_vec();

        assert!(!tracker.torrents[&infohash1].peers["556816578366975432pasaq 10"].connected);

        logger_sender.send(LogMsg::End).unwrap();
        handle_logger.join().unwrap();
    }

    /// Converts a vector of u8 to a urlencoded string.
    pub fn to_urlencoded(bytes: &[u8]) -> String {
        bytes
            .iter()
            .map(|b| {
                if b.is_ascii_alphanumeric() || *b == b'.' || *b == b'-' || *b == b'_' || *b == b'~'
                {
                    String::from(*b as char)
                } else {
                    format!("%{:02x}", *b)
                }
            })
            .collect()
    }
}
