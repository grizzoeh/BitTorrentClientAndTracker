use std::sync::{atomic::AtomicBool, Arc};

pub struct TrackerController {
    shutdown_bool: Arc<AtomicBool>,
    handles: Vec<Option<std::thread::JoinHandle<()>>>,
}

impl TrackerController {
    pub fn new(
        shutdown_bool: Arc<AtomicBool>,
        handles: Vec<Option<std::thread::JoinHandle<()>>>,
    ) -> TrackerController {
        TrackerController {
            shutdown_bool,
            handles,
        }
    }
}

impl Drop for TrackerController {
    fn drop(&mut self) {
        self.shutdown_bool
            .store(true, std::sync::atomic::Ordering::Relaxed);
        for op_handle in self.handles.iter_mut() {
            let handle = op_handle
                .take()
                .expect("Failed closing thread, already closed");
            let id = handle.thread().id();

            match handle.join() {
                Ok(_) => {
                    println!("Thread {:#?} joined", id);
                }
                Err(_) => {
                    println!("Thread {:#?} failed to join", id);
                }
            }
        }
    }
}
