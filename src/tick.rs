use chrono::{DateTime, FixedOffset, TimeZone};
use lazy_static::lazy_static;
use serde::Deserialize;

lazy_static! {
    // 随便编的日期，没有找到只包含Hour:Minute:Second的日期库
    static ref YEAR: i32 = 2021;
    static ref MONTH: i32 = 10;
    static ref DAY: i32 = 30;
    pub static ref START_TIME_MORNINIG: DateTime<FixedOffset> = FixedOffset::east(8 * 60 * 60)
        .ymd(*YEAR, *MONTH as u32, *DAY as u32)
        .and_hms(9, 30, 0);
    pub static ref END_TIME_MORNINIG: DateTime<FixedOffset> = FixedOffset::east(8 * 60 * 60)
        .ymd(*YEAR, *MONTH as u32, *DAY as u32)
        .and_hms(11, 30, 0);
    pub static ref START_TIME_AFTERNOON: DateTime<FixedOffset> = FixedOffset::east(8 * 60 * 60)
        .ymd(*YEAR, *MONTH as u32, *DAY as u32)
        .and_hms(13, 0, 0);
    pub static ref END_TIME_AFTERNOON: DateTime<FixedOffset> = FixedOffset::east(8 * 60 * 60)
        .ymd(*YEAR, *MONTH as u32, *DAY as u32)
        .and_hms(15, 0, 0);
}
pub fn get_time(ntime: u64) -> DateTime<FixedOffset> {
    // 91003000 = 9:10:03
    let pst = FixedOffset::east(8 * 60 * 60);
    //println!("hms is :{} {} {}", (ntime/10000000) as u32, (ntime%10000000/100000) as u32, (ntime%100000/1000) as u32);
    let dt = pst.ymd(*YEAR, *MONTH as u32, *DAY as u32).and_hms(
        (ntime / 10000000) as u32,
        (ntime % 10000000 / 100000) as u32,
        (ntime % 100000 / 1000) as u32,
    );
    dt
}

pub fn default_dt() -> DateTime<FixedOffset> {
    FixedOffset::east(8 * 60 * 60)
        .ymd(1970, 1, 1)
        .and_hms(0, 0, 1)
}

#[derive(Debug, Deserialize, Clone)]
pub struct Tick {
    pub chWindCode: String,
    pub nTime: u64,
    pub Status: u64,
    pub PreClose: u64,
    pub Open: u64,
    pub High: u64,
    pub Low: u64,
    pub nPrice: u64,
    pub nAskPrice1: u64,
    pub nAskPrice2: u64,
    pub nAskPrice3: u64,
    pub nAskPrice4: u64,
    pub nAskPrice5: u64,
    pub nAskPrice6: u64,
    pub nAskPrice7: u64,
    pub nAskPrice8: u64,
    pub nAskPrice9: u64,
    pub nAskPrice10: u64,
    pub nAskVolume1: u64,
    pub nAskVolume2: u64,
    pub nAskVolume3: u64,
    pub nAskVolume4: u64,
    pub nAskVolume5: u64,
    pub nAskVolume6: u64,
    pub nAskVolume7: u64,
    pub nAskVolume8: u64,
    pub nAskVolume9: u64,
    pub nAskVolume10: u64,
    pub nBidPrice1: u64,
    pub nBidPrice2: u64,
    pub nBidPrice3: u64,
    pub nBidPrice4: u64,
    pub nBidPrice5: u64,
    pub nBidPrice6: u64,
    pub nBidPrice7: u64,
    pub nBidPrice8: u64,
    pub nBidPrice9: u64,
    pub nBidPrice10: u64,
    pub nBidVolume1: u64,
    pub nBidVolume2: u64,
    pub nBidVolume3: u64,
    pub nBidVolume4: u64,
    pub nBidVolume5: u64,
    pub nBidVolume6: u64,
    pub nBidVolume7: u64,
    pub nBidVolume8: u64,
    pub nBidVolume9: u64,
    pub nBidVolume10: u64,
    pub nMatchItems: u64,
    pub TotalVolume: u64,
    pub TotalTurnover: u64,
    pub TotalBidVolume: u64,
    pub TotalAskVolume: u64,
    pub WeightedAvgBidPrice: u64,
    pub WeightedAvgAskPrice: u64,
    pub IOPV: u64,
    pub YieldToMaturity: u64,
    pub HighLimited: u64, // tick数据中的涨停价比普通值少了一位，需要特殊处理
    pub LowLimited: u64,  //tick数据中的跌停价比普通值少了一位，需要特殊处理
    #[serde(skip_deserializing)]
    #[serde(default = "default_dt")]
    pub dt: DateTime<FixedOffset>,
}
