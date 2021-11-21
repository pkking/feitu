use chrono::{DateTime, Duration, FixedOffset};
use csv;
use serde::Deserialize;
use simple_log::LogConfigBuilder;
use std::cmp::max;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display};
use std::io::BufReader;
use std::io::Read;
use std::{fs::File, u64::MAX, u64::MIN};

use crate::tick::Tick;

use super::tick;
use super::transaction;

#[derive(Debug)]
pub struct order {
    open_price: u64,
    time: DateTime<FixedOffset>,
    selt_time: DateTime<FixedOffset>,
    volume: usize,
    sell_price: u64,
    want_sell_all: bool,
    sell_price_avg: u64,
    left: usize,
    profit: i128,
    tax: u64,
    commission: u64,
}
#[derive(Debug, Deserialize)]
pub struct config {
    buy_point: f64,
    gap_window: i64,
    buy_volume: usize,
    buy_cooldown_time: i64,
    sell_delay_time: i64,
    sell_all_delay: i64,
    log_level: String,
    log_file: String,
    log_size: u64,
    log_count: u32,
    pub tick_data: String,
    pub trans_data: String,
}

// 基本思路：
// 维护一个滑动窗口，计算最大涨幅，并出发下单操作
// 个人理解的【市价委托】，是在当前tick时间内，按照买/卖1~10的价格顺序依次撮合交易
// 买卖需要考虑涨跌停
pub struct StockSys {
    pub conf: config,
    pub orders: Vec<order>,
    //pub last_buy_order: usize,
    //pub last_sell_order: usize,
    //pub last_sell_idx: usize,
    //pub last_buy_idx: usize,
    pub gap_window: Vec<tick::Tick>,
    pub gap_rate: f64,
    //pub min_idx: usize,
    //pub max_idx: usize,
    pub min: u64,
    pub max: u64,
    pub trans: HashMap<u64, transaction::transaction>,
}

pub fn new_config(path: &str) -> Result<config, Box<dyn Error>> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let conf: config = toml::from_str(&contents).unwrap();
    Ok(conf)
}

pub fn new_stock_sys(config: &str) -> Result<StockSys, Box<dyn Error>> {
    Ok(StockSys {
        conf: new_config(config)?,
        orders: Vec::new(),
        //last_buy_order: 0,
        //last_sell_order: 0,
        //last_sell_idx: 0,
        //last_buy_idx: 0,
        gap_window: Vec::new(),
        gap_rate: 0.0,
        //max_idx: 0,
        //min_idx: 0,
        max: MIN,
        min: MAX,
        trans: HashMap::new(),
    })
}

pub fn read_tick_from_data(path: &str) -> Result<Vec<tick::Tick>, Box<dyn Error>> {
    let mut res: Vec<tick::Tick> = Vec::new();
    let f = File::open(path)?;
    let reader = BufReader::new(f);
    let mut rdr = csv::Reader::from_reader(reader);
    for result in rdr.deserialize() {
        // Notice that we need to provide a type hint for automatic
        // deserialization.
        let mut record: tick::Tick = result?;
        record.dt = tick::get_time(record.nTime);
        res.push(record);
    }
    Ok(res)
}

impl Display for order {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "open price:{} sell price:{} buy time:{} sell time:{} volume:{} left:{} profit:{} tax:{} commission:{}", self.open_price, self.sell_price_avg, self.time, self.selt_time, self.volume, self.left, self.profit, self.tax, self.commission)?;
        Ok(())
    }
}

impl StockSys {
    pub fn init_logger(&self) {
        let config = LogConfigBuilder::builder()
            .path(self.conf.log_file.to_string())
            .size(self.conf.log_size)
            .roll_count(self.conf.log_count)
            .level(self.conf.log_level.to_string())
            .output_file()
            .build();

        simple_log::new(config).expect("failed to init log config");
    }
    // 输出一些统计信息
    pub fn statistics(&self) {
        let mut profit: i128 = 0;
        let mut tax_commission: u64 = 0;
        let mut win_orders: Vec<&order> = Vec::new();
        let mut lose_orders: Vec<&order> = Vec::new();

        for order in &self.orders {
            if order.profit > 0 {
                win_orders.push(order);
            } else {
                lose_orders.push(order);
            }
            profit += order.profit;
            tax_commission += order.tax + order.commission;
        }
        info!("profit :{}", profit);
        info!(
            "profit with tax commission:{}",
            profit - tax_commission as i128
        );
        info!("profit wins:");
        for order in win_orders {
            info!("{}", order);
        }
        info!("profit lose:");
        for order in lose_orders {
            info!("{}", order);
        }
    }
    // 刷新最大涨幅
    fn get_gap(&mut self, tick: &tick::Tick) {
        let mut min = MAX;
        let mut min_idx: usize = 0;

        if self.gap_window.len() == 0 {
            return;
        }
        // gap rate = now price - min price / min price

        for (idx, i) in self.gap_window.iter().enumerate() {
            if min > i.nPrice {
                min = i.nPrice;
                min_idx = idx;
            }
        }

        self.min = min;
        self.gap_window.drain(..min_idx);
        self.gap_rate = (tick.nPrice as f64 - self.min as f64) / self.min as f64;
        //debug!(
        //    "{} new gap_rate:{} price {} min {}",
        //    tick.dt, self.gap_rate, tick.nPrice, self.min
        //);
    }
    // 判断是否可以交易的条件：
    // 1. 是否在交易时间段
    // TODO:暂时不考虑T+0限制
    fn can_trade(&self, tick: &tick::Tick) -> bool {
        return (tick.dt >= *tick::START_TIME_MORNINIG && tick.dt <= *tick::END_TIME_MORNINIG)
            || (tick.dt >= *tick::START_TIME_AFTERNOON && tick.dt <= *tick::END_TIME_AFTERNOON);
    }

