use csv;
use serde::de::DeserializeOwned;
use std::error::Error;
use std::io;
use std::process;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::fmt::Debug;

mod tick;
mod trans;
mod strategy;

fn printcsv<T:DeserializeOwned + Debug>(path: &str) -> Result<Vec<T>, Box<dyn Error>> {
    let f = File::open(path)?;
    let reader = BufReader::new(f);
    let mut rdr = csv::Reader::from_reader(reader);
    let mut count :u32 = 0;
    let mut res =  Vec::new();
    for result in rdr.deserialize() {
        // Notice that we need to provide a type hint for automatic
        // deserialization.
        let record: T = result?;
        res.push(record);
        count += 1;
    }
    println!("{} for {}", count, std::any::type_name::<T>(),);
    Ok(res)
}

fn main() {
    let conf = strategy::new_config("src/strategy.toml").unwrap();

    println!("config {:?}", &conf);
    //printcsv::<tick::Tick>("../601012.SH.Tick.csv").unwrap();
    //printcsv::<trans::transaction>("../601012.SH.Transaction.csv");
}