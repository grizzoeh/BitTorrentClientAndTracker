use crate::{
    download_manager::PieceStatus,
    errors::listener_error::ListenerError,
    logger::LogMsg,
    peer_entities::communication_method::{CommunicationMethod, TCP},
    peer_entities::peer::{add_piece_to_bitfield, IncomingPeer},
    peer_entities::peer_connection::PeerConnection,
    ui::ui_codes::*,
    upload_manager::PieceRequest,
    utilities::{constants::CHOKE_ID, utils::UiParams},
};
use glib::Sender as UISender;
use std::{
    io::{self},
    net::TcpListener,
    sync::mpsc::{Receiver, Sender},
    sync::{Arc, Mutex},
    thread::{self, spawn},
    time::Duration,
};

/// This struct listens for incoming connections and creates a new thread for each one.
#[allow(clippy::type_complexity)]
pub struct Listener {
    listener: TcpListener,
    bitfield: Arc<Vec<Mutex<PieceStatus>>>,
    listener_control_receiver: Arc<Mutex<Receiver<String>>>,
    logger_sender: Arc<Mutex<Sender<LogMsg>>>,
    upload_sender: Arc<Mutex<Sender<Option<PieceRequest>>>>,
    client_id: String,
    info_hash: Vec<u8>,
    sender_client: Arc<Mutex<UISender<Vec<(usize, UiParams, String)>>>>,
    torrent_name: String,
    threads_handles: Arc<Mutex<Vec<thread::JoinHandle<()>>>>,
}

#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
impl Listener {
    /// Creates a new listener instance.
    pub fn new(
        addr: &str,
        bitfield: Arc<Vec<Mutex<PieceStatus>>>,
        listener_control_receiver: Arc<Mutex<Receiver<String>>>,
        logger_sender: Arc<Mutex<Sender<LogMsg>>>,
        upload_sender: Arc<Mutex<Sender<Option<PieceRequest>>>>,
        client_id: String,
        info_hash: Vec<u8>,
        sender_client: Arc<Mutex<UISender<Vec<(usize, UiParams, String)>>>>,
        torrent_name: String,
    ) -> Result<Arc<Self>, ListenerError> {
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        Ok(Arc::new(Self {
            listener,
            bitfield,
            listener_control_receiver,
            logger_sender,
            upload_sender,
            client_id,
            info_hash,
            sender_client,
            torrent_name,
            threads_handles: Arc::new(Mutex::new(Vec::new())),
        }))
    }

    /// Starts to listen to incoming connections.
    pub fn listen(self: Arc<Self>) -> Result<(), ListenerError> {
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
                        let self_copy = self.clone();
                        self.logger_sender.lock()?.send(LogMsg::Info(format!(
                            "Incoming Peer Connection: {}",
                            peer_connection.peer.read()?.id
                        )))?;
                        self.threads_handles.lock().unwrap().push(spawn(move || {
                            let _r = self_copy.handle_connection(peer_connection);
                        }));
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    if self.listener_control_receiver.lock()?.try_recv().is_ok() {
                        self.logger_sender.lock()?.send(LogMsg::Info(
                            "Listener received Stop message, terminating listener...".to_string(),
                        ))?;
                        self.join_finished_threads()?;
                        return Ok(());
                    } else {
                        thread::sleep(Duration::from_secs(5));
                        continue;
                    }
                }

                Err(e) => {
                    self.join_finished_threads()?;
                    return Err(ListenerError::from(e));
                }
            }
        }
        Ok(())
    }

    /// Joins all threads when the listener is stopped.
    fn join_finished_threads(self: Arc<Self>) -> Result<(), ListenerError> {
        let mut handles = self.threads_handles.lock().unwrap();
        while handles.len() > 0 {
            match handles.pop() {
                Some(handle) => {
                    handle.join().unwrap();
                    self.sender_client.lock()?.send(vec![(
                        UPDATE_ACTIVE_CONNS,
                        UiParams::Usize(1),
                        self.torrent_name.clone(),
                    )])?
                }
                None => break,
            };
        }
        Ok(())
    }

    /// Returns an Incoming Peer connection given a CommunicationMethod.
    fn init_incoming(
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
            .set_read_timeout(Some(std::time::Duration::from_secs(3)))?;
        Ok(Arc::new(peer_connection))
    }

    /// Starts the initial message exchanges with the connected peer.
    fn handle_connection(
        self: Arc<Self>,
        peer_connection: Arc<PeerConnection<IncomingPeer>>,
    ) -> Result<(), ListenerError> {
        peer_connection.clone().handshake(self.client_id.clone())?;
        self.sender_client.lock()?.send(vec![(
            UPDATE_PEER_ID_IP_PORT,
            UiParams::Vector(vec![
                peer_connection.peer.read()?.id.clone(),
                peer_connection.peer.read()?.ip.clone(),
                format!("{}", peer_connection.peer.read()?.port.clone()),
            ]),
            self.torrent_name.clone(),
        )])?;
        self.sender_client.lock()?.send(vec![(
            UPDATE_ACTIVE_CONNS,
            UiParams::Usize(1),
            self.torrent_name.clone(),
        )])?;

        peer_connection.clone().unchoke()?;
        self.sender_client.lock()?.send(vec![(
            UPDATE_UNCHOKE,
            UiParams::Vector(vec![
                peer_connection.peer.read()?.id.clone(),
                "Unchoked".to_string(),
            ]),
            self.torrent_name.clone(),
        )])?;
        self.sender_client.lock()?.send(vec![(
            UPDATE_INTERESTED,
            UiParams::Vector(vec![
                peer_connection.peer.read()?.id.clone(),
                "Interested".to_string(),
            ]),
            self.torrent_name.clone(),
        )])?;

        peer_connection
            .clone()
            .bitfield(self.clone().build_bitfield()?)?;
        loop {
            match peer_connection.clone().read_detect_message() {
                Ok(msg) => {
                    if msg == CHOKE_ID {
                        self.sender_client.lock()?.send(vec![(
                            DELETE_ONE_ACTIVE_CONNECTION,
                            UiParams::Vector(vec![
                                peer_connection.peer.read()?.id.clone(),
                                "Disconnected".to_string(),
                            ]),
                            self.torrent_name.clone(),
                        )])?;
                        return Ok(());
                    }
                }
                Err(e) => {
                    self.sender_client.lock()?.send(vec![(
                        DELETE_ONE_ACTIVE_CONNECTION,
                        UiParams::Vector(vec![
                            peer_connection.peer.read()?.id.clone(),
                            "Disconnected".to_string(),
                        ]),
                        self.torrent_name.clone(),
                    )])?;
                    return Err(ListenerError::new(format!(
                        "Error reading incoming peer message: {}",
                        e
                    )));
                }
            };
        }
    }

    /// Returns a Vec of bytes representing the common Bitfield.
    fn build_bitfield(self: Arc<Self>) -> Result<Vec<u8>, ListenerError> {
        let mut bitfield_len = (self.bitfield.len() as usize) / 8;
        if bitfield_len % 8 != 0 {
            bitfield_len += 1;
        }
        let mut bitfield = vec![0; bitfield_len as usize];
        for (i, piece) in self.bitfield.iter().enumerate() {
            if let PieceStatus::Downloaded = piece.lock().unwrap().to_owned() {
                add_piece_to_bitfield(&mut bitfield, i as u32);
            }
        }
        Ok(bitfield)
    }
}