    pub fn do_strategy(&mut self, tick: &tick::Tick) {
        if self.can_trade(tick) {
            self.update_gap(tick);
            self.process_order(tick);
        }
    }
    // 下单逻辑，买单需要考虑卖单的数量能否撮合
    // TODO:价格是否应该参考trans里的内容
    fn buy(&mut self, tick: &tick::Tick) {
        let mut value: u64 = 0;
        let mut left: u64 = self.conf.buy_volume as u64;
        for (p, v) in [
            (tick.nAskPrice1, tick.nAskVolume1),
            (tick.nAskPrice2, tick.nAskVolume2),
            (tick.nAskPrice3, tick.nAskVolume3),
            (tick.nAskPrice4, tick.nAskVolume4),
            (tick.nAskPrice5, tick.nAskVolume5),
            (tick.nAskPrice6, tick.nAskVolume6),
            (tick.nAskPrice7, tick.nAskVolume7),
            (tick.nAskPrice8, tick.nAskVolume8),
            (tick.nAskPrice9, tick.nAskVolume9),
            (tick.nAskPrice10, tick.nAskVolume10),
        ]
        .iter()
        {
            if left <= *v {
                debug!("{} buy at {} price {} ", tick.dt, left, p);
                value += left * (*p);
                break;
            } else {
                left = left - *v;
                debug!("{} buy at {} price {}", tick.dt, *v, p);
                value += *v * *p;
            }
        }

        self.orders.push(order {
            open_price: value / self.conf.buy_volume as u64,
            time: tick::get_time(tick.nTime),
            volume: self.conf.buy_volume,
            sell_price: 0,
            left: self.conf.buy_volume,
            profit: 0,
            sell_price_avg: 0,
            tax: 0,
            commission: 0,
            want_sell_all: false,
            selt_time: tick::default_dt(),
        });
        //self.last_buy_order = self.orders.len() - 1;
        self.gap_window.clear();
        self.min = MAX;
        self.max = MIN;
        //self.min_idx = 0;
        //self.max_idx = 0;
    }

