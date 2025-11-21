use anyhow::Result;
use chrono::NaiveDate;
use serde::{Deserialize};
use std::{collections::{ HashMap}, path::PathBuf};

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

pub fn load_all_records(path: PathBuf) -> Result<(HashMap<String, Vec<StockRecord>>, Vec<String>)> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut data_map: HashMap<String, Vec<StockRecord>> = HashMap::new();

    for result in reader.deserialize() {
        let record: StockRecord = result?;
        data_map.entry(record.ticker.clone()).or_insert_with(Vec::new).push(record);
    }

    let mut tickers = Vec::new();
    for (ticker, records) in data_map.iter_mut() {
        records.sort_by_key(|r| r.date);
        tickers.push(ticker.clone());
    }

    tickers.sort();

    Ok((data_map,tickers))
}

pub fn get_ticker_info(data_map: &HashMap<String, Vec<StockRecord>>, ticker: &str) -> (String, Vec<f64>) { 
    let ticker_data = match data_map.get(ticker) {
        Some(d) => d,
        None => return ("No data for this ticker.".to_string(), Vec::new()),
    };
    
    let start_date = ticker_data.first().unwrap().date;
    let end_date = ticker_data.last().unwrap().date;
    let count = ticker_data.len();
    let last_price = ticker_data.last().unwrap().close;

    let mut log_returns = Vec::new();
    for window in ticker_data.windows(2) {
        let s1 = window[0].close;
        let s2 = window[1].close;
        if s1 > 0.0 && s2 > 0.0 {
            log_returns.push((s2/s1).ln());
        }
    }

    let info = format!(
        "Ticker: {}\nDate Range: {} to {}\nRecord Count: {}\nLast Close Price: {:.2}\nLog Returns Computed: {}",
        ticker, start_date, end_date, count, last_price, log_returns.len()
    );

    (info, log_returns)
}