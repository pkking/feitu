use serde::Deserialize;
use std::convert::TryInto;
use std::f64::{MAX as MAXF64, MIN as MINF64};
use std::u64::MAX;
use std::{fs::File, u64::MIN};
use std::error::Error;
use std::fmt::Debug;
use std::io::Read;
use chrono::{DateTime, TimeZone, NaiveDateTime, Utc};
use csv;
use std::io::BufReader;

use super::tick;
#[derive(Debug)]
pub struct order {
    open_price :u64,
    time: u64,
    volume: usize,
    sell_price :u64,
    left :usize,
    profit :u64,
}
#[derive(Debug, Deserialize)]
pub struct config {
    buy_point :f64,
    gap_window :u64,
    // 1 market price
    // 2 Ask/BidPrice1
    // .. 
    // 11 Ask/BidPrice10
    buy_price :u64,
    buy_volume :usize,
    
    buy_cold_time :u64,
    sell_delay_time :u64,
    // 1 passive
    // 2 active
    sell_type :u64,
    sell_price :u64,
    sell_all_type :u64,
    sell_all_delay :u64,
}

pub struct StockSys {
    pub conf: config,
    pub orders: Vec<order>,
    pub last_buy_order :usize,
    pub last_sell_order :usize,
    pub last_sell_idx :usize,
    pub last_buy_idx :usize,
    pub gap_window :Vec<tick::Tick>,
    pub gap_rate: f64,
    pub min_idx : usize,
    pub max_idx : usize,
    pub min : u64,
    pub max : u64,
}

pub fn new_config(path: &str) -> Result<config, Box<dyn Error>> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let conf :config = toml::from_str(&contents).unwrap();
    Ok(conf)
}

pub fn new_stock_sys(config :&str) -> Result<StockSys, Box<dyn Error>> {
    Ok(StockSys {
        conf: new_config(config)?,
        orders: Vec::new(),
        last_buy_order :0,
        last_sell_order :0,
        last_sell_idx :0,
        last_buy_idx :0,
        gap_window: Vec::new(),
        gap_rate :0.0,
        max_idx :0,
        min_idx :0,
        max :MIN,
        min :MAX,
    })
}



pub fn read_tick_from_data(path :&str) -> Result<Vec<tick::Tick>, Box<dyn Error>> {
    let mut res :Vec<tick::Tick> = Vec::new();
    let f = File::open(path)?;
    let reader = BufReader::new(f);
    let mut rdr = csv::Reader::from_reader(reader);
    for result in rdr.deserialize() {
        // Notice that we need to provide a type hint for automatic
        // deserialization.
        let record = result?;
        res.push(record);
    }
    Ok(res)
}

impl StockSys {
    fn get_gap(&mut self) {
        let mut min = MAX;
        //let mut max = MIN;
        if self.gap_window.len() == 0 {
            return;
        }
        // gap rate = now price - min price / min price

        for (idx, i) in self.gap_window.iter().enumerate() {
            if min > i.nPrice  {
                min = i.nPrice;
                self.min_idx = idx;
            }
        }
        //self.max = max;
        self.min = min;
        self.gap_window.drain(..self.min_idx);
    }
    // 判断是否可以交易的条件：
    // 1. 是否在交易时间段
    fn can_trade(&self, tick :&tick::Tick) -> bool {
        //return (now >= 93000000 && now <= 113000000) || (now >= 130000000 && now <= 150000000);
        return tick.nTime >= 93000000 && tick.nTime <= 93300000 ;
    }

