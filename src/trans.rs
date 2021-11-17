use serde::Deserialize;


#[derive(Debug, Deserialize)]
pub struct transaction {
    Tkr :String,
    Time :u64,
    Index :u64,
    Price :u64,
    Volume :u64,
    Turnover :u64,
    BSFlag :char,
    OrderKind :u64,
    FunctionCode :u64,
    AskOrder :u64,
    BidOrder :u64,
}