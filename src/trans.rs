use serde::Deserialize;


#[derive(Debug, Deserialize)]
pub struct transaction {
    pub Tkr :String,
    pub Time :u64,
    pub Index :u64,
    pub Price :u64,
    pub Volume :u64,
    pub Turnover :u64,
    pub BSFlag :char,
    pub OrderKind :u64,
    pub FunctionCode :u64,
    pub AskOrder :u64,
    pub BidOrder :u64,
}
