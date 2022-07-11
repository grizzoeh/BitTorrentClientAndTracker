# 22C1-Crab-Rave -> BitTorrent Client

## Executing the Client

    To execute the client `config.yml` must be configured.

    Command: cargo run [insert torrents directory path]

    The torrents directory path can contain 1 or more .torrent files. 

## Executing AppServer (to test seeder mode)

    Must be executed during the Client execution.
    
    Command: cargo run --bin app_server
