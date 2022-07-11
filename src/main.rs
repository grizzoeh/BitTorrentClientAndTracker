use crabrave::{
    client::{Client, ClientInterface},
    parsing::args::get_torrents_paths,
    parsing::config_parser::config_parse,
    parsing::torrent_parser::torrent_parse,
    ui::ui_codes::*,
    utilities::constants::*,
    utilities::utils::{to_gb, UiParams},
};
use gtk::{prelude::*, Builder, Grid, Label, Window, *};
use std::{
    collections::HashMap,
    env,
    sync::mpsc::{channel, Receiver, Sender},
    sync::{Arc, Mutex},
    thread::{spawn, JoinHandle},
};

/// Initializes the entire application.
#[allow(clippy::type_complexity)]
pub fn main() {
    let args: Vec<String> = env::args().collect();
    if (args.len() < 2) || (args.len() > 2) {
        return println!("Incorrect number of arguments");
    }

    let torrent_dir = match args.get(1) {
        Some(dir) => dir,
        None => return println!("Incorrect number of arguments"),
    };

    let torrent_paths: Vec<String> = get_torrents_paths(torrent_dir).unwrap();
    let torrent_paths_aux = torrent_paths.clone();
    let (aux_tx, aux_rx): (
        Sender<glib::Sender<Vec<(usize, UiParams, String)>>>,
        Receiver<glib::Sender<Vec<(usize, UiParams, String)>>>,
    ) = channel();

    let ui_handle = spawn(move || run_ui(aux_tx, torrent_paths));

    let client_sender = aux_rx.recv().unwrap();

    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    let mut port_counter = 0;
    for torrent_path in torrent_paths_aux {
        let torrent_path_aux1 = torrent_path.clone();
        let mut config = config_parse(CONFIG_PATH.to_string()).unwrap();

        config.insert("torrent_path".to_string(), torrent_path.clone());
        let torrent_data = match torrent_parse(&torrent_path) {
            Ok(data) => data,
            Err(_) => {
                continue;
            }
        };

        let ui_sender = Arc::new(Mutex::new(client_sender.clone()));
        let port = LISTENING_PORT + port_counter;
        let (client, logger_handler) =
            Client::create(config, ui_sender, torrent_path_aux1, port, torrent_data).unwrap();

        let (download_handler, listener_handler, upload_handler) = client.start().unwrap();
        handles.push(logger_handler);
        handles.push(download_handler);
        handles.push(listener_handler);
        handles.push(upload_handler);

        port_counter += 1;
    }

    // Waits for the threads to finish
    while !handles.is_empty() {
        match handles.pop() {
            Some(handle) => handle.join().unwrap(),
            None => break,
        };
    }
    // Waits for the UI thread to finish
    ui_handle.join().unwrap();
}

/// Runs the UI.
fn run_ui(
    sender_aux: Sender<glib::Sender<Vec<(usize, UiParams, String)>>>,
    torrent_paths: Vec<String>,
) {
    let application = gtk::Application::new(Some("com.crabrave"), Default::default());
    application.connect_activate(move |application| {
        build_ui(application, sender_aux.clone(), torrent_paths.clone())
    });
    application.run_with_args(&[""]);
}

