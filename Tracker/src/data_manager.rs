use crate::{
    errors::data_manager_error::DataManagerError, logger::LogMsg, peer::Peer, tracker::Tracker,
};
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
        Arc, Mutex,
    },
};

/// This struct is used to log the events of the program. It receives a DataMsg
pub struct DataManager {
    pub file: File,
    tracker: Option<Arc<Mutex<Tracker>>>,
    shutdown_bool: Arc<AtomicBool>,
    pub logger_sender: Arc<Mutex<Sender<LogMsg>>>,
}

/// This struct represents the message that is sent to the DataManager. It contains the message in string format.
#[derive(Debug, PartialEq, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum DataMsg {
    Info((i64, Peer)), // (operation, peer)
    End,
}

impl DataManager {
    /// Creates a new DataManager instance.
    pub fn new(
        path: String,
        shutdown_bool: Arc<AtomicBool>,
        logger_sender: Sender<LogMsg>,
    ) -> Result<DataManager, DataManagerError> {
        // Open file if exists, else create it
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(path)?;
        Ok(DataManager {
            file,
            tracker: None,
            shutdown_bool,
            logger_sender: Arc::new(Mutex::new(logger_sender)),
        })
    }

    /// This function starts the DataManager listener.
    pub fn start(&mut self) -> Result<(), DataManagerError> {
        self.logger_sender
            .lock()?
            .send(LogMsg::Info("DataManager listener starting...".to_string()))?;
        while !self.shutdown_bool.load(Ordering::Relaxed) {
            // Every 10 seconds, save the tracker in the file
            std::thread::sleep(std::time::Duration::from_secs(10));

            if self.tracker.as_ref().unwrap().lock()?.new_changes {
                self.tracker.as_ref().unwrap().lock()?.new_changes = false;
                let _r = self.save_tracker();
            }
        }

        if self.tracker.as_ref().unwrap().lock()?.new_changes {
            let _r = self.save_tracker();
        }

        Ok(())
    }

    /// Saves the tracker data into the json file only if there is anything new to store.
    fn save_tracker(&mut self) -> Result<(), DataManagerError> {
        if let Some(tracker) = &self.tracker {
            self.logger_sender
                .lock()?
                .send(LogMsg::Info("Saving data into the json file".to_string()))?;

            let tracker_copy = tracker.lock()?.clone();
            let mut json = serde_json::to_string(&tracker_copy)?;
            json.push('\n');

            let _ = self.file.seek(SeekFrom::Start(0));
            self.file.write_all(json.as_bytes())?;
            let _ = self.file.flush();
        }
        Ok(())
    }

    /// Read file with serde and save tracker
    pub fn init_tracker(&mut self) -> Result<Arc<Mutex<Tracker>>, DataManagerError> {
        self.logger_sender.lock()?.send(LogMsg::Info(
            "Initializing tracker with json file data".to_string(),
        ))?;

        let mut contents = String::new();
        self.file.read_to_string(&mut contents)?;
        if contents.is_empty() {
            self.tracker = Some(Arc::new(Mutex::new(Tracker {
                historical_torrents: vec![],
                historical_peers: HashMap::new(),
                torrents: HashMap::new(),
                new_changes: false,
            })));
        } else {
            let tracker = serde_json::from_str(&contents)?;
            self.tracker = Some(Arc::new(Mutex::new(tracker)));
        }

        Ok(self.tracker.clone().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::{channel, Receiver, Sender};

    #[test]
    fn test_init_empty_tracker() {
        // init DataManager
        let path = "src/test_files/data_manager_test_files/test_data_empty.json".to_string();
        let (sender, _receiver): (Sender<LogMsg>, Receiver<LogMsg>) = channel();
        let mut data_manager = DataManager::new(
            path.clone(),
            Arc::new(AtomicBool::new(false)),
            sender.clone(),
        )
        .unwrap();
        // init Tracker
        let tracker = data_manager.init_tracker().unwrap();
        // check if tracker is empty
        assert_eq!(tracker.lock().unwrap().torrents.len(), 0);
    }

    #[test]
    fn test_init_tracker() {
        // init DataManager
        let path = "src/test_files/data_manager_test_files/test_data.json".to_string();
        let (sender, _receiver): (Sender<LogMsg>, Receiver<LogMsg>) = channel();
        let mut data_manager = DataManager::new(
            path.clone(),
            Arc::new(AtomicBool::new(false)),
            sender.clone(),
        )
        .unwrap();
        // init Tracker
        let tracker = data_manager.init_tracker().unwrap();
        // check if tracker has the correct data
        assert_eq!(tracker.lock().unwrap().torrents.len(), 1); // has only 1 torrent
        assert_eq!(
            // has only 1 peer
            tracker
                .lock()
                .unwrap()
                .torrents
                .get(&vec![1, 2, 3])
                .unwrap()
                .peers
                .len(),
            1
        );
        assert_eq!(
            // peer id is "pepe10"
            tracker
                .lock()
                .unwrap()
                .torrents
                .get(&vec![1, 2, 3])
                .unwrap()
                .peers
                .get(&"pepe10".to_string())
                .unwrap()
                .id,
            "pepe10".to_string()
        );
    }

    #[test]
    fn test_save_tracker() {
        // init DataManager
        let path = "src/test_files/data_manager_test_files/test_data.json".to_string();
        let (sender, _receiver): (Sender<LogMsg>, Receiver<LogMsg>) = channel();
        let mut data_manager = DataManager::new(
            path.clone(),
            Arc::new(AtomicBool::new(false)),
            sender.clone(),
        )
        .unwrap();
        // init Tracker
        let tracker = data_manager.init_tracker().unwrap();
        // save this value to check if it is saved correctly when changes are made
        let new_changes = tracker.lock().unwrap().new_changes;
        if new_changes {
            tracker.lock().unwrap().new_changes = false;
        } else {
            tracker.lock().unwrap().new_changes = true;
        };
        // save tracker
        let _ = data_manager.save_tracker();
        // check if file has the correct data
        let (sender2, _receiver2): (Sender<LogMsg>, Receiver<LogMsg>) = channel();
        let mut data_manager2 = DataManager::new(
            path.clone(),
            Arc::new(AtomicBool::new(false)),
            sender2.clone(),
        )
        .unwrap();
        let tracker2 = data_manager2.init_tracker().unwrap();
        // check if tracker new_changes is changed
        assert_ne!(tracker2.lock().unwrap().new_changes, new_changes);
    }
}
