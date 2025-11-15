use anyhow::{Ok, Result, anyhow};
use rand::{rngs::StdRng, Rng, SeedableRng};
use rand_distr::{Distribution, Normal, num_traits::{Float, SaturatingMul}};
use rayon::prelude::*;
use slint::SharedString;
use statrs::statistics::{Data, Distribution as StatDist, Median, OrderStatistics};
use std::{fmt, path};

use crate::SimParams;

#[derive(Debug, Clone)]
pub struct SimStats {
    pub model: String,
    pub paths: usize,
    pub horizon: usize,
    pub mean: f64,
    pub std_dev: f64,
    pub median: f64,
    pub p5: f64,
    pub p25: f64,
    pub p75: f64,
    pub p95: f64,
    pub var95: f64,
}

pub fn run_simulation (params: SimParams, hist_log_returns: Vec<f64>,) -> Result<(SimStats, (Vec<u8>, u32, u32), (Vec<u8>, u32, u32))> {
    let init_price = params.initial_price as f64;
    let mu = params.mu as f64;
    let sigma = params.sigma as f64;
    let horizon = params.horizon as usize;
    let num_paths = params.num_paths as usize;
    let dt = params.dt as f64;
    let model_name = match params.model_type.as_str() {
        "GBM" => "GBM",
        "Bootstrap" => "Bootstrap",
        _ => "",
    };

    let paths: Vec<Vec<f64>> = (0..num_paths).into_par_iter().map(|i| {
        // For antithetic variates: pair paths (i, i+1) use the same seed
        // Path i uses normal Z, path i+1 uses -Z
        let base_seed = if params.use_antithetic {
            (params.seed as u64).wrapping_add((i / 2) as u64)
        } else {
            (params.seed as u64).wrapping_add(i as u64)
        };
        let mut rng = StdRng::seed_from_u64(base_seed);
        let is_antithetic = params.use_antithetic && (i % 2 == 1);

        match params.model_type.as_str() {
            "GBM" => generate_gbm_path(init_price,mu,sigma,horizon,dt,is_antithetic,&mut rng),
            "Bootstrap" => generate_bootstrap_path(init_price,horizon,&hist_log_returns, &mut rng),
            _ => Vec::new()
        }
    }).collect();

    let mut terminal_prices: Vec<f64> = paths.iter().map(|path| *path.last().unwrap()).collect();
    let stats = calculate_statistics(&mut terminal_prices, init_price, model_name,num_paths, horizon)?;

    let paths_png = crate::plotting::plot_price_paths(&paths)?;
    let hist_png = crate::plotting::plot_histogram(&terminal_prices, 100)?;

    Ok((stats, paths_png, hist_png))
}

fn generate_gbm_path(init_price: f64, mu: f64, sigma: f64, steps: usize, dt: f64, is_antithetic: bool, rng: &mut StdRng,) -> Vec<f64> {
    let mut path = Vec::with_capacity(steps+1);
    path.push(init_price);
    let mut current_price = init_price;

    let drift = (mu - 0.5 * sigma.powi(2)) * dt;
    let diffusion = sigma * dt.sqrt();
    let normal = Normal::new(0.0, 1.0).unwrap();

    for _ in 0..steps {
        let mut z = normal.sample(rng);
        if is_antithetic {
            z = -z;
        }

        let next_price = current_price * (drift + diffusion * z).exp();
        path.push(next_price);
        current_price = next_price;
    }
    path
}

fn generate_bootstrap_path(init_price: f64, steps: usize, log_returns: &[f64], rng: &mut StdRng) -> Vec<f64> {
    if log_returns.is_empty() {
        return vec![init_price; steps+1];
    }

    let mut path = Vec::with_capacity(steps+1);
    path.push(init_price);
    let mut current_price = init_price;

    for _ in 0..steps {
        let idx = rng.random_range(0..log_returns.len());
        let log_return = log_returns[idx];
        let next_price = current_price * log_return.exp();
        path.push(next_price);
        current_price = next_price;
    }
    path
}

pub fn estimate_paramaters(log_returns: &[f64]) -> Result<(f64, f64)> {
    if log_returns.len() < 2 {
        return Err(anyhow!("Not enough data to estimate parameters. Need at least 2 log returns."));
    }
    let data = Data::new(log_returns.to_vec());
    let mu = data.mean().unwrap_or(0.0);
    let sigma = data.std_dev().unwrap_or(0.0);

    Ok((mu, sigma))
}

fn calculate_statistics(terminal_prices: &mut [f64], initial_price: f64, model: &str, paths: usize, horizon: usize) -> Result<SimStats> {
    if terminal_prices.is_empty() {
        return Err(anyhow!("No terminal prices to analyze"));
    }

    let mut data = Data::new(terminal_prices.to_vec());
    let mean = data.mean().unwrap_or(0.0);
    let std_dev = data.std_dev().unwrap_or(0.0);
    let median = data.median();

    let mut ordered_data = Data::new(terminal_prices.to_vec());
    let p5 = ordered_data.percentile(5);
    let p25 = ordered_data.percentile(25);
    let p75 = ordered_data.percentile(75);
    let p95 = ordered_data.percentile(95);

    // Calculate VaR95 from returns: negative of 5th percentile of total returns
    let returns: Vec<f64> = terminal_prices.iter().map(|&tp| (tp - initial_price) / initial_price).collect();
    let mut returns_data = Data::new(returns);
    let p5_return = returns_data.percentile(5);
    let var95 = -p5_return;

    Ok(SimStats { model: model.to_string(), paths, horizon, mean, std_dev, median, p5, p25, p75, p95, var95 })

}