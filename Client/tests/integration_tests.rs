#[cfg(test)]
mod peer_mock;
mod tests {
    use crate::peer_mock::*;
    use crabrave::download_manager::DownloadManager;
    use crabrave::listener::Listener;
    use crabrave::logger::LogMsg;
    use crabrave::logger::Logger;
    use crabrave::peer_entities::communication_method::CommunicationMethod;
    use crabrave::upload_manager::PieceRequest;
    use crabrave::upload_manager::UploadManager;
    use crabrave::{
        download_manager::DownloaderInfo, peer_entities::peer::Peer,
        peer_entities::peer_connection::PeerConnection, utilities::utils::UiParams,
    };
    use sha1::{Digest, Sha1};
    use std::fs::remove_file;
    use std::fs::File;
    use std::io::Read;
    use std::path::Path;
    use std::sync::mpsc::{Receiver, Sender};
    use std::sync::{mpsc::channel, Arc, Mutex, RwLock};
    use std::thread::spawn;

    #[test]
    fn test_integracion_descarga() {
        // initialization
        let download_path = "tests/test_files/downloads/".to_string();
        let download_pieces_path = "tests/test_files/download_pieces_test".to_string();
        let piece_length = 262144;
        let torrent_name = "archivotorrent".to_string();
        let _r = remove_file(Path::new(&format!("{}/{}", download_path, torrent_name))).unwrap(); // delete file if exists
        let _r = remove_file(Path::new(&format!(
            "{}/{}",
            download_pieces_path, "piece_0.txt"
        )))
        .unwrap(); // delete piece_0.txt if exists
        let info_hash: Vec<u8> = vec![5; 20];

        let mut piece_file =
            File::open("tests/test_files/integration_test_piece_src.txt".to_string()).unwrap();
        let mut piece = Vec::new();
        let _r = piece_file.read_to_end(&mut piece).unwrap();

        let mut hasher = Sha1::new();
        hasher.update(piece.clone());
        let pieces_hash = hasher.finalize().to_vec();

        let client_id = "client_id11111111111".to_string();

        let peer1 = Peer::new("default_id1".to_string(), "localhost1".to_string(), 8080);

        let (sender_logger, receiver_logger): (Sender<LogMsg>, Receiver<LogMsg>) = channel();
        let (sender_upload, receiver_upload): (
            Sender<Option<PieceRequest>>,
            Receiver<Option<PieceRequest>>,
        ) = channel();
        let (listener_control_tx, listener_control_rx): (Sender<String>, Receiver<String>) =
            channel();
        let peer_connection1 = PeerConnection::new(
            peer1,
            info_hash.clone(),
            client_id.clone(),
            Arc::new(Mutex::new(CommunicationMock1::create())),
            Arc::new(Mutex::new(sender_logger.clone())),
            Arc::new(Mutex::new(sender_upload.clone())),
        );

        let peers_vec = vec![Arc::new(peer_connection1)];

        let (sender_client, _receiver): (
            glib::Sender<Vec<(usize, UiParams, String)>>,
            glib::Receiver<Vec<(usize, UiParams, String)>>,
        ) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let downloader_info = DownloaderInfo {
            piece_length: piece_length as u64,
            download_path: download_path.clone(),
            logger_sender: Arc::new(Mutex::new(sender_logger.clone())),
            pieces_hash: pieces_hash,
            peers: Arc::new(RwLock::new(peers_vec)),
            info_hash: info_hash.clone(),
            client_id: client_id.clone(),
            upload_sender: Arc::new(Mutex::new(sender_upload.clone())),
            torrent_name: "archivotorrent.txt".to_string(),
            file_length: piece_length as u64,
            ui_sender: Arc::new(Mutex::new(sender_client.clone())),
            download_pieces_path: download_pieces_path.clone(),
        };

        // Execute
        let download_manager = DownloadManager::new(downloader_info).unwrap();
        let bitfield = download_manager.bitfield.clone();

        let listener = Listener::new(
            format!("127.0.0.1:{}", 1476).as_str(),
            bitfield.clone(),
            Arc::new(Mutex::new(listener_control_rx)),
            Arc::new(Mutex::new(sender_logger.clone())),
            Arc::new(Mutex::new(sender_upload.clone())),
            client_id.clone(),
            info_hash.clone(),
            Arc::new(Mutex::new(sender_client.clone())),
            "test.torrent".to_string(),
        )
        .unwrap();
        let upload_manager = UploadManager::new(
            sender_logger.clone(),
            download_path.clone(),
            bitfield,
            Arc::new(Mutex::new(receiver_upload)),
            Arc::new(Mutex::new(listener_control_tx.clone())),
        );
        let mut logger = Logger::new(
            "tests/test_files/logs_test.txt".to_string(),
            receiver_logger,
        )
        .unwrap();
        let logger_handler = spawn(move || {
            let _r = logger.start();
        });
        let download_handle = spawn(move || {
            let _r = download_manager.start_download();
        });
        let listener_handle = spawn(move || {
            let _r = listener.listen();
        });

        let sender_client_cp = Arc::new(Mutex::new(sender_client.clone()));
        let torrent_name_cp = "archivotorrent.txt".to_string();

        let upload_handle = spawn(move || {
            let _r = upload_manager.start_uploader(sender_client_cp, torrent_name_cp);
        });

        let _rl = listener_handle.join();
        let _rd = download_handle.join();
        let _ru = upload_handle.join();
        let _rlog = logger_handler.join();
        // assertion

        let mut piece_file =
            File::open("tests/test_files/integration_test_piece_src.txt".to_string()).unwrap();
        let mut piece = Vec::new();
        let _r = piece_file.read_to_end(&mut piece).unwrap();

        let mut downloaded_piece_file =
            File::open("tests/test_files/download_pieces_test/piece_0.txt".to_string()).unwrap();
        let mut downloaded_piece = Vec::new();
        let _r = downloaded_piece_file
            .read_to_end(&mut downloaded_piece)
            .unwrap();

        assert_eq!(piece, downloaded_piece);
    }
}

// esto se corre con
// cargo test --test "*" -- --nocapture
// el no capture para imprimir cosas sin q rompa el test
