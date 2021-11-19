

mod tick;
mod trans;
mod strategy;

use strategy::{new_stock_sys, read_tick_from_data};


fn back_testing() {
    let mut sys = new_stock_sys("src/strategy.toml").expect("fail to create new sotck instance");
    let ticks = read_tick_from_data("../601012.SH.Tick.csv").expect("read ticks data failed");

    for tick in ticks {
        sys.do_strategy(&tick);
    }

    sys.statistics();
}

fn main() {
   // println!("config {:?}", &conf);
    //let ticks = printcsv::<tick::Tick>("../601012.SH.Tick.csv").unwrap();

    back_testing();
    //printcsv::<trans::transaction>("../601012.SH.Transaction.csv");
}