    pub fn do_strategy(&mut self, tick :&tick::Tick) {
        if self.can_trade(tick) {
            println!("will do strategy {} now price:{}", tick.nTime, tick.nPrice);
            self.update_gap(tick);
            self.process_order(tick);
        }
    }
    // 下单逻辑，买单需要考虑卖单的数量能否撮合
    fn buy(&mut self, tick :&tick::Tick) {
        let mut value: u64 = 0;
        let mut left :u64 = self.conf.buy_volume as u64;
        for (p, v) in [(tick.nAskPrice1,tick.nAskVolume1), (tick.nAskPrice2, tick.nAskVolume2),(tick.nAskPrice3,tick.nAskVolume3), (tick.nAskPrice4, tick.nAskVolume4),(tick.nAskPrice5,tick.nAskVolume5), (tick.nAskPrice6, tick.nAskVolume6),(tick.nAskPrice7,tick.nAskVolume7), (tick.nAskPrice8, tick.nAskVolume8),(tick.nAskPrice9,tick.nAskVolume9), (tick.nAskPrice10, tick.nAskVolume10)].iter() {
            if *p < self.conf.buy_price {
                println!("price {} lower than buy price {}", p, self.conf.buy_price);
                continue;
            }
            if left <= *v {
                println!("buy {} at price {}", left, p);
                value += left  * (*p);
                break;
            } else {
                left= left - *v;
                println!("buy {} at price {}", *v, p);
                value += *v * *p;
            }
        }

        self.orders.push(
            order { open_price: value/self.conf.buy_volume as u64, time: tick.nTime, volume: self.conf.buy_volume , sell_price: 0, left: self.conf.buy_volume, profit: 0}
        );
        self.last_buy_order = self.orders.len() - 1;
        self.gap_window.clear();
        self.min = MAX;
        self.max = MIN;
        self.min_idx = 0;
        self.max_idx = 0;
    }
    fn can_sell(&self, tick :&tick::Tick) -> bool {
        // 挂单卖出不用考虑跌停的情况
        for order in &self.orders {
            if tick.nTime > order.time + self.conf.sell_delay_time {
                println!("sell order:{:?}", order);
                return true;
            }
        }
        false
    }
    // TODO:暂时不考虑买卖不影响股价，否则可能要使用一些平滑的方法去买卖
    fn sell(&mut self, tick :&tick::Tick) {
        // 挂单卖出不用考虑跌停的情况
        for order in &mut self.orders {
            if tick.nTime > order.time + self.conf.sell_delay_time && order.left > 0{
                if order.sell_price == 0 {
                    order.sell_price = tick.nAskPrice1;
                }
                for (p, v) in [(tick.nBidPrice1,tick.nBidVolume1), (tick.nBidPrice2, tick.nBidVolume2),(tick.nBidPrice3,tick.nBidVolume3), (tick.nBidPrice4, tick.nBidVolume4),(tick.nBidPrice5,tick.nBidVolume5), (tick.nBidPrice6, tick.nBidVolume6),(tick.nBidPrice7,tick.nBidVolume7), (tick.nBidPrice8, tick.nBidVolume8),(tick.nBidPrice9,tick.nBidVolume9), (tick.nBidPrice10, tick.nBidVolume10)].iter() {
                    if *p >= order.sell_price {
                        println!("sell order:{:?}", order);
                        if *v >= order.left as u64 {
                            println!("sell {} at price {}", order.left, p);
                            order.profit += order.left as u64 * *p;
                            order.left = 0;
                            break;
                        } else {
                            println!("sell {} at price {}", v, p);
                            order.profit += *v * *p;
                            order.left -= *v as usize;
                        }
                    }
                }

            }
        }
    }
    // 能否下单的判断方法：
    // 在交易后的冷却时间内不能下单
    // 涨幅是达到阈值了才下单
    // 涨停时不能买
    fn can_buy(&self, tick :&tick::Tick) -> bool {
        if tick.nPrice == tick.HighLimited {
            return false;
        }
        if self.conf.buy_point < self.gap_rate {
            match self.orders.get(self.last_buy_order) {
                Some(buy_order) => {
                    let buy = tick.nTime > buy_order.time + self.conf.buy_cold_time;
                    match buy {
                        true => println!("will buy at {} price {} when time after buy time:{} + cold time:{}", tick.nTime, tick.nPrice, buy_order.time, self.conf.buy_cold_time),
                        false => println!("will not buy at {} price {} when time in buy time:{} + cold time:{}", tick.nTime, tick.nPrice, buy_order.time, self.conf.buy_cold_time)
                    }
                    return buy;
                },
                None => {println!("first buy at {} price {} when buy time is none", tick.nTime, tick.nPrice);return true;}
            }
        }
        false
    }
    fn process_order(&mut self, tick :&tick::Tick) {
        if self.can_buy(tick) {
            self.buy(tick);
        }
        self.sell(tick);
    }
    // 每次tick到达时，更新最大涨幅，时间窗口未10min
    fn update_gap(&mut self, tick :&tick::Tick) {
        let mut need_update_gap = false;

        self.gap_window.push(tick.clone());
        if self.gap_window.last().unwrap().nTime - self.gap_window.first().unwrap().nTime >= self.conf.gap_window {
            self.gap_window.remove(0);
            if self.min_idx == 0 || self.max_idx == 0 {
                need_update_gap = true;
                //println!("remove {:?} min {} and low {} max {} and high {} ", first, first.Low, self.min, first.High, self.max);
            }
        }
        if tick.nPrice > self.max {
            self.max = tick.nPrice;
            self.max_idx = self.gap_window.len()-1;
            //need_update_gap = true;
        }
        if tick.nPrice < self.min {
            self.min = tick.nPrice;
            need_update_gap = true;
            self.gap_window.clear();
            self.gap_window.push(tick.clone());
            self.min_idx = 0;
        }
        if need_update_gap {
            //println!("for tick window:{:?}", &self.gap_window);
            self.get_gap();
        }
        self.gap_rate = (tick.nPrice as f64 - self.min as f64)/self.min as f64;
        println!("new gap_rate:{} at {} price {} min {}", self.gap_rate, tick.nTime, tick.nPrice, self.min);
    }
}
