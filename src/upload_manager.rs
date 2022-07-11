use crate::{
    download_manager::PieceStatus, errors::upload_manager_error::UploadManagerError,
    logger::LogMsg, peer_entities::communication_method::CommunicationMethod,
    peer_entities::peer_connection::fmt_chunk, ui::ui_codes::*, utilities::constants::CHUNK_SIZE,
    utilities::utils::UiParams,
};
use chrono::{offset::Utc, DateTime};
use glib::Sender as UISender;
use std::{
    io::{Read, Seek},
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Mutex,
    },
    time::SystemTime,
};

/// The UploadManager is responsible for managing the upload process of a piece.
pub struct UploadManager {
    logger_sender: Sender<LogMsg>,
    pub download_path: String,
    pub bitfield: Arc<Vec<Mutex<PieceStatus>>>,
    pub receiver: Arc<Mutex<Receiver<Option<PieceRequest>>>>,
    listener_control_sender: Arc<Mutex<Sender<String>>>,
}

/// This struct contains the required data to upload a piece.
pub struct PieceRequest {
    pub piece_index: u32,
    pub offset: u32,
    pub length: u32,
    pub stream: Arc<Mutex<Box<dyn CommunicationMethod + Send>>>,
    pub peer_id: String,
}

impl UploadManager {
    /// Creates a new UploaderManager instance.
    pub fn new(
        logger_sender: Sender<LogMsg>,
        download_path: String,
        bitfield: Arc<Vec<Mutex<PieceStatus>>>,
        receiver: Arc<Mutex<Receiver<Option<PieceRequest>>>>,
        listener_control_sender: Arc<Mutex<Sender<String>>>,
    ) -> Self {
        Self {
            logger_sender,
            download_path,
            bitfield,
            receiver,
            listener_control_sender,
        }
    }

    /// Starts to listen for piece requests and then uploads the corresponding piece.
    #[allow(clippy::type_complexity)]
    pub fn start_uploader(
        &self,
        sender_client: Arc<Mutex<UISender<Vec<(usize, UiParams, String)>>>>,
        torrent_name: String,
    ) -> Result<(), UploadManagerError> {
        self.logger_sender.send(LogMsg::Info(
            "Uploader started waiting for requests...".to_string(),
        ))?;
        loop {
            let piece_request = self.receiver.lock()?.recv()?;
            //check if i have the downloaded piece
            if let Some(piece_request) = piece_request {
                let piece_index = piece_request.piece_index;
                let offset = piece_request.offset;
                let length = piece_request.length;

                let piece_lock = &self.bitfield[piece_index as usize];
                if let PieceStatus::NotDownloaded = piece_lock.lock()?.to_owned() {
                    continue;
                }
                self.logger_sender.send(LogMsg::Info(format!(
                    "Sending piece: {}, offset:{}, chunk len: {}",
                    piece_index, offset, length
                )))?;

                let stream = piece_request.stream;
                let mut piece_data = vec![0; length as usize];

                let mut piece_file = std::fs::File::open(format!(
                    "{}/piece_{}.txt",
                    self.download_path, piece_index
                ))?;
                piece_file.seek(std::io::SeekFrom::Start(offset as u64))?;
                let _ = piece_file.read(&mut piece_data)?;

                let system_time = SystemTime::now();
                let datetime: DateTime<Utc> = system_time.into();
                let timestamp = datetime.timestamp();
                let piece_data = &fmt_chunk(piece_index, offset, &piece_data);
                stream.lock()?.write_all(piece_data)?;

                let system_time2 = SystemTime::now();
                let datetime2: DateTime<Utc> = system_time2.into();
                let timestamp2 = datetime2.timestamp();
                let upload_speed = CHUNK_SIZE as i64 / (timestamp2 - timestamp);

                sender_client.lock()?.send(vec![(
                    UPDATE_UPSPEED,
                    UiParams::Vector(vec![
                        piece_request.peer_id,
                        format!("{} bytes / sec", upload_speed),
                    ]),
                    torrent_name.clone(),
                )])?;
            } else {
                self.listener_control_sender
                    .lock()?
                    .send("stop".to_string())?;
                self.logger_sender.send(LogMsg::Info(
                    "Uploader stopped, terminating listener...".to_string(),
                ))?;

                return Ok(());
            }
        }
    }
}
