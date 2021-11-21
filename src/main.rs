#[macro_use]
extern crate log;

mod strategy;
mod tick;
mod transaction;

use strategy::{new_stock_sys, read_tick_from_data};
use transaction::read_trans_data_from_file;

fn back_testing() {
    let mut sys = new_stock_sys("src/strategy.toml").expect("fail to create new sotck instance");
    sys.init_logger();
    let ticks = read_tick_from_data(&sys.conf.tick_data).expect("read ticks data failed!");
   // sys.trans = read_trans_data_from_file(&sys.conf.trans_data).expect("read transaction data failed!");

    for tick in ticks {
        sys.do_strategy(&tick);
    }

    sys.statistics();
}

fn main() {
    back_testing();
}
