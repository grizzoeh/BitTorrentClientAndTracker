use crate::communication_method::{CommunicationMethod, TCP};
use crate::download_manager::PieceStatus;
use crate::errors::listener_error::ListenerError;
use crate::logger::LogMsg;
use crate::peer::{add_piece_to_bitfield, IncomingPeer};
use crate::peer_connection::PeerConnection;
use crate::threadpool::ThreadPool;
use crate::upload_manager::PieceRequest;
use std::io::{self};
use std::net::TcpListener;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;

pub struct Listener {
    listener: TcpListener,
    _peers: Vec<PeerConnection<IncomingPeer>>,
    bitfield: Arc<RwLock<Vec<PieceStatus>>>,
    listener_control_receiver: Arc<Mutex<Receiver<String>>>,
    logger_sender: Arc<Mutex<Sender<LogMsg>>>,
    upload_sender: Arc<Mutex<Sender<Option<PieceRequest>>>>,
    client_id: String,
    info_hash: Vec<u8>,
}

impl Listener {
    pub fn new(
        addr: &str,
        bitfield: Arc<RwLock<Vec<PieceStatus>>>,
        listener_control_receiver: Arc<Mutex<Receiver<String>>>,
        logger_sender: Arc<Mutex<Sender<LogMsg>>>,
        upload_sender: Arc<Mutex<Sender<Option<PieceRequest>>>>,
        client_id: String,
        info_hash: Vec<u8>,
    ) -> Result<Arc<Self>, ListenerError> {
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        Ok(Arc::new(Self {
            listener,
            _peers: Vec::new(),
            bitfield,
            listener_control_receiver,
            logger_sender,
            upload_sender,
            client_id,
            info_hash,
        }))
    }

    pub fn listen(self: Arc<Self>) -> Result<(), ListenerError> {
        let threadpool = ThreadPool::new(2);
        self.logger_sender.lock()?.send(LogMsg::Info(
            "Started Listening for connections...".to_string(),
        ))?;

        for stream in self.listener.incoming() {
            match stream {
                Ok(s) => {
                    let stream_connection = TCP { stream: Some(s) };
                    if let Ok(peer_connection) =
                        self.clone().init_incoming(Box::new(stream_connection))
                    {
                        let copy = self.clone();
                        self.logger_sender.lock()?.send(LogMsg::Info(format!(
                            "Incoming Peer Connection: {}",
                            peer_connection.peer.read()?.id
                        )))?;
                        let _exe_ret = threadpool.execute(move || {
                            let _r = copy.handle_connection(peer_connection);
                        });
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    if self.listener_control_receiver.lock()?.try_recv().is_ok() {
                        self.logger_sender.lock()?.send(LogMsg::Info(
                            "Listener received Stop message, terminating listener...".to_string(),
                        ))?;
                        println!("Listener received Stop message, terminating listener...");
                        return Ok(());
                    } else {
                        thread::sleep(Duration::from_secs(5));
                        continue;
                    }
                }
                Err(e) => return Err(ListenerError::from(e)),
            }
        }
        Ok(())
    }

    pub fn init_incoming(
        self: Arc<Self>,
        s: Box<dyn CommunicationMethod + Send>,
    ) -> Result<Arc<PeerConnection<IncomingPeer>>, ListenerError> {
        let peer = IncomingPeer::new(
            String::from("default_id"),
            s.peer_addr()?.ip().to_string(),
            s.peer_addr()?.port(),
        );
        let peer_connection = PeerConnection::new(
            peer,
            self.info_hash.clone(),
            self.client_id.clone(),
            Arc::new(Mutex::new(s)),
            self.logger_sender.clone(),
            self.upload_sender.clone(),
        );

        peer_connection
            .stream
            .lock()?
            .set_read_timeout(Some(std::time::Duration::from_secs(120)))?;
        Ok(Arc::new(peer_connection))
    }

    pub fn handle_connection(
        self: Arc<Self>,
        peer_connection: Arc<PeerConnection<IncomingPeer>>,
    ) -> Result<(), ListenerError> {
        peer_connection
            .clone()
            .handshake(peer_connection.peer.read().unwrap().id.clone())?;
        peer_connection.clone().unchoke()?;
        peer_connection.clone().bitfield(self.build_bitfield()?)?;
        loop {
            peer_connection.clone().read_detect_message()?;
        }
    }

    pub fn build_bitfield(self: Arc<Self>) -> Result<Vec<u8>, ListenerError> {
        let mut bitfield_len = (self.bitfield.read()?.len() as usize) / 8;
        if bitfield_len % 8 != 0 {
            bitfield_len += 1;
        }
        let mut bitfield = vec![0; bitfield_len as usize];
        for (i, piece) in self.bitfield.read()?.iter().enumerate() {
            if *piece == PieceStatus::Downloaded {
                add_piece_to_bitfield(&mut bitfield, i as u32);
            }
        }
        Ok(bitfield)
    }
}
