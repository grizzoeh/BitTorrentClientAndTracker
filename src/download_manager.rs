use crate::client::Client;
use crate::errors::download_manager_error::DownloadManagerError;
use crate::peer_connection::PeerConnection;
use crate::threadpool::ThreadPool;
use sha1::{Digest, Sha1};
use std::fs::File;
use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, Mutex, RwLock};

const CHUNK_SIZE: u32 = 16384;
const INITIAL_OFFSET: u32 = 0;
const PIECE_HASH_LEN: usize = 20;
const _CHOKE_ID: u8 = 0;
const UNCHOKE_ID: u8 = 1;
const _INTERESTED_ID: u8 = 2;
const _NOT_INTERESTED_ID: u8 = 3;
const _HAVE_ID: u8 = 4;
const BITFIELD_ID: u8 = 5;
const _REQUEST_ID: u8 = 6;
const PIECE_ID: u8 = 7;
const _CANCEL_ID: u8 = 8;
const ERROR_ID: u8 = 10;

pub struct DownloadManager {
    pub download_path: String,
    client: Arc<Client>,
    pub bitfield: RwLock<Vec<PieceStatus>>,
    pieces_quantity: usize,
    idle_peers: Mutex<Vec<PeerConnection<TcpStream>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceStatus {
    NotDownloaded,
    Downloading,
    Downloaded,
}

impl DownloadManager {
    pub fn new(
        download_path: String,
        client: Arc<Client>,
    ) -> Result<Arc<DownloadManager>, DownloadManagerError> {
        let pieces_quantity = (*client.length.lock()? / *client.pieces_length.lock()?) as usize;
        let bitfield = RwLock::new(vec![PieceStatus::NotDownloaded; pieces_quantity]);
        Ok(Arc::new(DownloadManager {
            idle_peers: Mutex::new(Vec::new()),
            pieces_quantity,
            download_path,
            client,
            bitfield,
        }))
    }

    pub fn start_download(self: Arc<Self>) -> Result<(), DownloadManagerError> {
        println!("DOWNLOADING STARTED");
        let threadpool = ThreadPool::new(8);
        for _ in 0..(self.pieces_quantity / 10) {
            let self_copy = self.clone();
            let _exe_ret = threadpool.execute(move || {
                let _ret = self_copy.download_ten_pieces();
            });
        }
        // podriamos agregar una variable atomic bool para checkear cuando todas las piecas estén descargadas y salir del loop
        // porq no sabemos realmente cuantas veces tenemos que iterar

        // agregar un loop q mande keep alive a los idle peers cada 2min

        Ok(())
    }

    fn download_ten_pieces(self: Arc<Self>) -> Result<(), DownloadManagerError> {
        let mut piece_indices = Vec::new();
        let mut peer_connection = self.clone().get_peer_connection()?;
        println!("PEER CONNECTION ESTABLISHED");

        let mut message_type: u8 = ERROR_ID;
        while message_type != UNCHOKE_ID {
            // If we receive a "have" message instead of a piece message
            message_type = peer_connection.read_detect_message()?;
            if message_type == BITFIELD_ID {
                peer_connection.unchoke()?;
                peer_connection.interested()?;

                let mut i: usize = 0;
                while piece_indices.len() < 10 {
                    if peer_connection.peer.has_piece(i as u32)
                        && self.bitfield.read()?[i as usize] == PieceStatus::NotDownloaded
                    {
                        piece_indices.push(i);
                        self.bitfield.write()?[i as usize] = PieceStatus::Downloading;
                    }

                    i += 1;
                    if i >= self.pieces_quantity {
                        break;
                    }
                }
                println!("PIECES TO DOWNLOAD: {:?}", piece_indices);
            }
        }

        for piece_idx in piece_indices.clone() {
            if let Ok(piece_data) = self
                .clone()
                .request_piece(piece_idx as u32, &mut peer_connection)
            {
                store_piece(
                    &self.client.download_path.lock()?.clone(),
                    piece_idx as u32,
                    &piece_data,
                )?;
                self.bitfield.write()?[piece_idx as usize] = PieceStatus::Downloaded;
                println!("piece {} downloaded", piece_idx);
            } else {
                self.clean_bitfield(piece_indices)?;
                return Err(DownloadManagerError::new(
                    "Error requesting piece".to_string(),
                ));
            }
        }
        self.idle_peers.lock()?.push(peer_connection);
        println!("Ended Job Successfully");
        Ok(())
    }

