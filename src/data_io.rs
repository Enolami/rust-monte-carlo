use anyhow::Result;
use chrono::NaiveDate;
use serde::{Deserialize};
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Debug, Deserialize, Clone)]
pub struct StockRecord {
    #[serde(rename = "<Ticker>")]
    pub ticker: String,
    #[serde(rename = "<DTYYYYMMDD>", deserialize_with = "deserialize_date")]
    pub date: chrono::NaiveDate,
    #[serde(rename = "<Open>")]
    pub open: f64,
    #[serde(rename = "<High>")]
    pub high: f64,
    #[serde(rename = "<Low>")]
    pub low: f64,
    #[serde(rename = "<Close>")]
    pub close: f64,
    #[serde(rename = "<Volume>")]
    pub volume: i64,

}

fn deserialize_date<'de, D>(deserializer : D) -> Result<NaiveDate, D::Error>
where 
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    NaiveDate::parse_from_str(&s, "%Y%m%d").map_err(serde::de::Error::custom)
}

pub fn load_all_records(path: PathBuf) -> Result<(Vec<StockRecord>, Vec<String>)> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut records = Vec::new();
    //use BtreeMap instead of Hashmap for better:
    //Sorted data, lower memory usage, O(logn) as avg,
    let mut tickers = BTreeMap::new();

    for result in reader.deserialize() {
        let record: StockRecord = result?;
        tickers.insert(record.ticker.clone(), true);
        records.push(record);
    }

    records.sort_by_key(|r| r.date);

    let ticker_list = tickers.keys().cloned().collect();
    Ok((records, ticker_list))
}

pub fn get_ticker_info(all_data: &[StockRecord], ticker: &str) -> (String, Vec<f64>) {
    let ticker_data: Vec<&StockRecord> = all_data.iter().filter(|r| r.ticker == ticker).collect();
    
    if ticker_data.is_empty() {
        return ("No data for this ticker.".to_string(), Vec::new());
    }

    let start_date = ticker_data.first().unwrap().date;
    let end_date = ticker_data.last().unwrap().date;
    let count = ticker_data.len();
    let last_price = ticker_data.last().unwrap().close;

    let mut log_returns = Vec::new();
    for window in ticker_data.windows(2) {
        let s1 = window[0].close;
        let s2 = window[1].close;
        if s1 > 0.0 && s2 > 0.0 {
            log_returns.push((s2 / s1).ln());
        }
    }

    let info = format!(
        "Ticker: {}\nDate Range: {} to {}\nRecord Count: {}\nLast Close Price: {:.2}\nLog Returns Computed: {}",
        ticker, start_date, end_date, count, last_price, log_returns.len()
    );

    (info, log_returns)
}