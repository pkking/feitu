use serde::Deserialize;


#[derive(Debug, Deserialize, Clone)]
pub struct Tick {
    pub chWindCode :String,
    pub nTime :u64,
    pub Status :u64,
    pub PreClose :u64,
    pub Open :u64,
    pub High :u64,
    pub Low :u64,
    pub nPrice :u64,
    pub nAskPrice1 :u64,
    pub nAskPrice2 :u64,
    pub nAskPrice3 :u64,
    pub nAskPrice4 :u64,
    pub nAskPrice5 :u64,
    pub nAskPrice6 :u64,
    pub nAskPrice7 :u64,
    pub nAskPrice8 :u64,
    pub nAskPrice9 :u64,
    pub nAskPrice10 :u64,
    pub nAskVolume1 :u64,
    pub nAskVolume2 :u64,
    pub nAskVolume3 :u64,
    pub nAskVolume4 :u64,
    pub nAskVolume5 :u64,
    pub nAskVolume6 :u64,
    pub nAskVolume7 :u64,
    pub nAskVolume8 :u64,
    pub nAskVolume9 :u64,
    pub nAskVolume10 :u64,
    pub nBidPrice1 :u64,
    pub nBidPrice2 :u64,
    pub nBidPrice3 :u64,
    pub nBidPrice4 :u64,
    pub nBidPrice5 :u64,
    pub nBidPrice6 :u64,
    pub nBidPrice7 :u64,
    pub nBidPrice8 :u64,
    pub nBidPrice9 :u64,
    pub nBidPrice10 :u64,
    pub nBidVolume1 :u64,
    pub nBidVolume2 :u64,
    pub nBidVolume3 :u64,
    pub nBidVolume4 :u64,
    pub nBidVolume5 :u64,
    pub nBidVolume6 :u64,
    pub nBidVolume7 :u64,
    pub nBidVolume8 :u64,
    pub nBidVolume9 :u64,
    pub nBidVolume10 :u64,
    pub nMatchItems :u64,
    pub TotalVolume :u64,
    pub TotalTurnover :u64,
    pub TotalBidVolume :u64,
    pub TotalAskVolume :u64,
    pub WeightedAvgBidPrice :u64,
    pub WeightedAvgAskPrice :u64,
    pub IOPV :u64,
    pub YieldToMaturity :u64,
    pub HighLimited :u64, // tick数据中的涨停价比普通值少了一位，需要特殊处理
    pub LowLimited :u64,  //tick数据中的跌停价比普通值少了一位，需要特殊处理
}