    fn request_piece(
        self: Arc<Self>,
        piece_idx: u32,
        peer: &mut PeerConnection<TcpStream>,
    ) -> Result<Vec<u8>, DownloadManagerError> {
        let piece_length = *self.client.pieces_length.lock()? as u32;
        let mut offset: u32 = INITIAL_OFFSET;
        let mut piece_data: Vec<u8> = vec![];

        while offset < piece_length {
            peer.request_chunk(piece_idx, offset, &CHUNK_SIZE)?;
            let mut message_type = ERROR_ID;

            while message_type != PIECE_ID {
                // in case we receive a "have" message instead of a piece message
                message_type = peer.read_detect_message()?;
            }
            let chunk = peer.read_chunk(piece_idx, offset)?;
            piece_data.extend(chunk);
            offset += CHUNK_SIZE;
        }
        verify_piece(&self.client.pieces.lock()?.clone(), &piece_data, &piece_idx)?;
        Ok(piece_data)
    }

    fn get_peer_connection(
        self: Arc<Self>,
    ) -> Result<PeerConnection<TcpStream>, DownloadManagerError> {
        println!("GETTING PEER CONNECTION");
        if let Some(peer_connection) = self.idle_peers.lock()?.pop() {
            return Ok(peer_connection);
        } else {
            let aux = self.client.peers.read()?;
            for client_peer in aux.iter() {
                match self.client.clone().try_peer_connection(client_peer) {
                    Ok(peer_connection) => return Ok(peer_connection),
                    Err(_) => {
                        continue;
                    }
                };
            }
        }
        Err(DownloadManagerError::new("no peers available".to_string()))
    }

    fn clean_bitfield(
        self: Arc<Self>,
        piece_indices: Vec<usize>,
    ) -> Result<(), DownloadManagerError> {
        for piece_idx in piece_indices {
            self.bitfield.write()?[piece_idx as usize] = PieceStatus::NotDownloaded;
        }
        Ok(())
    }
}

fn verify_piece(
    pieces: &[u8],
    piece_data: &[u8],
    piece_idx: &u32,
) -> Result<(), DownloadManagerError> {
    let piece_idx = *piece_idx as usize;
    let mut hasher = Sha1::new();
    hasher.update(piece_data);
    let piece_hash = hasher.finalize().to_vec();
    if piece_hash
        != pieces[(piece_idx * PIECE_HASH_LEN)..((piece_idx + 1) * PIECE_HASH_LEN)].to_vec()
    {
        return Err(DownloadManagerError::new(format!(
            "piece hash does not match, piece_idx: {}\nleft: {:?}\nright: {:?}",
            piece_idx,
            piece_hash,
            pieces[(piece_idx * PIECE_HASH_LEN)..((piece_idx + 1) * PIECE_HASH_LEN)].to_vec()
        )));
    }
    Ok(())
}

fn store_piece(
    download_path: &str,
    piece_idx: u32,
    piece_data: &[u8],
) -> Result<(), DownloadManagerError> {
    let mut file = File::create(format!("{}piece_{}.txt", download_path, piece_idx))?;
    file.write_all(piece_data)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_verify_piece_with_wrong_piece() {
        // init hashes
        let mut hasher = Sha1::new();
        hasher.update(
            [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]
            .as_ref(),
        );
        let hash1 = hasher.finalize()[..].to_vec();

        let mut hasher = Sha1::new();
        hasher.update([
            21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40,
        ]);
        let hash2 = hasher.finalize()[..].to_vec();
        // end init hashes

        let mut pieces = hash1.clone();
        pieces.extend(hash2.clone());

        let piece_data = [
            21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 200,
        ]; // wrong piece
        let piece_idx = 1;
        assert!(verify_piece(&pieces, &piece_data, &piece_idx).is_err());
    }

    #[test]
    fn test_verify_right_piece() {
        // init hashes
        let mut hasher = Sha1::new();
        hasher.update(
            [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]
            .as_ref(),
        );
        let hash1 = hasher.finalize()[..].to_vec();

        let mut hasher = Sha1::new();
        hasher.update([
            21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40,
        ]);
        let hash2 = hasher.finalize()[..].to_vec();
        // end init hashes

        let mut pieces = hash1.clone();
        pieces.extend(hash2.clone());

        let piece_data = [
            21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40,
        ];
        let piece_idx = 1;
        assert!(verify_piece(&pieces, &piece_data, &piece_idx).is_ok());
    }
}
