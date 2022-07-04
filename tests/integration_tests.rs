// #[cfg(test)]
// mod tests {
//     use crabrave::communication_method::CommunicationMethod;
//     use crabrave::download_manager::DownloadManager;
//     use crabrave::listener::Listener;
//     use crabrave::logger::LogMsg;
//     use crabrave::logger::Logger;
//     use crabrave::test_helper;
//     use crabrave::test_helper::CommunicationMock1;
//     use crabrave::upload_manager::PieceRequest;
//     use crabrave::upload_manager::UploadManager;
//     use crabrave::{
//         args::get_torrents_paths, constants::CHUNK_SIZE, download_manager::DownloaderInfo,
//         peer::Peer, peer_connection::PeerConnection, utils::UiParams,
//     };
//     use sha1::{Digest, Sha1};
//     use std::sync::mpsc;
//     use std::sync::mpsc::{Receiver, Sender};
//     use std::sync::{mpsc::channel, Arc, Mutex, RwLock};
//     use std::thread::spawn;
//     // use crabrave::client::Client;
//     // use crabrave::config_parser::config_parse;
//     // use crabrave::constants::*;
//     // use crabrave::download_manager::{DownloadManager, DownloaderInfo};
//     // use crabrave::peer::{Peer, PeerInterface};
//     // use glib;
//     // use gtk::prelude::*;
//     // use gtk::*;
//     // use gtk::{Builder, Grid, Label, Window};
//     // use sha1::{Digest, Sha1};
//     // use std::collections::HashMap;
//     // use std::env;
//     // use std::sync::mpsc::{channel, Receiver, Sender};
//     // use std::sync::{Arc, Mutex, RwLock};
//     extern crate native_tls;
//     // use crabrave::errors::download_manager_error::DownloadManagerError;
//     // use crabrave::errors::listener_error::ListenerError;
//     // use crabrave::errors::logger_error::LoggerError;
//     // use crabrave::errors::upload_manager_error::UploadManagerError;
//     // use crabrave::ui_codes::*;
//     // use crabrave::utils::{to_gb, UiParams};
//     // use std::thread::{spawn, JoinHandle};

//     #[test]
//     fn test_compilation() {
//         assert!(true);
//     }

//     // #[test]
//     // fn test_connection() {
//     //     let torrent_paths: Vec<String> =
//     //         get_torrents_paths(&get_torrents_paths("src/torrent_test_files").unwrap()[0]).unwrap();
//     //     let torrent_path = torrent_paths[0].clone();
//     //     let mut config = config_parse(CONFIG_PATH.to_string()).unwrap();
//     //     config.insert("torrent_path".to_string(), torrent_path);
//     //     let (aux_tx, aux_rx): (
//     //         Sender<glib::Sender<Vec<(usize, UiParams, String)>>>,
//     //         Receiver<glib::Sender<Vec<(usize, UiParams, String)>>>,
//     //     ) = channel();

//     //     let client_sender = aux_rx.recv().unwrap();
//     //     let ui_sender = Arc::new(Mutex::new(client_sender.clone()));

//     //     let (client, logger_handler) =
//     //         Client::new(config, ui_sender, torrent_paths[0].clone(), 1476).unwrap();

//     //     let mut handles: Vec<(
//     //         JoinHandle<Result<(), LoggerError>>,
//     //         JoinHandle<Result<(), DownloadManagerError>>,
//     //         JoinHandle<Result<(), ListenerError>>,
//     //         JoinHandle<Result<(), UploadManagerError>>,
//     //     )> = Vec::new();

//     //     let (download_handle, listener_handle, upload_handle) = client.start().unwrap();
//     //     handles.push((
//     //         logger_handler,
//     //         download_handle,
//     //         listener_handle,
//     //         upload_handle,
//     //     ));

//     //     assert_eq!(true, true);
//     // }

//     // #[test]
//     // fn test_correct_url_returns_correct_number_of_paths() {
//     //     let paths = get_torrents_paths("src/torrent_test_files").unwrap();
//     //     println!("{:?}", paths);
//     //     assert_eq!(paths.len(), 4);
//     // }

// //     #[test]
// //     fn test_integracion_descarga() {
// //         // initialization
// //         let piece1 = vec![0; CHUNK_SIZE as usize];
// //         let piece2 = vec![1; CHUNK_SIZE as usize];
// //         let piece3 = vec![2; CHUNK_SIZE as usize];

// //         let mut hasher = Sha1::new();
// //         hasher.update(piece1);
// //         let hash1 = hasher.finalize()[..].to_vec();

// //         let mut hasher2 = Sha1::new();
// //         hasher2.update(piece2);
// //         let hash2 = hasher2.finalize()[..].to_vec();

// //         let mut hasher3 = Sha1::new();
// //         hasher3.update(piece3);
// //         let hash3 = hasher3.finalize()[..].to_vec();

// //         let mut piece_hash = Vec::new();
// //         piece_hash.extend(hash1);
// //         piece_hash.extend(hash2);
// //         piece_hash.extend(hash3);

// //         let info_hash: Vec<u8> = vec![5; 20];
// //         let client_id = "client_id11111111111".to_string();

// //         let peer1 = Peer::new("default_id".to_string(), "localhost".to_string(), 8080);
// //         let peer2 = Peer::new("default_id".to_string(), "localhost".to_string(), 8080);
// //         let peer3 = Peer::new("default_id".to_string(), "localhost".to_string(), 8080);

