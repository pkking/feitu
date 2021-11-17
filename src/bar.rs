use chrono::{DateTime, TimeZone, NaiveDateTime, Utc};
use super::tick;

pub struct time_bar {
    dt :DateTime,
    open :u64,
    high:u64,
    low:u64,
    close:u64,
    volume:u64,
}

impl 
pub fn new_bar(t :tick::Tick) -> Option<time_bar> {
    time_bar
}