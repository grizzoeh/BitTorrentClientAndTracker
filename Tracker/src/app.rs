use std::{
    io::stdin,
    sync::{atomic::AtomicBool, mpsc::channel, Arc},
    thread::spawn,
};

use crate::{
    constants::{LISTENER_IP, LISTENER_PORT, LOG_PATH},
    controller::TrackerController,
    data_manager::DataManager,
    errors::app_error::AppError,
    listener::Listener,
    logger::{LogMsg, Logger},
};

pub fn initialize_app(data_manager_path: &str) -> Result<(), AppError> {
    let shutdown_bool = Arc::new(AtomicBool::new(false));
    let (logger_sender, logger_receiver) = channel();

    let mut data_manager = DataManager::new(
        data_manager_path.to_string(),
        shutdown_bool.clone(),
        logger_sender.clone(),
    )?;

    let tracker = data_manager.init_tracker().expect("Failed to init tracker");

    let handle_data_manager = spawn(move || {
        let _r = data_manager.start(); // THREAD DATA MANAGER
    });

    let mut logger = Logger::new(LOG_PATH.to_string(), logger_receiver)?;

    // Start logger in a new thread
    let handle_logger = spawn(move || {
        let _r = logger.start(); // THREAD LOGGER
    });
    let listener = Listener::new(
        LISTENER_PORT,
        LISTENER_IP.to_string(),
        logger_sender.clone(),
        shutdown_bool.clone(),
    )?;
    let handle_listener = spawn(move || {
        let _r = listener.listen(tracker);
    });

    let _controller = TrackerController::new(
        shutdown_bool,
        vec![
            Some(handle_data_manager),
            Some(handle_logger),
            Some(handle_listener),
        ],
    );
    println!("Press enter to finish...");
    let mut input = String::new();
    stdin().read_line(&mut input)?;
    logger_sender.send(LogMsg::End)?;
    Ok(())
}