/// Builds the UI and all of its components.
#[allow(clippy::type_complexity)]
fn build_ui(
    application: &gtk::Application,
    sender_aux: Sender<glib::Sender<Vec<(usize, UiParams, String)>>>,
    torrent_paths: Vec<String>,
) {
    let (sender_client, receiver_client): (
        glib::Sender<Vec<(usize, UiParams, String)>>,
        glib::Receiver<Vec<(usize, UiParams, String)>>,
    ) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    sender_aux.send(sender_client).unwrap();

    let glade_src = include_str!("ui/gtk.tabs.ui");
    let builder = Builder::from_string(glade_src);

    let grid: Grid = builder
        .object("download_details_grid")
        .expect("Couldn't get downloads grid ");

    let buttonbox: ButtonBox = builder
        .object("buttonbox")
        .expect("Couldn't get sum active connections ");

    let mut row_jump = 100;
    let mut labels = HashMap::<String, Vec<Label>>::new();
    let mut buttons = HashMap::<String, Button>::new();

    let tabs_window: Window = builder.object("tabs").expect("Couldn't get tabs_window");
    tabs_window.set_title("Bittorrent Client - CrabRave");
    tabs_window.set_application(Some(application));

    let mut dic_torrents = HashMap::<String, HashMap<String, Vec<String>>>::new();

    let dic_info_torrents = HashMap::from([
        (String::from("downloaded_pieces"), vec![String::from("0")]),
        (String::from("verified_pieces"), vec![String::from("0")]),
        (String::from("peer_number"), vec![String::from("0")]),
        (String::from("active_connections"), vec![String::from("0")]),
        (String::from("verification_hash"), vec![String::from("")]),
        (String::from("total_size"), vec![String::from("0 GB")]),
        (String::from("peers_quantity"), vec![String::from("0")]),
        (String::from("pieces_quantity"), vec![String::from("0")]),
        (String::from("download_speed"), vec![String::from("0")]),
        (String::from("upload_speed"), vec![String::from("0")]),
        (String::from("percentage"), vec![String::from("0")]),
        (String::from("filename"), vec![String::from("")]),
        (String::from("torrents"), torrent_paths.clone()),
    ]);

    let label_id1: Label = gtk::Label::new(Some("ID"));
    let label_ip1 = gtk::Label::new(Some("IP"));
    let label_port1 = gtk::Label::new(Some("Port"));
    let label_download_speed1 = gtk::Label::new(Some("Download Speed"));
    let label_upload_speed1 = gtk::Label::new(Some("Upload Speed"));
    let label_peer_status1 = gtk::Label::new(Some("Peer Status"));
    let label_client_status1 = gtk::Label::new(Some("Client Status"));
    let label_filenam1 = gtk::Label::new(Some("File Name"));

    grid.attach(&label_id1, -8, 0, 9, 13);
    grid.attach(&label_ip1, -7, 0, 10, 13);
    grid.attach(&label_port1, -5, 0, 11, 13);

    grid.attach(&label_download_speed1, 1, 0, 9, 13);
    grid.attach(&label_upload_speed1, 2, 0, 8, 13);

    grid.attach(&label_peer_status1, 5, 0, 3, 13);
    grid.attach(&label_client_status1, 6, 0, 8, 13);
    grid.attach(&label_filenam1, 13, 0, 4, 13);

    tabs_window.show_all();

    for torrent in torrent_paths.clone() {
        let torrent_paths_aux = torrent_paths.clone();

        dic_torrents.insert(torrent.clone(), dic_info_torrents.clone());

        dic_torrents.insert(torrent.clone(), dic_info_torrents.clone());

        let mut split_file = torrent.split('/');
        let last_item = split_file.next_back().unwrap();
        let button = gtk::Button::with_mnemonic(last_item);
        buttonbox.pack_start(&button, false, false, 0);
        let dic_aux = dic_torrents.clone();
        let file_aux = torrent.clone();
        let builder_aux = builder.clone();

        button.connect_clicked(move |_| {
            // Downloaded pieces
            let torrent_hash = dic_aux.get(&file_aux).unwrap();
            let downloaded_pieces_text = &torrent_hash.get("downloaded_pieces").unwrap()[0];
            let download_label: Label = builder_aux
                .object("summary_downloaded_pieces")
                .expect("Couldn't get sum down pieces");

            download_label.set_label(downloaded_pieces_text.as_str());

            // Verified pieces
            let verified_pieces_text = &torrent_hash.get("verified_pieces").unwrap()[0];
            let verified_label: Label = builder_aux
                .object("summary_verified_pieces")
                .expect("Couldn't get sum verified pieces");
            verified_label.set_label(verified_pieces_text.as_str());

            // Active connections
            let active_conns_text = &torrent_hash.get("active_connections").unwrap()[0];
            let active_conns_label: Label = builder_aux
                .object("summary_active_conns")
                .expect("Couldn't get sum active conns");
            active_conns_label.set_label(active_conns_text.as_str());

            // Total size
            let total_size_text = &torrent_hash.get("total_size").unwrap()[0];
            let total_size_label: Label = builder_aux
                .object("summary_total_siz")
                .expect("Couldn't get sum total size");
            total_size_label.set_label(total_size_text.as_str());

            // Peers quantity
            let peers_quantity_text = &torrent_hash.get("peers_quantity").unwrap()[0];
            let peers_quantity_label: Label = builder_aux
                .object("summary_peers_quant")
                .expect("Couldn't get sum peers quantity");
            peers_quantity_label.set_label(peers_quantity_text.as_str());
            // Filename
            let peers_quantity_label: Label = builder_aux
                .object("summary_nam")
                .expect("Couldn't get sum peers quantity");

            peers_quantity_label.set_label(torrent_paths_aux[0].as_str());
            // Progress bar
            let percentage = &torrent_hash.get("percentage").unwrap()[0];
            let progress_bar: ProgressBar = builder_aux
                .object("progressbar")
                .expect("Couldn't get progress bar");
            progress_bar.set_fraction(percentage.parse::<f64>().unwrap());
        });
        buttons.insert(torrent.to_string(), button);
    }
    buttonbox.show_all();

    receiver_client.attach(None, move |msg| {
        let code = &msg[0].0;
        let param = &msg[0].1;
        let current_torrent = &msg[0].2;

        match *code {
            UPDATE_DOWNSPEED => {
                if let UiParams::Vector(vector) = param {
                    let mut dic_aux = HashMap::<String, Vec<String>>::new();
                    let current_torrent_hash = &dic_torrents[current_torrent];

                    for key in current_torrent_hash.keys() {
                        if key == "download_speed" {
                            dic_aux.insert(key.to_string(), vec![vector[1].to_string()]);
                        } else if key == "torrents" {
                            dic_aux.insert(key.to_string(), current_torrent_hash[key].clone());
                        } else {
                            dic_aux.insert(
                                key.to_string(),
                                vec![current_torrent_hash[key][0].to_string()],
                            );
                        }
                    }
                    *dic_torrents.get_mut(current_torrent).unwrap() = dic_aux;
                    let label = &labels.get_mut(&vector[0]).unwrap()[0];
                    label.set_label(vector[1].as_str());
                }
                glib::Continue(true)
            }
            UPDATE_UPSPEED => {
                if let UiParams::Vector(vector) = param {
                    let mut dic_aux = HashMap::<String, Vec<String>>::new();
                    let current_torrent_hash = &dic_torrents[current_torrent];

                    for key in current_torrent_hash.keys() {
                        if key == "upload_speed" {
                            dic_aux.insert(key.to_string(), vec![vector[1].to_string()]);
                        } else if key == "torrents" {
                            dic_aux.insert(key.to_string(), current_torrent_hash[key].clone());
                        } else {
                            dic_aux.insert(
                                key.to_string(),
                                vec![current_torrent_hash[key][0].to_string()],
                            );
                        }
                    }
                    *dic_torrents.get_mut(current_torrent).unwrap() = dic_aux;

                    let label = &labels.get_mut(&vector[0]).unwrap()[1];
                    label.set_label(vector[1].as_str());
                }
                glib::Continue(true)
            }
            DELETE_ONE_ACTIVE_CONNECTION => {
                // Changes the peer status to disconnected
                if let UiParams::Vector(vector) = param {
                    let label = &labels.get_mut(&vector[0]).unwrap();
                    label[0].set_label("0 bytes / sec");
                    label[1].set_label("0 bytes / sec");
                    label[2].set_label("Disconnected");
                    label[3].set_label("Disconnected");
                }

                // Reduces the number of active connections
                let current_torrent_hash = &dic_torrents[current_torrent];
                let mut dic_aux = HashMap::<String, Vec<String>>::new();

                for key in current_torrent_hash.keys() {
                    if key == "active_connections" {
                        let mut aux = current_torrent_hash[key][0].parse::<i32>().unwrap() - 1;
                        if current_torrent_hash[key][0].parse::<i32>().unwrap() < 0 {
                            aux = 0;
                        }

                        dic_aux.insert(key.to_string(), vec![(aux).to_string()]);
                    } else if key == "torrents" {
                        dic_aux.insert(key.to_string(), current_torrent_hash[key].clone());
                    } else {
                        dic_aux.insert(
                            key.to_string(),
                            vec![current_torrent_hash[key][0].to_string()],
                        );
                    }
                }
                let dic_aux_aux = dic_aux.clone();
                *dic_torrents.get_mut(current_torrent).unwrap() = dic_aux;

                for file in dic_aux_aux.get("torrents").unwrap() {
                    let dic_aux = dic_torrents.clone();
                    let file_aux = file.clone();
                    let builder_aux = builder.clone();
                    let button = buttons[file].clone();

                    button.connect_clicked(move |_| {
                        // Downloaded pieces
                        let torrent_hash = dic_aux.get(&file_aux).unwrap();
                        let downloaded_pieces_text =
                            &torrent_hash.get("downloaded_pieces").unwrap()[0];
                        let download_label: Label = builder_aux
                            .object("summary_downloaded_pieces")
                            .expect("Couldn't get sum down pieces");

                        download_label.set_label(downloaded_pieces_text.as_str());

                        // Verified pieces
                        let verified_pieces_text = &torrent_hash.get("verified_pieces").unwrap()[0];
                        let verified_label: Label = builder_aux
                            .object("summary_verified_pieces")
                            .expect("Couldn't get sum verified pieces");
                        verified_label.set_label(verified_pieces_text.as_str());

                        // Active connections
                        let active_conns_text = &torrent_hash.get("active_connections").unwrap()[0];
                        let active_conns_label: Label = builder_aux
                            .object("summary_active_conns")
                            .expect("Couldn't get sum active conns");
                        active_conns_label.set_label(active_conns_text.as_str());

                        // Total size
                        let total_size_text = &torrent_hash.get("total_size").unwrap()[0];
                        let total_size_label: Label = builder_aux
                            .object("summary_total_siz")
                            .expect("Couldn't get sum total size");
                        total_size_label.set_label(total_size_text.as_str());

                        // Peers quantity
                        let peers_quantity_text = &torrent_hash.get("peers_quantity").unwrap()[0];
                        let peers_quantity_label: Label = builder_aux
                            .object("summary_peers_quant")
                            .expect("Couldn't get sum peers quantity");
                        peers_quantity_label.set_label(peers_quantity_text.as_str());

                        // Filename
                        let peers_quantity_text = &torrent_hash.get("filename").unwrap()[0];
                        let peers_quantity_label: Label = builder_aux
                            .object("summary_nam")
                            .expect("Couldn't get sum peers quantity");
                        peers_quantity_label.set_label(peers_quantity_text.as_str());
                        // Progress bar
                        let percentage = &torrent_hash.get("percentage").unwrap()[0];
                        let progress_bar: ProgressBar = builder_aux
                            .object("progressbar")
                            .expect("Couldn't get progress bar");
                        progress_bar.set_fraction(percentage.parse::<f64>().unwrap());
                    });
                }
                buttonbox.show_all();
                glib::Continue(true)
            }
            GET_PIECES_QUANTITY => {
                if let UiParams::Integer(quantity) = param {
                    let current_torrent_hash = &dic_torrents[current_torrent];

                    let mut dic_aux = HashMap::<String, Vec<String>>::new();

                    for key in current_torrent_hash.keys() {
                        if key == "pieces_quantity" {
                            dic_aux.insert(key.to_string(), vec![quantity.to_string()]);
                        } else if key == "torrents" {
                            dic_aux.insert(key.to_string(), current_torrent_hash[key].clone());
                        } else {
                            dic_aux.insert(
                                key.to_string(),
                                vec![current_torrent_hash[key][0].to_string()],
                            );
                        }
                    }
                    let dic_aux_aux = dic_aux.clone();
                    *dic_torrents.get_mut(current_torrent).unwrap() = dic_aux;

                    for file in dic_aux_aux.get("torrents").unwrap() {
                        let dic_aux = dic_torrents.clone();
                        let file_aux = file.clone();
                        let builder_aux = builder.clone();
                        let button = buttons[file].clone();

                        button.connect_clicked(move |_| {
                            // Downloaded pieces
                            let torrent_hash = dic_aux.get(&file_aux).unwrap();
                            let downloaded_pieces_text =
                                &torrent_hash.get("downloaded_pieces").unwrap()[0];
                            let download_label: Label = builder_aux
                                .object("summary_downloaded_pieces")
                                .expect("Couldn't get sum down pieces");

                            download_label.set_label(downloaded_pieces_text.as_str());

                            // Verified pieces
                            let verified_pieces_text =
                                &torrent_hash.get("verified_pieces").unwrap()[0];
                            let verified_label: Label = builder_aux
                                .object("summary_verified_pieces")
                                .expect("Couldn't get sum verified pieces");
                            verified_label.set_label(verified_pieces_text.as_str());

                            // Active connections
                            let active_conns_text =
                                &torrent_hash.get("active_connections").unwrap()[0];
                            let active_conns_label: Label = builder_aux
                                .object("summary_active_conns")
                                .expect("Couldn't get sum active conns");
                            active_conns_label.set_label(active_conns_text.as_str());

                            // Total size
                            let total_size_text = &torrent_hash.get("total_size").unwrap()[0];
                            let total_size_label: Label = builder_aux
                                .object("summary_total_siz")
                                .expect("Couldn't get sum total size");
                            total_size_label.set_label(total_size_text.as_str());

                            // Peers quantity
                            let peers_quantity_text =
                                &torrent_hash.get("peers_quantity").unwrap()[0];
                            let peers_quantity_label: Label = builder_aux
                                .object("summary_peers_quant")
                                .expect("Couldn't get sum peers quantity");
                            peers_quantity_label.set_label(peers_quantity_text.as_str());

                            // Filename
                            let peers_quantity_text = &torrent_hash.get("filename").unwrap()[0];
                            let peers_quantity_label: Label = builder_aux
                                .object("summary_nam")
                                .expect("Couldn't get sum peers quantity");
                            peers_quantity_label.set_label(peers_quantity_text.as_str());
                            // Progress bar
                            let percentage = &torrent_hash.get("percentage").unwrap()[0];
                            let progress_bar: ProgressBar = builder_aux
                                .object("progressbar")
                                .expect("Couldn't get progress bar");
                            progress_bar.set_fraction(percentage.parse::<f64>().unwrap());
                        });
                    }
                    buttonbox.show_all();
                }
                glib::Continue(true)
            }
            UPDATE_INITIAL_DOWNLOADED_PIECES => {
                if let UiParams::Integer(cont_num_downloaded_pieces) = param {
                    let current_torrent_hash = &dic_torrents[current_torrent];

                    let mut dic_aux = HashMap::<String, Vec<String>>::new();

                    for key in current_torrent_hash.keys() {
                        if key == "downloaded_pieces" || key == "verified_pieces" {
                            dic_aux.insert(
                                key.to_string(),
                                vec![cont_num_downloaded_pieces.to_string()],
                            );
                        } else if key == "percentage" {
                            let percentage: f64 = *cont_num_downloaded_pieces as f64
                                / current_torrent_hash.get("pieces_quantity").unwrap()[0]
                                    .parse::<i32>()
                                    .unwrap() as f64;
                            dic_aux.insert(key.to_string(), vec![percentage.to_string()]);
                        } else if key == "torrents" {
                            dic_aux.insert(key.to_string(), current_torrent_hash[key].clone());
                        } else {
                            dic_aux.insert(
                                key.to_string(),
                                vec![current_torrent_hash[key][0].to_string()],
                            );
                        }
                    }
                    let dic_aux_aux = dic_aux.clone();
                    *dic_torrents.get_mut(current_torrent).unwrap() = dic_aux;

                    for file in dic_aux_aux.get("torrents").unwrap() {
                        let dic_aux = dic_torrents.clone();
                        let file_aux = file.clone();
                        let builder_aux = builder.clone();
                        let button = buttons[file].clone();

                        button.connect_clicked(move |_| {
                            // Downloaded pieces
                            let torrent_hash = dic_aux.get(&file_aux).unwrap();
                            let downloaded_pieces_text =
                                &torrent_hash.get("downloaded_pieces").unwrap()[0];
                            let download_label: Label = builder_aux
                                .object("summary_downloaded_pieces")
                                .expect("Couldn't get sum down pieces");

                            download_label.set_label(downloaded_pieces_text.as_str());

                            // Verified pieces
                            let verified_pieces_text =
                                &torrent_hash.get("verified_pieces").unwrap()[0];
                            let verified_label: Label = builder_aux
                                .object("summary_verified_pieces")
                                .expect("Couldn't get sum verified pieces");
                            verified_label.set_label(verified_pieces_text.as_str());

                            // Active connections
                            let active_conns_text =
                                &torrent_hash.get("active_connections").unwrap()[0];
                            let active_conns_label: Label = builder_aux
                                .object("summary_active_conns")
                                .expect("Couldn't get sum active conns");
                            active_conns_label.set_label(active_conns_text.as_str());

                            // Total size
                            let total_size_text = &torrent_hash.get("total_size").unwrap()[0];
                            let total_size_label: Label = builder_aux
                                .object("summary_total_siz")
                                .expect("Couldn't get sum total size");
                            total_size_label.set_label(total_size_text.as_str());

                            // Peers quantity
                            let peers_quantity_text =
                                &torrent_hash.get("peers_quantity").unwrap()[0];
                            let peers_quantity_label: Label = builder_aux
                                .object("summary_peers_quant")
                                .expect("Couldn't get sum peers quantity");
                            peers_quantity_label.set_label(peers_quantity_text.as_str());

                            // Filename
                            let peers_quantity_text = &torrent_hash.get("filename").unwrap()[0];
                            let peers_quantity_label: Label = builder_aux
                                .object("summary_nam")
                                .expect("Couldn't get sum peers quantity");
                            peers_quantity_label.set_label(peers_quantity_text.as_str());
                            // Progress bar
                            let percentage = &torrent_hash.get("percentage").unwrap()[0];
                            let progress_bar: ProgressBar = builder_aux
                                .object("progressbar")
                                .expect("Couldn't get progress bar");
                            progress_bar.set_fraction(percentage.parse::<f64>().unwrap());
                        });
                    }
                    buttonbox.show_all();
                }
                glib::Continue(true)
            }
            UPDATE_DOWNLOADED_PIECES => {
                let current_torrent_hash = &dic_torrents[current_torrent];
                let cont_num_downloaded_pieces = vec![
                    current_torrent_hash["downloaded_pieces"][0]
                        .parse::<i32>()
                        .unwrap()
                        + 1,
                ];

                let mut dic_aux = HashMap::<String, Vec<String>>::new();

                for key in current_torrent_hash.keys() {
                    if key == "downloaded_pieces" {
                        dic_aux.insert(
                            key.to_string(),
                            vec![cont_num_downloaded_pieces[0].to_string()],
                        );
                    } else if key == "percentage" {
                        let percentage: f64 = cont_num_downloaded_pieces[0] as f64
                            / current_torrent_hash.get("pieces_quantity").unwrap()[0]
                                .parse::<i32>()
                                .unwrap() as f64;
                        dic_aux.insert(key.to_string(), vec![percentage.to_string()]);
                    } else if key == "torrents" {
                        dic_aux.insert(key.to_string(), current_torrent_hash[key].clone());
                    } else {
                        dic_aux.insert(
                            key.to_string(),
                            vec![current_torrent_hash[key][0].to_string()],
                        );
                    }
                }
                let dic_aux_aux = dic_aux.clone();
                *dic_torrents.get_mut(current_torrent).unwrap() = dic_aux;

                for file in dic_aux_aux.get("torrents").unwrap() {
                    let dic_aux = dic_torrents.clone();
                    let file_aux = file.clone();
                    let builder_aux = builder.clone();
                    let button = buttons[file].clone();

                    button.connect_clicked(move |_| {
                        // Downloaded pieces
                        let torrent_hash = dic_aux.get(&file_aux).unwrap();
                        let downloaded_pieces_text =
                            &torrent_hash.get("downloaded_pieces").unwrap()[0];
                        let download_label: Label = builder_aux
                            .object("summary_downloaded_pieces")
                            .expect("Couldn't get sum down pieces");

                        download_label.set_label(downloaded_pieces_text.as_str());

                        // Verified pieces
                        let verified_pieces_text = &torrent_hash.get("verified_pieces").unwrap()[0];
                        let verified_label: Label = builder_aux
                            .object("summary_verified_pieces")
                            .expect("Couldn't get sum verified pieces");
                        verified_label.set_label(verified_pieces_text.as_str());

                        // Active connections
                        let active_conns_text = &torrent_hash.get("active_connections").unwrap()[0];
                        let active_conns_label: Label = builder_aux
                            .object("summary_active_conns")
                            .expect("Couldn't get sum active conns");
                        active_conns_label.set_label(active_conns_text.as_str());

                        // Total size
                        let total_size_text = &torrent_hash.get("total_size").unwrap()[0];
                        let total_size_label: Label = builder_aux
                            .object("summary_total_siz")
                            .expect("Couldn't get sum total size");
                        total_size_label.set_label(total_size_text.as_str());

                        // Peers quantity
                        let peers_quantity_text = &torrent_hash.get("peers_quantity").unwrap()[0];
                        let peers_quantity_label: Label = builder_aux
                            .object("summary_peers_quant")
                            .expect("Couldn't get sum peers quantity");
                        peers_quantity_label.set_label(peers_quantity_text.as_str());

                        // Filename
                        let peers_quantity_text = &torrent_hash.get("filename").unwrap()[0];
                        let peers_quantity_label: Label = builder_aux
                            .object("summary_nam")
                            .expect("Couldn't get sum peers quantity");
                        peers_quantity_label.set_label(peers_quantity_text.as_str());
                        // Progress bar
                        let percentage = &torrent_hash.get("percentage").unwrap()[0];
                        let progress_bar: ProgressBar = builder_aux
                            .object("progressbar")
                            .expect("Couldn't get progress bar");
                        progress_bar.set_fraction(percentage.parse::<f64>().unwrap());
                    });
                }
                buttonbox.show_all();

                glib::Continue(true)
            }
            UPDATE_ACTIVE_CONNS => {
                let current_torrent_hash = &dic_torrents[current_torrent];
                let mut dic_aux = HashMap::<String, Vec<String>>::new();

                for key in current_torrent_hash.keys() {
                    if key == "active_connections" {
                        let mut aux = current_torrent_hash[key][0].parse::<i32>().unwrap() + 1;
                        if current_torrent_hash[key][0].parse::<i32>().unwrap() > 50 {
                            aux = 50;
                        }

                        dic_aux.insert(key.to_string(), vec![(aux).to_string()]);
                    } else if key == "torrents" {
                        dic_aux.insert(key.to_string(), current_torrent_hash[key].clone());
                    } else {
                        dic_aux.insert(
                            key.to_string(),
                            vec![current_torrent_hash[key][0].to_string()],
                        );
                    }
                }
                let dic_aux_aux = dic_aux.clone();
                *dic_torrents.get_mut(current_torrent).unwrap() = dic_aux;

                for file in dic_aux_aux.get("torrents").unwrap() {
                    let dic_aux = dic_torrents.clone();
                    let file_aux = file.clone();
                    let builder_aux = builder.clone();
                    let button = buttons[file].clone();

                    button.connect_clicked(move |_| {
                        // Downloaded pieces
                        let torrent_hash = dic_aux.get(&file_aux).unwrap();
                        let downloaded_pieces_text =
                            &torrent_hash.get("downloaded_pieces").unwrap()[0];
                        let download_label: Label = builder_aux
                            .object("summary_downloaded_pieces")
                            .expect("Couldn't get sum down pieces");

                        download_label.set_label(downloaded_pieces_text.as_str());

                        // Verified pieces
                        let verified_pieces_text = &torrent_hash.get("verified_pieces").unwrap()[0];
                        let verified_label: Label = builder_aux
                            .object("summary_verified_pieces")
                            .expect("Couldn't get sum verified pieces");
                        verified_label.set_label(verified_pieces_text.as_str());

                        // Active connections
                        let active_conns_text = &torrent_hash.get("active_connections").unwrap()[0];
                        let active_conns_label: Label = builder_aux
                            .object("summary_active_conns")
                            .expect("Couldn't get sum active conns");
                        active_conns_label.set_label(active_conns_text.as_str());

                        // Total size
                        let total_size_text = &torrent_hash.get("total_size").unwrap()[0];
                        let total_size_label: Label = builder_aux
                            .object("summary_total_siz")
                            .expect("Couldn't get sum total size");
                        total_size_label.set_label(total_size_text.as_str());

                        // Peers quantity
                        let peers_quantity_text = &torrent_hash.get("peers_quantity").unwrap()[0];
                        let peers_quantity_label: Label = builder_aux
                            .object("summary_peers_quant")
                            .expect("Couldn't get sum peers quantity");
                        peers_quantity_label.set_label(peers_quantity_text.as_str());

                        // Filename
                        let peers_quantity_text = &torrent_hash.get("filename").unwrap()[0];
                        let peers_quantity_label: Label = builder_aux
                            .object("summary_nam")
                            .expect("Couldn't get sum peers quantity");
                        peers_quantity_label.set_label(peers_quantity_text.as_str());
                        // Progress bar
                        let percentage = &torrent_hash.get("percentage").unwrap()[0];
                        let progress_bar: ProgressBar = builder_aux
                            .object("progressbar")
                            .expect("Couldn't get progress bar");
                        progress_bar.set_fraction(percentage.parse::<f64>().unwrap());
                    });
                }
                buttonbox.show_all();
                glib::Continue(true)
            }
            UPDATE_TOTAL_SIZE => {
                if let UiParams::U64(size) = param {
                    let current_torrent_hash = &dic_torrents[current_torrent];
                    let mut dic_aux = HashMap::<String, Vec<String>>::new();

                    for key in current_torrent_hash.keys() {
                        if key == "total_size" {
                            dic_aux.insert(key.to_string(), vec![to_gb(*size)]);
                        } else if key == "torrents" {
                            dic_aux.insert(key.to_string(), current_torrent_hash[key].clone());
                        } else {
                            dic_aux.insert(
                                key.to_string(),
                                vec![current_torrent_hash[key][0].to_string()],
                            );
                        }
                    }

                    let dic_aux_aux = dic_aux.clone();
                    *dic_torrents.get_mut(current_torrent).unwrap() = dic_aux;

                    for file in dic_aux_aux.get("torrents").unwrap() {
                        let dic_aux = dic_torrents.clone();
                        let file_aux = file.clone();
                        let builder_aux = builder.clone();
                        let button = buttons[file].clone();

                        button.connect_clicked(move |_| {
                            // Downloaded pieces
                            let torrent_hash = dic_aux.get(&file_aux).unwrap();
                            let downloaded_pieces_text =
                                &torrent_hash.get("downloaded_pieces").unwrap()[0];
                            let download_label: Label = builder_aux
                                .object("summary_downloaded_pieces")
                                .expect("Couldn't get sum down pieces");

                            download_label.set_label(downloaded_pieces_text.as_str());

                            // Verified pieces
                            let verified_pieces_text =
                                &torrent_hash.get("verified_pieces").unwrap()[0];
                            let verified_label: Label = builder_aux
                                .object("summary_verified_pieces")
                                .expect("Couldn't get sum verified pieces");
                            verified_label.set_label(verified_pieces_text.as_str());

                            // Active connections
                            let active_conns_text =
                                &torrent_hash.get("active_connections").unwrap()[0];
                            let active_conns_label: Label = builder_aux
                                .object("summary_active_conns")
                                .expect("Couldn't get sum active conns");
                            active_conns_label.set_label(active_conns_text.as_str());

                            // Total size
                            let total_size_text = &torrent_hash.get("total_size").unwrap()[0];
                            let total_size_label: Label = builder_aux
                                .object("summary_total_siz")
                                .expect("Couldn't get sum total size");
                            total_size_label.set_label(total_size_text.as_str());

                            // Peers quantity
                            let peers_quantity_text =
                                &torrent_hash.get("peers_quantity").unwrap()[0];
                            let peers_quantity_label: Label = builder_aux
                                .object("summary_peers_quant")
                                .expect("Couldn't get sum peers quantity");
                            peers_quantity_label.set_label(peers_quantity_text.as_str());

                            // Filename
                            let peers_quantity_text = &torrent_hash.get("filename").unwrap()[0];
                            let peers_quantity_label: Label = builder_aux
                                .object("summary_nam")
                                .expect("Couldn't get sum peers quantity");
                            peers_quantity_label.set_label(peers_quantity_text.as_str());
                            // Progress bar
                            let percentage = &torrent_hash.get("percentage").unwrap()[0];
                            let progress_bar: ProgressBar = builder_aux
                                .object("progressbar")
                                .expect("Couldn't get progress bar");
                            progress_bar.set_fraction(percentage.parse::<f64>().unwrap());
                        });
                    }
                    buttonbox.show_all();
                }
                glib::Continue(true)
            }
            UPDATE_FILENAME => {
                if let UiParams::String(filename) = param {
                    let current_torrent_hash = &dic_torrents[current_torrent];
                    let mut dic_aux = HashMap::<String, Vec<String>>::new();

                    for key in current_torrent_hash.keys() {
                        if key == "filename" {
                            dic_aux.insert(key.to_string(), vec![filename.to_string()]);
                        } else if key == "torrents" {
                            dic_aux.insert(key.to_string(), current_torrent_hash[key].clone());
                        } else {
                            dic_aux.insert(
                                key.to_string(),
                                vec![current_torrent_hash[key][0].to_string()],
                            );
                        }
                    }
                    let dic_aux_aux = dic_aux.clone();
                    *dic_torrents.get_mut(current_torrent).unwrap() = dic_aux;

                    for file in dic_aux_aux.get("torrents").unwrap() {
                        let dic_aux = dic_torrents.clone();
                        let file_aux = file.clone();
                        let builder_aux = builder.clone();
                        let button = buttons[file].clone();

                        button.connect_clicked(move |_| {
                            // Downloaded pieces
                            let torrent_hash = dic_aux.get(&file_aux).unwrap();
                            let downloaded_pieces_text =
                                &torrent_hash.get("downloaded_pieces").unwrap()[0];
                            let download_label: Label = builder_aux
                                .object("summary_downloaded_pieces")
                                .expect("Couldn't get sum down pieces");

                            download_label.set_label(downloaded_pieces_text.as_str());

                            // Verified pieces
                            let verified_pieces_text =
                                &torrent_hash.get("verified_pieces").unwrap()[0];
                            let verified_label: Label = builder_aux
                                .object("summary_verified_pieces")
                                .expect("Couldn't get sum verified pieces");
                            verified_label.set_label(verified_pieces_text.as_str());

                            // Active connections
                            let active_conns_text =
                                &torrent_hash.get("active_connections").unwrap()[0];
                            let active_conns_label: Label = builder_aux
                                .object("summary_active_conns")
                                .expect("Couldn't get sum active conns");
                            active_conns_label.set_label(active_conns_text.as_str());

                            // Total size
                            let total_size_text = &torrent_hash.get("total_size").unwrap()[0];
                            let total_size_label: Label = builder_aux
                                .object("summary_total_siz")
                                .expect("Couldn't get sum total size");
                            total_size_label.set_label(total_size_text.as_str());

                            // Peers quantity
                            let peers_quantity_text =
                                &torrent_hash.get("peers_quantity").unwrap()[0];
                            let peers_quantity_label: Label = builder_aux
                                .object("summary_peers_quant")
                                .expect("Couldn't get sum peers quantity");
                            peers_quantity_label.set_label(peers_quantity_text.as_str());

                            // Filename
                            let peers_quantity_text = &torrent_hash.get("filename").unwrap()[0];
                            let peers_quantity_label: Label = builder_aux
                                .object("summary_nam")
                                .expect("Couldn't get sum peers quantity");
                            peers_quantity_label.set_label(peers_quantity_text.as_str());
                            // Progress bar
                            let percentage = &torrent_hash.get("percentage").unwrap()[0];
                            let progress_bar: ProgressBar = builder_aux
                                .object("progressbar")
                                .expect("Couldn't get progress bar");
                            progress_bar.set_fraction(percentage.parse::<f64>().unwrap());
                        });
                    }
                    buttonbox.show_all();
                }

                glib::Continue(true)
            }
            UPDATE_PEER_ID_IP_PORT => {
                if let UiParams::Vector(vector) = param {
                    for i in 0..vector.len() {
                        if i == 0 {
                            let label_id: Label = gtk::Label::new(Some(&vector[0]));
                            grid.attach(&label_id, -8, row_jump, 9, 13);
                        }
                        if i == 1 {
                            let label_ip: Label = gtk::Label::new(Some(&vector[1]));
                            grid.attach(&label_ip, -7, row_jump, 10, 13);
                        }
                        if i == 2 {
                            let label_port: Label = gtk::Label::new(Some(&vector[2]));
                            grid.attach(&label_port, -5, row_jump, 11, 13);
                        }
                    }
                }

                let label_down_speed: Label =
                    gtk::Label::new(Some("0 bytes / sec".to_string().as_str()));

                let label_up_speed: Label =
                    gtk::Label::new(Some("0 bytes / sec".to_string().as_str()));

                let label_peer_status: Label = gtk::Label::new(Some("Choked"));

                let label_client_status: Label = gtk::Label::new(Some("Not Interested"));

                grid.attach(&label_down_speed, 1, row_jump, 9, 13);
                grid.attach(&label_up_speed, 2, row_jump, 8, 13);

                grid.attach(&label_peer_status, 5, row_jump, 3, 13);
                grid.attach(&label_client_status, 6, row_jump, 8, 13);

                let mut split_filename = current_torrent.split('/');
                let last_item = split_filename.next_back().unwrap();
                let label_filename: Label = gtk::Label::new(Some(last_item));

                grid.attach(&label_filename, 13, row_jump, 4, 13);

                grid.show_all();
                row_jump += 100;
                if let UiParams::Vector(vector) = param {
                    let id_aux = vector[0].clone();
                    labels.insert(
                        id_aux,
                        vec![
                            label_down_speed,
                            label_up_speed,
                            label_peer_status,
                            label_client_status,
                        ],
                    );
                }
                glib::Continue(true)
            }
            UPDATE_UNCHOKE => {
                if let UiParams::Vector(vector) = param {
                    let label = &labels.get_mut(&vector[0]).unwrap()[2];
                    label.set_label(vector[1].as_str());
                }
                glib::Continue(true)
            }
            UPDATE_INTERESTED => {
                if let UiParams::Vector(vector) = param {
                    let label = &labels.get_mut(&vector[0]).unwrap()[3];
                    label.set_label(vector[1].as_str());
                }
                glib::Continue(true)
            }
            UPDATE_VERIFIED_PIECES => {
                let current_torrent_hash = &dic_torrents[current_torrent];
                let mut dic_aux = HashMap::<String, Vec<String>>::new();

                for key in current_torrent_hash.keys() {
                    if key == "verified_pieces" {
                        dic_aux.insert(
                            key.to_string(),
                            vec![(current_torrent_hash["verified_pieces"][0]
                                .parse::<i32>()
                                .unwrap()
                                + 1)
                            .to_string()],
                        );
                    } else if key == "torrents" {
                        dic_aux.insert(key.to_string(), current_torrent_hash[key].clone());
                    } else {
                        dic_aux.insert(
                            key.to_string(),
                            vec![current_torrent_hash[key][0].to_string()],
                        );
                    }
                }
                let dic_aux_aux = dic_aux.clone();
                *dic_torrents.get_mut(current_torrent).unwrap() = dic_aux;

                for file in dic_aux_aux.get("torrents").unwrap() {
                    let dic_aux = dic_torrents.clone();
                    let file_aux = file.clone();
                    let builder_aux = builder.clone();
                    let button = buttons[file].clone();

                    button.connect_clicked(move |_| {
                        // Downloaded pieces
                        let torrent_hash = dic_aux.get(&file_aux).unwrap();
                        let downloaded_pieces_text =
                            &torrent_hash.get("downloaded_pieces").unwrap()[0];
                        let download_label: Label = builder_aux
                            .object("summary_downloaded_pieces")
                            .expect("Couldn't get sum down pieces");

                        download_label.set_label(downloaded_pieces_text.as_str());

                        // Verified pieces
                        let verified_pieces_text = &torrent_hash.get("verified_pieces").unwrap()[0];
                        let verified_label: Label = builder_aux
                            .object("summary_verified_pieces")
                            .expect("Couldn't get sum verified pieces");
                        verified_label.set_label(verified_pieces_text.as_str());

                        // Active connections
                        let active_conns_text = &torrent_hash.get("active_connections").unwrap()[0];
                        let active_conns_label: Label = builder_aux
                            .object("summary_active_conns")
                            .expect("Couldn't get sum active conns");
                        active_conns_label.set_label(active_conns_text.as_str());

                        // Total size
                        let total_size_text = &torrent_hash.get("total_size").unwrap()[0];
                        let total_size_label: Label = builder_aux
                            .object("summary_total_siz")
                            .expect("Couldn't get sum total size");
                        total_size_label.set_label(total_size_text.as_str());

                        // Peers quantity
                        let peers_quantity_text = &torrent_hash.get("peers_quantity").unwrap()[0];
                        let peers_quantity_label: Label = builder_aux
                            .object("summary_peers_quant")
                            .expect("Couldn't get sum peers quantity");
                        peers_quantity_label.set_label(peers_quantity_text.as_str());

                        // Filename
                        let peers_quantity_text = &torrent_hash.get("filename").unwrap()[0];
                        let peers_quantity_label: Label = builder_aux
                            .object("summary_nam")
                            .expect("Couldn't get sum peers quantity");
                        peers_quantity_label.set_label(peers_quantity_text.as_str());
                        // Progress bar
                        let percentage = &torrent_hash.get("percentage").unwrap()[0];
                        let progress_bar: ProgressBar = builder_aux
                            .object("progressbar")
                            .expect("Couldn't get progress bar");
                        progress_bar.set_fraction(percentage.parse::<f64>().unwrap());
                    });
                }
                buttonbox.show_all();
                glib::Continue(true)
            }
            UPDATE_PEERS_NUMBER => {
                if let UiParams::Usize(peers_number) = param {
                    let current_torrent_hash = &dic_torrents[current_torrent];
                    let mut dic_aux = HashMap::<String, Vec<String>>::new();

                    for key in current_torrent_hash.keys() {
                        if key == "peers_quantity" {
                            dic_aux.insert(key.to_string(), vec![peers_number.to_string()]);
                        } else if key == "torrents" {
                            dic_aux.insert(key.to_string(), current_torrent_hash[key].clone());
                        } else {
                            dic_aux.insert(
                                key.to_string(),
                                vec![current_torrent_hash[key][0].to_string()],
                            );
                        }
                    }
                    let dic_aux_aux = dic_aux.clone();
                    *dic_torrents.get_mut(current_torrent).unwrap() = dic_aux;

                    for file in dic_aux_aux.get("torrents").unwrap() {
                        let dic_aux = dic_torrents.clone();
                        let file_aux = file.clone();
                        let builder_aux = builder.clone();
                        let button = buttons[file].clone();

                        button.connect_clicked(move |_| {
                            // Downloaded pieces
                            let torrent_hash = dic_aux.get(&file_aux).unwrap();
                            let downloaded_pieces_text =
                                &torrent_hash.get("downloaded_pieces").unwrap()[0];
                            let download_label: Label = builder_aux
                                .object("summary_downloaded_pieces")
                                .expect("Couldn't get sum down pieces");

                            download_label.set_label(downloaded_pieces_text.as_str());

                            // Verified pieces
                            let verified_pieces_text =
                                &torrent_hash.get("verified_pieces").unwrap()[0];
                            let verified_label: Label = builder_aux
                                .object("summary_verified_pieces")
                                .expect("Couldn't get sum verified pieces");
                            verified_label.set_label(verified_pieces_text.as_str());

                            // Active connections
                            let active_conns_text =
                                &torrent_hash.get("active_connections").unwrap()[0];
                            let active_conns_label: Label = builder_aux
                                .object("summary_active_conns")
                                .expect("Couldn't get sum active conns");
                            active_conns_label.set_label(active_conns_text.as_str());

                            // Total size
                            let total_size_text = &torrent_hash.get("total_size").unwrap()[0];
                            let total_size_label: Label = builder_aux
                                .object("summary_total_siz")
                                .expect("Couldn't get sum total size");
                            total_size_label.set_label(total_size_text.as_str());

                            // Peers quantity
                            let peers_quantity_text =
                                &torrent_hash.get("peers_quantity").unwrap()[0];
                            let peers_quantity_label: Label = builder_aux
                                .object("summary_peers_quant")
                                .expect("Couldn't get sum peers quantity");
                            peers_quantity_label.set_label(peers_quantity_text.as_str());
                            // Filename
                            let peers_quantity_text = &torrent_hash.get("filename").unwrap()[0];
                            let peers_quantity_label: Label = builder_aux
                                .object("summary_nam")
                                .expect("Couldn't get sum peers quantity");
                            peers_quantity_label.set_label(peers_quantity_text.as_str());
                            // Progress bar
                            let percentage = &torrent_hash.get("percentage").unwrap()[0];
                            let progress_bar: ProgressBar = builder_aux
                                .object("progressbar")
                                .expect("Couldn't get progress bar");
                            progress_bar.set_fraction(percentage.parse::<f64>().unwrap());
                        });
                    }
                    buttonbox.show_all();
                }
                glib::Continue(true)
            }
            _ => glib::Continue(true),
        }
    });

    tabs_window.show();
}
