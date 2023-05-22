use std::fs;
use serde::{Deserialize,Serialize};
use serde_json::Result;

const STAT_PATH: &str = "./server/stats.json";

pub fn incr_count(url: &String) {
    // todo: Need to add functionality to read the stats.json file into a json structure and increment count by 1 on request
    // this is called by net_server on a successful get request url parse
}