use anyhow::{Ok, Result, anyhow};
use rand::{rngs::StdRng, Rng, SeedableRng};
use rand_distr::{Distribution, Normal};
use rayon::prelude::*;
use statrs::statistics::{Data, Distribution as StatDist, Median, OrderStatistics};
use nalgebra::DVector;

use crate::{SimParams, porfolio::PortfolioConfig};

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
        let seed = (params.seed as u64).wrapping_add(i as u64);
        let mut rng = StdRng::seed_from_u64(seed);

        match params.model_type.as_str() {
            "GBM" => generate_gbm_path(init_price,mu,sigma,horizon,dt,params.use_antithetic && (i%2==1),&mut rng),
            "Bootstrap" => generate_bootstrap_path(init_price,horizon,&hist_log_returns, &mut rng),
            _ => Vec::new()
        }
    }).collect();

    let mut terminal_prices: Vec<f64> = paths.iter().map(|path| *path.last().unwrap()).collect();
    let stats = calculate_statistics(&mut terminal_prices, model_name,num_paths, horizon, init_price)?;

    let paths_png = crate::plotting::plot_price_paths(&paths)?;
    let hist_png = crate::plotting::plot_histogram(&terminal_prices, 100)?;

    Ok((stats, paths_png, hist_png))
}

pub fn run_portfolio_simulation(params: SimParams, config: PortfolioConfig) -> Result<(SimStats, (Vec<u8>, u32, u32), (Vec<u8>, u32,u32))> {
    let horizon = params.horizon as usize;
    let num_paths = params.num_paths as usize;
    let dt = params.dt as f64;
    let num_assets = config.assets.len();

    let paths: Vec<Vec<f64>> = (0..num_paths).into_par_iter().map(|i| {
        let seed = (params.seed as u64).wrapping_add(i as u64);
        let mut rng = StdRng::seed_from_u64(seed);
        let normal = Normal::new(0.0, 1.0).unwrap();

        let mut current_asset_prices: Vec<f64> = config.assets.iter().map(|a| a.last_price).collect();

        let mut portfolio_path = Vec::with_capacity(horizon+1);
        portfolio_path.push(config.init_value);

        for _ in 0..horizon {
            let z_independent: Vec<f64> = (0..num_assets).map(|_| normal.sample(&mut rng)).collect();
            let z_vec = DVector::from_vec(z_independent);

            let z_correlated = &config.cholesky_l * z_vec;

            let mut current_portfolio_val = 0.0;

            for j in 0..num_assets {
                let asset = &config.assets[j];
                let drift = (asset.mu - 0.5 * asset.sigma.powi(2)) * dt;
                let diffusion = asset.sigma * dt.sqrt();
                let shock = diffusion * z_correlated[j];

                current_asset_prices[j] *= (drift + shock).exp();

                current_portfolio_val += current_asset_prices[j] * asset.shares;
            }
            portfolio_path.push(current_portfolio_val);
        }
        portfolio_path
    }).collect();

    let mut terminal_values: Vec<f64> = paths.iter().map(|p| *p.last().unwrap()).collect();
    let stats = calculate_statistics(&mut terminal_values, "Portfolio GBM", num_paths, horizon, config.init_value)?;

    let paths_png = crate::plotting::plot_price_paths(&paths)?;
    let hist_png = crate::plotting::plot_histogram(&terminal_values, 100)?;

    Ok((stats, paths_png, hist_png))
}

fn generate_gbm_path(init_price: f64, mu: f64, sigma: f64, steps: usize, dt: f64, is_antithetic: bool, rng: &mut StdRng,) -> Vec<f64> {
    //plus 1 for init_price
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
        return Err(anyhow!("Not enough data to estimate parameters. Neet at least 2 log returns."));
    }
    let data = Data::new(log_returns.to_vec());
    let mu = data.mean().unwrap_or(0.0);
    let sigma = data.std_dev().unwrap_or(0.0);

    Ok((mu, sigma))
}

fn calculate_statistics(terminal_prices: &mut [f64], model: &str, paths: usize, horizon: usize, init_price: f64) -> Result<SimStats> {
    if terminal_prices.is_empty() {
        return Err(anyhow!("No terminal prcies to analyze"));
    }

    let data = Data::new(terminal_prices.to_vec());
    let mean = data.mean().unwrap_or(0.0);
    let std_dev = data.std_dev().unwrap_or(0.0);
    let median = data.median();

    let mut ordered_data = Data::new(terminal_prices.to_vec());
    let p5 = ordered_data.percentile(5);
    let p25 = ordered_data.percentile(25);
    let p75 = ordered_data.percentile(75);
    let p95 = ordered_data.percentile(95);

    let returns: Vec<f64> = terminal_prices.iter()
        .map(|&price| (price - init_price) / init_price)
        .collect();
    
    let mut returns_data = Data::new(returns);
    let p5_return = returns_data.percentile(5);
    let var95 = -p5_return;

    Ok(SimStats { model: model.to_string(), paths, horizon, mean, std_dev, median, p5, p25, p75, p95, var95 })

}