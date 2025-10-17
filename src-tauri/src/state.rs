use crate::m3u_parser::Channel;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::SystemTime;

pub struct DbState {
    pub db: Mutex<Connection>,
}

#[derive(Debug, Clone)]
pub struct ChannelCache {
    pub channel_list_id: Option<i32>,
    pub channels: Vec<Channel>,
    pub last_updated: SystemTime,
}

pub struct ChannelCacheState {
    pub cache: Mutex<Option<ChannelCache>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChannelList {
    pub id: i32,
    pub name: String,
    pub source: String,
    pub is_default: bool,
    pub filepath: Option<String>,
    pub last_fetched: Option<i64>,
}
