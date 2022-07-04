use crate::communication_method::CommunicationMethod;
use crate::{
    download_manager::PieceStatus, errors::upload_manager_error::UploadManagerError,
    logger::LogMsg, peer_connection::fmt_chunk, ui_codes::*, utils::UiParams,
};
use chrono::offset::Utc;
use chrono::DateTime;
use glib::Sender as UISender;
use std::time::SystemTime;
use std::{
    io::{Read, Seek},
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Mutex, RwLock,
    },
};

pub struct UploadManager {
    logger_sender: Sender<LogMsg>,
    pub download_path: String,
    pub bitfield: Arc<RwLock<Vec<PieceStatus>>>,
    pub receiver: Arc<Mutex<Receiver<Option<PieceRequest>>>>,
    listener_control_sender: Arc<Mutex<Sender<String>>>,
}
pub struct PieceRequest {
    pub piece_index: u32,
    pub offset: u32,
    pub length: u32,
    pub stream: Arc<Mutex<Box<dyn CommunicationMethod + Send>>>,
    pub peer_id: String,
}

impl UploadManager {
    pub fn new(
        logger_sender: Sender<LogMsg>,
        download_path: String,
        bitfield: Arc<RwLock<Vec<PieceStatus>>>,
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
            if let Some(piece_request) = piece_request {
                let piece_index = piece_request.piece_index;
                let offset = piece_request.offset;
                let length = piece_request.length;
                if self.bitfield.read()?[piece_index as usize] != PieceStatus::Downloaded {
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

                stream
                    .lock()?
                    .write_all(&fmt_chunk(piece_index, offset, &piece_data))?;

                let system_time2 = SystemTime::now();
                let datetime2: DateTime<Utc> = system_time2.into();
                let timestamp2 = datetime2.timestamp();
                let upload_speed = 16384 / (timestamp2 - timestamp);

                sender_client.lock()?.send(vec![(
                    UPDATE_UPSPEED,
                    UiParams::Vector(vec![
                        piece_request.peer_id,
                        format!("{} bytes / sec", upload_speed),
                    ]),
                    torrent_name.clone(),
                )])?;
            } else {
                println!("Uploader stopped, lock listener_control_sender");
                self.listener_control_sender
                    .lock()?
                    .send("stop".to_string())?;
                println!("Uploader stopped,listener sended");
                return Ok(());
            }
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use std::{sync::mpsc::channel, thread::spawn};

//     use crate::{constants::CHUNK_SIZE, test_helper::MockTcpStream};

//     use super::*;

//     #[test]
//     fn test_uploading_entire_piece_successfuly() {
//         // init test data
//         let chunks_quantity_by_piece = 262144 / CHUNK_SIZE as u32;
//         let download_path = "src/upload_manager_test_files/pieces".to_string();
//         let (sender, receiver): (
//             Sender<Option<PieceRequest<MockTcpStream>>>,
//             Receiver<Option<PieceRequest<MockTcpStream>>>,
//         ) = channel();
//         let (logger_sender, logger_receiver): (Sender<LogMsg>, Receiver<LogMsg>) = channel();
//         let uploader = UploadManager::new(
//             logger_sender,
//             download_path.clone(),
//             Arc::new(RwLock::new(vec![PieceStatus::Downloaded; 10])),
//             Arc::new(Mutex::new(receiver)),
//         );
//         let _uploader_handler = spawn(move || uploader.start_uploader());
//         let stream = Arc::new(Mutex::new(MockTcpStream::new(vec![0; 100])));

//         // send requests to uploader
//         for i in 0..chunks_quantity_by_piece + 1 {
//             let piece_request = PieceRequest {
//                 piece_index: 0 as u32,
//                 offset: (CHUNK_SIZE * i) as u32,
//                 length: CHUNK_SIZE,
//                 stream: stream.clone(),
//             };
//             sender.send(Some(piece_request)).unwrap();
//         }
//         // close uploader
//         sender.send(None).unwrap();
//         let file_content = std::fs::read(format!("{}/piece_0.txt", download_path)).unwrap();
//         // consume logger_receiver to get all logs
//         for _ in 0..18 {
//             // si no leo las cosas del channel del logger rompe el test, nose porq
//             //println!("logger_receiver: {:?}", logger_receiver.recv().unwrap());
//             logger_receiver.recv().unwrap();
//         }
//         // checkeo que el archivo tenga el mismo contenido que el stream
//         assert_eq!(
//             stream.lock().unwrap().clone().cursor_w.into_inner().len(),
//             file_content.len() + 13 * (chunks_quantity_by_piece as usize) // add 13 bytes per chunk (piece header)
//         );

//         drop(stream);
//         println!("join returned:{:?}", _uploader_handler.join().unwrap());
//     }

//     #[test]
//     fn test_trying_to_request_piece_not_downloaded() {
//         // init test data
//         let download_path = "src/upload_manager_test_files/pieces".to_string();
//         let (sender, receiver): (
//             Sender<Option<PieceRequest<MockTcpStream>>>,
//             Receiver<Option<PieceRequest<MockTcpStream>>>,
//         ) = channel();
//         let (logger_sender, logger_receiver): (Sender<LogMsg>, Receiver<LogMsg>) = channel();
//         let mut bitfield = vec![PieceStatus::Downloaded; 10];
//         bitfield[8] = PieceStatus::NotDownloaded; // piece 8 is not downloaded
//         let uploader = UploadManager::new(
//             logger_sender,
//             download_path.clone(),
//             Arc::new(RwLock::new(bitfield)),
//             Arc::new(Mutex::new(receiver)),
//         );
//         let _uploader_handler = spawn(move || uploader.start_uploader());
//         let stream = Arc::new(Mutex::new(MockTcpStream::new(vec![0; 100])));

//         // send requests to uploader
//         let piece_request = PieceRequest {
//             piece_index: 8 as u32, // request piece 8
//             offset: 0 as u32,
//             length: CHUNK_SIZE,
//             stream: stream.clone(),
//         };
//         sender.send(Some(piece_request)).unwrap();
//         // close uploader
//         sender.send(None).unwrap();
//         // consume logger_receiver to get all logs
//         logger_receiver.recv().unwrap(); // aca no hace falta hacerlo nose porq

//         // checkeo que el archivo tenga el mismo contenido que el stream
//         assert_eq!(
//             stream.lock().unwrap().clone().cursor_w.into_inner().len(), // should be 0 bytes, because of not having piece 8
//             0
//         );
//         drop(stream);
//         println!("join returned:{:?}", _uploader_handler.join().unwrap());
//     }
// }