// //         let (sender_logger, receiver_logger): (Sender<LogMsg>, Receiver<LogMsg>) = channel();
// //         let (sender_upload, receiver_upload): (
// //             Sender<Option<PieceRequest>>,
// //             Receiver<Option<PieceRequest>>,
// //         ) = channel();

// //         let peer_connection1 = PeerConnection::new(
// //             peer1,
// //             info_hash.clone(),
// //             client_id.clone(),
// //             Arc::new(Mutex::new(CommunicationMock1::create())),
// //             Arc::new(Mutex::new(sender_logger.clone())),
// //             Arc::new(Mutex::new(sender_upload.clone())),
// //         );
// //         let peer_connection2 = PeerConnection::new(
// //             peer2,
// //             info_hash.clone(),
// //             client_id.clone(),
// //             Arc::new(Mutex::new(test_helper::CommunicationMock2::create())),
// //             Arc::new(Mutex::new(sender_logger.clone())),
// //             Arc::new(Mutex::new(sender_upload.clone())),
// //         );
// //         let peer_connection3 = PeerConnection::new(
// //             peer3,
// //             info_hash.clone(),
// //             client_id.clone(),
// //             Arc::new(Mutex::new(test_helper::CommunicationMock3::create())),
// //             Arc::new(Mutex::new(sender_logger.clone())),
// //             Arc::new(Mutex::new(sender_upload.clone())),
// //         );

// //         let peers_vec = vec![
// //             Arc::new(peer_connection1),
// //             Arc::new(peer_connection2),
// //             Arc::new(peer_connection3),
// //         ];

// //         let (sender_client, _): (
// //             glib::Sender<Vec<(usize, UiParams, String)>>,
// //             glib::Receiver<Vec<(usize, UiParams, String)>>,
// //         ) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

// //         let download_path = "tests/test_files/downloads/".to_string();
// //         let downloader_info = DownloaderInfo {
// //             length: 3 * CHUNK_SIZE as u64,
// //             piece_length: CHUNK_SIZE as u64,
// //             download_path: download_path.clone(),
// //             logger_sender: Arc::new(Mutex::new(sender_logger.clone())),
// //             pieces_hash: piece_hash,
// //             peers: Arc::new(RwLock::new(peers_vec)),
// //             info_hash: info_hash.clone(),
// //             client_id: client_id.clone(),
// //             upload_sender: Arc::new(Mutex::new(sender_upload.clone())),
// //             torrent_name: "archivotorrent.txt".to_string(),
// //             file_length: 3 * CHUNK_SIZE as u64,
// //             ui_sender: Arc::new(Mutex::new(sender_client.clone())),
// //         };

// //         // execute

// //         let download_manager = DownloadManager::new(downloader_info).unwrap();
// //         let listener_channel = mpsc::channel();
// //         let bitfield = download_manager.clone().get_bitfield();
// //         let listener = Listener::new(
// //             format!("127.0.0.1:{}", 1476).as_str(),
// //             bitfield.clone(),
// //             Arc::new(Mutex::new(listener_channel.1)),
// //             Arc::new(Mutex::new(sender_logger.clone())),
// //             Arc::new(Mutex::new(sender_upload.clone())),
// //             client_id.clone(),
// //             info_hash.clone(),
// //         )
// //         .unwrap();
// //         let upload_manager = UploadManager::new(
// //             sender_logger.clone(),
// //             download_path.clone(),
// //             bitfield,
// //             Arc::new(Mutex::new(receiver_upload)),
// //             Arc::new(Mutex::new(listener_channel.0)),
// //         );
// //         let mut logger = Logger::new(
// //             "tests/test_files/logs_test.txt".to_string(),
// //             receiver_logger,
// //         )
// //         .unwrap();
// //         let logger_handler = spawn(move || logger.start());
// //         let download_handle = spawn(move || download_manager.start_download());
// //         let listener_handle = spawn(move || listener.listen());

// //         let sender_client_cp = Arc::new(Mutex::new(sender_client.clone()));
// //         let torrent_name_cp = "archivotorrent.txt".to_string();

// //         let upload_handle =
// //             spawn(move || upload_manager.start_uploader(sender_client_cp, torrent_name_cp));

// //         let _rd = download_handle.join().unwrap();
// //         let _rl = listener_handle.join().unwrap();
// //         let _ru = upload_handle.join().unwrap();
// //         let _rlog = logger_handler.join().unwrap();
// //         // assertion

// //         let mut downloaded_pieces = Vec::new();
// //         for entry in std::fs::read_dir(download_path).unwrap().flatten() {
// //             if !entry.path().is_dir() {
// //                 let piece_id: usize = entry
// //                     .file_name()
// //                     .to_str()
// //                     .unwrap()
// //                     .split('_')
// //                     .collect::<Vec<&str>>()[1]
// //                     .split('.')
// //                     .collect::<Vec<&str>>()[0]
// //                     .parse()
// //                     .unwrap();
// //                 downloaded_pieces.push(piece_id.clone());
// //             }
// //         }
// //         assert_eq!(downloaded_pieces.len(), 3);
// //     }
// // }

// // esto se corre con
// // cargo test --test "*" -- --nocapture
// // el no capture para imprimir cosas sin q rompa el test
