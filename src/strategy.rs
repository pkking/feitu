use serde::Deserialize;
use std::fs::File;
use std::error::Error;
use std::fmt::Debug;
use std::io::Read;

#[derive(Debug, Deserialize)]
pub struct config {
    raise_water_level :f64,
    raise_window :u32,
    // 1 market price
    // 2 Ask/BidPrice1
    // .. 
    // 11 Ask/BidPrice10
    buy_price :u32,
    buy_volume :u32,
    
    buy_cold_time :u32,
    sell_delay_time :u32,
    // 1 passive
    // 2 active
    sell_type :u32,
    sell_price :u32,
    sell_all_type :u32,
    sell_all_delay :u32,
}

pub fn new_config(path: &str) -> Result<config, Box<dyn Error>> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let conf :config = toml::from_str(&contents).unwrap();
    Ok(conf)
}