    // TODO:暂时不考虑买卖影响股价，不拆分订单
    fn sell(&mut self, tick: &tick::Tick) {
        // 挂单卖出不用考虑跌停的情况
        for order in &mut self.orders {
            if tick.dt - order.time > Duration::seconds(self.conf.sell_delay_time) && order.left > 0
            {
                if order.sell_price == 0 {
                    debug!(
                        "{} begin to sell at price:{} (buy time:{}) after {}s",
                        tick.dt, tick.nAskPrice1, order.time, self.conf.sell_delay_time
                    );
                    order.sell_price = tick.nAskPrice1; // 以卖1挂卖单
                }
                // 超过时间没有卖完，需要尽量卖出
                // 这里的想法是从当前tick开始，尝试所有的买单，直到卖完
                // TODO: 是否成交需要参考trans
                if tick.dt - order.time
                    > Duration::seconds(self.conf.sell_all_delay)
                        + Duration::seconds(self.conf.sell_delay_time)
                    && order.want_sell_all == false
                {
                    debug!(
                        "{} change price to {} (buy time:{}) to sell left {} after {}s",
                        tick.dt, tick.nPrice, order.time, order.left, self.conf.sell_all_delay
                    );
                    order.sell_price = tick.nPrice;
                    order.want_sell_all = true;
                }
                // 尝试所有的卖价，争取一次卖出
                for (p, v) in [
                    (tick.nBidPrice1, tick.nBidVolume1),
                    (tick.nBidPrice2, tick.nBidVolume2),
                    (tick.nBidPrice3, tick.nBidVolume3),
                    (tick.nBidPrice4, tick.nBidVolume4),
                    (tick.nBidPrice5, tick.nBidVolume5),
                    (tick.nBidPrice6, tick.nBidVolume6),
                    (tick.nBidPrice7, tick.nBidVolume7),
                    (tick.nBidPrice8, tick.nBidVolume8),
                    (tick.nBidPrice9, tick.nBidVolume9),
                    (tick.nBidPrice10, tick.nBidVolume10),
                ]
                .iter()
                {
                    // 跌停了不再交易
                    if tick.nPrice == tick.LowLimited * 10 && order.want_sell_all {
                        break;
                    }
                    if *p >= order.sell_price || order.want_sell_all {
                        if *v >= order.left as u64 {
                            debug!(
                                "{} sell {} price {} want {}",
                                tick.dt, order.left, p, order.want_sell_all
                            );
                            order.profit += order.left as i128 * *p as i128; // 先计算总的收入
                            order.sell_price_avg = order.profit as u64 / order.volume as u64; // 算出平均卖价
                            order.profit -= order.open_price as i128 * order.volume as i128; // 减去买入成本
                            order.tax = order.sell_price_avg * order.volume as u64 / 1000; // 减去印花税 1/1000
                            order.commission =
                                max(
                                    order.sell_price_avg * order.volume as u64 * 3 / 10000,
                                    50000,
                                ) + max(order.open_price * order.volume as u64 * 3 / 10000, 50000); // 减去佣金 3/10000
                            order.left = 0;
                            order.selt_time = tick.dt;
                            debug!("{} sell order:{}", tick.dt, order);
                            break;
                        } else {
                            debug!(
                                "{} sell {} price {} want {}",
                                tick.dt, v, p, order.want_sell_all
                            );
                            order.profit += *v as i128 * *p as i128;
                            order.left -= *v as usize;
                        }
                        //debug!("sell order:{:?}", order);
                    }
                }
            }
        }
    }
    // 能否下单的判断方法：
    // 在交易后的冷却时间内不能下单
    // 涨幅是达到阈值了才下单
    // 涨停时不能买
    fn can_buy(&self, tick: &tick::Tick) -> bool {
        if tick.nPrice == tick.HighLimited * 10 {
            return false;
        }
        if self.conf.buy_point < self.gap_rate {
            match self.orders.last() {
                Some(buy_order) => {
                    // 两次买入间隔大于 buy_cooldown_time 秒
                    let buy =
                        tick.dt - buy_order.time > Duration::seconds(self.conf.buy_cooldown_time);
                    match buy {
                        true => debug!(
                            "{} will buy (min price time {}, gap {}) price {} when time after buy time:{} + cold time:{}",
                            tick.dt, self.gap_window.get(0).unwrap().dt, self.gap_rate, tick.nPrice, buy_order.time, self.conf.buy_cooldown_time
                        ),
                        false => debug!(
                            "{} will not buy price {} when time in buy time:{} + cold time:{}",
                            tick.dt, tick.nPrice, buy_order.time, self.conf.buy_cooldown_time
                        ),
                    }
                    return buy;
                }
                None => {
                    debug!(
                        "{} first buy price {} when min price time is {} gap is {}",
                        tick.dt,
                        tick.nPrice,
                        self.gap_window.get(0).unwrap().dt,
                        self.gap_rate
                    );
                    return true;
                }
            }
        }
        false
    }
    fn process_order(&mut self, tick: &tick::Tick) {
        if self.can_buy(tick) {
            self.buy(tick);
        }
        self.sell(tick);
    }
    // 每次tick到达时，更新时间窗内的最大涨幅
    // 时间窗以最低价为起点，当前价为终点
    // TODO：将一定量的tick聚合到一个bar结构里
    fn update_gap(&mut self, tick: &tick::Tick) {
        //let mut need_update_gap = false;

        self.gap_window.push(tick.clone());
        if self.gap_window.last().unwrap().dt - self.gap_window.first().unwrap().dt
            >= Duration::seconds(self.conf.gap_window)
        {
            self.gap_window.remove(0);
            //if self.min_idx == 0 || self.max_idx == 0 {
            //need_update_gap = true;
            //debug!("remove {:?} min {} and low {} max {} and high {} ", first, first.Low, self.min, first.High, self.max);
            //}
        }
        if tick.nPrice > self.max {
            self.max = tick.nPrice;
            //self.max_idx = self.gap_window.len() - 1;
            //need_update_gap = true;
        }
        if tick.nPrice < self.min {
            self.min = tick.nPrice;
            //  need_update_gap = true;
            self.gap_window.clear();
            self.gap_window.push(tick.clone());
            //self.min_idx = 0;
        }
        //if need_update_gap {
        //debug!("for tick window:{:?}", &self.gap_window);
        self.get_gap(tick);
        //}
        //debug!(":{:?}", &self.gap_window);
    }
}
