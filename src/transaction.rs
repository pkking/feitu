use chrono::{DateTime, FixedOffset, TimeZone};
use serde::Deserialize;
use super::tick::{default_dt, get_time};
use std::collections::HashMap;
use std::io::BufReader;
use std::fs::File;
use std::error::Error;

#[derive(Debug, Deserialize, Clone)]
pub struct transaction {
    pub Tkr: String,
    pub Time: u64,
    #[serde(skip_deserializing)]
    #[serde(default = "default_dt")]
    pub dt: DateTime<FixedOffset>,
    pub Index: u64,
    pub Price: u64,
    pub Volume: u64,
    pub Turnover: u64,
    pub BSFlag: char,
    pub OrderKind: u64,
    pub FunctionCode: u64,
    pub AskOrder: u64,
    pub BidOrder: u64,
}

pub fn read_trans_data_from_file(path :&str) -> Result<HashMap<u64, transaction>, Box<dyn Error>> {
    let mut res: HashMap<u64, transaction> = HashMap::new();
    let f = File::open(path)?;
    let reader = BufReader::new(f);
    let mut rdr = csv::Reader::from_reader(reader);
    for result in rdr.deserialize() {
        // Notice that we need to provide a type hint for automatic
        // deserialization.
        let mut record: transaction = result?;
        record.dt = get_time(record.Time);
        res.insert(record.Time, record);
    }
    Ok(res)
}