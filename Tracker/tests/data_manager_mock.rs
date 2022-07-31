// Imports data manager
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    sync::{Arc, Mutex},
};
use tracker::{errors::data_manager_error::DataManagerError, tracker::Tracker};

/// This struct is used to log the events of the program. It receives a DataMsg
pub struct DataManager {
    pub file: File,
    tracker: Option<Arc<Mutex<Tracker>>>,
}

/// This struct represents the message that is sent to the DataManager. It contains the message in string format.
impl DataManager {
    /// Creates a new DataManager instance.
    pub fn new(path: String) -> Result<DataManager, DataManagerError> {
        // Open file if exists, else create it
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(path)
            .unwrap();
        Ok(DataManager {
            file,
            tracker: None,
        })
    }

    pub fn save_tracker(&mut self) {
        if let Some(tracker) = &self.tracker {
            let tracker_copy = tracker.lock().unwrap().clone();
            let mut json = serde_json::to_string(&tracker_copy).unwrap();
            json.push('\n');
            let _ = self.file.seek(SeekFrom::Start(0));
            self.file.write_all(json.as_bytes()).unwrap();
            let _ = self.file.flush();
        }
    }

    /// Read file with serde and save tracker
    pub fn init_tracker(&mut self) -> Result<Arc<Mutex<Tracker>>, DataManagerError> {
        let mut contents = String::new();
        self.file.read_to_string(&mut contents).unwrap();
        if contents.is_empty() {
            self.tracker = Some(Arc::new(Mutex::new(Tracker {
                historical_torrents: vec![],
                historical_peers: HashMap::new(),
                torrents: HashMap::new(),
                new_changes: false,
            })));
        } else {
            let tracker = serde_json::from_str(&contents).unwrap();
            self.tracker = Some(Arc::new(Mutex::new(tracker)));
        }

        Ok(self.tracker.clone().unwrap())
    }
}
