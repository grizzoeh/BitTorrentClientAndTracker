pub const GET_STATS: &[u8; 21] = b"GET /stats HTTP/1.1\r\n";
pub const GET_ANNOUNCE: &[u8; 14] = b"GET /announce?";
pub const GET_STATS_DATA: &[u8; 15] = b"GET /stats/data";
pub const LISTENER_IP: &str = "127.0.0.1";
pub const LISTENER_PORT: u16 = 8088;
pub const LOG_PATH: &str = "src/reports/logs.txt";
pub const DEFAULT_NUNWANT_VALUE: u32 = 50;
pub const DEFAULT_COMPACT_VALUE: u16 = 0;
pub const DEFAULT_NO_PEER_ID: u16 = 0;
pub const DEFAULT_KEY: &str = "";
pub const DEFAULT_TRACKERID: &str = "";
pub const DATA_MANAGER_PATH: &str = "src/data/tracker_data.json";
pub const SAVE_PEER: i64 = 0;
pub const STATS_JS: &str = "src/pages/stats.js";
pub const GET_STATS_JS: &[u8; 24] = b"GET /stats.js HTTP/1.1\r\n";
pub const ACCEPT_SLEEP_TIME: u64 = 1;
