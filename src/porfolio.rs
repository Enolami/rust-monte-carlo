use anyhow::{Result, anyhow};
use core::f64;
use std::{collections::HashMap, error, usize};
use nalgebra:: {Cholesky, DMatrix, DVector};
use crate::data_io::StockRecord;

#[derive(Debug, Clone)]
pub struct PortfolioAsset {
    pub ticker: String,
    pub shares: f64,
    pub mu: f64,
    pub sigma: f64,
    pub last_price: f64,
}

#[derive(Debug, Clone)]
pub struct PortfolioConfig {
    pub assets: Vec<PortfolioAsset>,
    pub cholesky_l: DMatrix<f64>, //for correlation
    pub init_value: f64,
}

pub fn build_portfolio_config(ticker_weights: &[(String, f64)], total_capital: f64, data_map: &HashMap<String, Vec<StockRecord>>) -> Result<PortfolioConfig> {
    let mut assets = Vec::new();
    let mut aligned_log_returns: Vec<Vec<f64>> = Vec::new();

    //find minimum history length for correlation
    //if single, take the shortest ticker history
    let mut min_len = usize::MAX;

    for (ticker, _) in ticker_weights {
        if let Some(records) = data_map.get(ticker) {
            //at least 30 records for correlation
            if records.len() < 30 {
                return Err(anyhow!("Ticker {} has insufficient data (<30 records)", ticker));
            }
            min_len = min_len.min(records.len() - 1); //-1 cuz returns are n-1
        } else {
            return Err(anyhow!("Ticker {} not found in loaded data", ticker));
        }
    }

    if min_len == usize::MAX {
        return Err(anyhow!("No valid data found for selected tickers"));
    }

    //asset list and return matrix
    for (ticker, weight_pct) in ticker_weights {
        let records = data_map.get(ticker).unwrap();

        let mut log_returns = Vec::new();
        for window in records.windows(2) {
            let r = (window[1].close/window[0].close).ln();
            log_returns.push(r);
        }

        let start_idx = log_returns.len() - min_len;
        let aligned_returns = log_returns[start_idx..].to_vec();

        let count = aligned_returns.len() as f64;
        let mean = aligned_returns.iter().sum::<f64>()/count;
        let variance = aligned_returns.iter().map(|x| (x-mean).powi(2)).sum::<f64>() / (count - 1.0);

        let last_price = records.last().unwrap().close;

        let alloc_capital = total_capital * (weight_pct/100.0);
        let shares = alloc_capital / last_price;

        assets.push(PortfolioAsset {
            ticker: ticker.clone(),
            shares,
            mu: mean,
            sigma: variance.sqrt(),
            last_price,
        });

        aligned_log_returns.push(aligned_returns);
    }

    let n_assests = assets.len();
    let mut correlation_matrix = DMatrix::identity(n_assests, n_assests);

    for i in 0..n_assests {
        for j in (i+1)..n_assests {
            let corr = calculate_pair_correlation(&aligned_log_returns[i], &aligned_log_returns[j]);
            correlation_matrix[(i,j)] = corr;
            correlation_matrix[(j,i)] = corr;
        }
    }

    let cholesky_l = match correlation_matrix.cholesky() {
        Some(cholesky) => cholesky.l(),
        None => return Err(anyhow!("Correlation matrix is not positive definite. Check for duplicate assets or insufficient data.")),
    };

    Ok(PortfolioConfig { assets , cholesky_l, init_value: total_capital })
}

fn calculate_pair_correlation(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len() as f64;
    let mean_x = x.iter().sum::<f64>() / n;
    let mean_y = y.iter().sum::<f64>() / n;

    let mut numerator = 0.0;
    let mut denom_x = 0.0;
    let mut denom_y = 0.0;

    for i in 0..x.len() {
        let dx = x[i] - mean_x;
        let dy = y[i] - mean_y;
        numerator += dx * dy;
        denom_x += dx * dx;
        denom_y += dy * dy;
    }

    if denom_x == 0.0 || denom_y == 0.0 {return 0.0;}
    numerator / (denom_x.sqrt() * denom_y.sqrt())
}