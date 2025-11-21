use anyhow::{Ok, Result, anyhow};
use rand::{rngs::StdRng, Rng, SeedableRng};
use rand_distr::{Distribution, Normal};
use rayon::prelude::*;
use statrs::statistics::{Data, Distribution as StatDist, Median, OrderStatistics};

use crate::SimParams;


// Model-specific parameters enum
#[derive(Debug, Clone)]
pub enum ModelParams {
    GBM {
        mu: f64,
        sigma: f64,
    },
    Bootstrap {
        
    },
    MeanReversion {
        theta: f64,       
        mu_long_term: f64, 
        sigma: f64,       
    },
    JumpDiffusion {
        mu: f64,         
        sigma: f64,       
        lambda: f64,      
        mu_j: f64,        
        sigma_j: f64,     
    },
    GARCH {
        omega: f64,      
        alpha: f64,       
        beta: f64,       
    },
}

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
            "GBM" => generate_gbm_path(init_price, mu, sigma, horizon, dt, params.use_antithetic && (i%2==1), &mut rng),
            "Bootstrap" => generate_bootstrap_path(init_price, horizon, &hist_log_returns, &mut rng),
            "MeanReversion" => {
                let theta = params.theta as f64;
                let mu_long_term = params.mu_long_term as f64;
                let sigma = params.sigma as f64;
                generate_mean_reversion_path(init_price, theta, mu_long_term, sigma, horizon, dt, params.use_antithetic && (i%2==1), &mut rng)
            }
            "JumpDiffusion" => {
                let mu = params.mu as f64;
                let sigma = params.sigma as f64;
                let lambda = params.lambda as f64;
                let mu_j = params.mu_j as f64;
                let sigma_j = params.sigma_j as f64;
                generate_jump_diffusion_path(init_price, mu, sigma, lambda, mu_j, sigma_j, horizon, dt, params.use_antithetic && (i%2==1), &mut rng)
            }
            "GARCH" => {
                let omega = params.omega as f64;
                let alpha = params.alpha as f64;
                let beta = params.beta as f64;
                generate_garch_path(init_price, omega, alpha, beta, horizon, dt, params.use_antithetic && (i%2==1), &mut rng)
            }
    _ => Vec::new()
}
    }).collect();

    let mut terminal_prices: Vec<f64> = paths.iter().map(|path| *path.last().unwrap()).collect();
    let stats = calculate_statistics(&mut terminal_prices, model_name,num_paths, horizon, init_price)?;

    let mu_long_term_value = if params.model_type == "MeanReversion" {
        Some(params.mu_long_term as f64)
    } else {
        None
    };

    let paths_png = crate::plotting::plot_price_paths(
        &paths,
        &params.model_type,
        mu_long_term_value,
    )?;
    let hist_png = crate::plotting::plot_histogram(&terminal_prices, 100)?;

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

// Helper function to create ModelParams from Slint's SimParams
pub fn create_model_params(model_type: &str, mu: f64, sigma: f64) -> ModelParams {
    match model_type {
        "GBM" => ModelParams::GBM { mu, sigma },
        "Bootstrap" => ModelParams::Bootstrap {},
        "MeanReversion" => ModelParams::MeanReversion {
            theta: 0.1,           // Default value
            mu_long_term: 100.0,  // Default value
            sigma,
        },
        "JumpDiffusion" => ModelParams::JumpDiffusion {
            mu,
            sigma,
            lambda: 2.0,      // Default: 2 jumps per year
            mu_j: -0.02,      // Default: small negative jump
            sigma_j: 0.05,    // Default: 5% jump volatility
        },
        "GARCH" => ModelParams::GARCH {
            omega: 0.00001,   // Default: small constant
            alpha: 0.1,       // Default: ARCH coefficient
            beta: 0.85,       // Default: GARCH coefficient
        },
        _ => ModelParams::GBM { mu, sigma }, // Default fallback
    }
}


fn generate_mean_reversion_path(
    init_price: f64,
    theta: f64,        // Speed of reversion
    mu_long_term: f64, // Long-term mean price
    sigma: f64,        // Volatility
    steps: usize,
    dt: f64,
    is_antithetic: bool,
    rng: &mut StdRng,
) -> Vec<f64> {
    let mut path = Vec::with_capacity(steps + 1);
    path.push(init_price);
    let mut current_price = init_price;

    let diffusion = sigma * dt.sqrt();
    let normal = Normal::new(0.0, 1.0).unwrap();

    for _ in 0..steps {
        let mut z = normal.sample(rng);
        if is_antithetic {
            z = -z;
        }

        // Ornstein-Uhlenbeck: dS = θ(μ - S)dt + σdW
        let drift = theta * (mu_long_term - current_price) * dt;
        let shock = diffusion * z;
        
        let next_price = current_price + drift + shock;
        
        // Optional: prevent negative prices (uncommon for mean reversion but safe)
        let next_price = next_price.max(0.01);
        
        path.push(next_price);
        current_price = next_price;
    }
    
    path
}


fn generate_jump_diffusion_path(
    init_price: f64,
    mu: f64,           // Drift
    sigma: f64,        // Diffusion volatility
    lambda: f64,       // Jump intensity (average jumps per unit time)
    mu_j: f64,         // Mean of jump size (in log space)
    sigma_j: f64,      // Std dev of jump size (in log space)
    steps: usize,
    dt: f64,
    is_antithetic: bool,
    rng: &mut StdRng,
) -> Vec<f64> {
    let mut path = Vec::with_capacity(steps + 1);
    path.push(init_price);
    let mut current_price = init_price;

    // GBM components
    let drift = (mu - 0.5 * sigma.powi(2)) * dt;
    let diffusion = sigma * dt.sqrt();
    let normal = Normal::new(0.0, 1.0).unwrap();

    // Jump components
    use rand_distr::Poisson;
    let poisson = Poisson::new(lambda * dt).unwrap();
    let jump_normal = Normal::new(mu_j, sigma_j).unwrap();

    for _ in 0..steps {
        // Diffusion part (GBM)
        let mut z = normal.sample(rng);
        if is_antithetic {
            z = -z;
        }
        
        let gbm_return = drift + diffusion * z;

        // Jump part
        let num_jumps = poisson.sample(rng) as usize;
        let mut jump_effect = 0.0;
        
        for _ in 0..num_jumps {
            // Jump size in log space
            let jump_size = jump_normal.sample(rng);
            jump_effect += jump_size;
        }

        // Combine: S_{t+1} = S_t * exp(gbm_return + jump_effect)
        let total_return = gbm_return + jump_effect;
        let next_price = current_price * total_return.exp();
        
        path.push(next_price);
        current_price = next_price;
    }
    
    path
}


fn generate_garch_path(
    init_price: f64,
    omega: f64,        // Constant term
    alpha: f64,        // ARCH coefficient
    beta: f64,         // GARCH coefficient
    steps: usize,
    dt: f64,
    is_antithetic: bool,
    rng: &mut StdRng,
) -> Vec<f64> {
    let mut path = Vec::with_capacity(steps + 1);
    path.push(init_price);
    let mut current_price = init_price;

    // Initialize variance (unconditional variance if stationary)
    let mut variance = if alpha + beta < 1.0 {
        omega / (1.0 - alpha - beta)
    } else {
        omega / 0.1  // Fallback if not stationary
    };
    
    let mut prev_return: f64 = 0.0;
    let normal = Normal::new(0.0, 1.0).unwrap();

    for _ in 0..steps {
        // Generate random shock
        let mut epsilon = normal.sample(rng);
        if is_antithetic {
            epsilon = -epsilon;
        }

        // Current return: r_t = σ_t * ε_t
        let volatility = variance.sqrt();
        let return_t = volatility * epsilon * dt.sqrt();

        // Update price: S_t = S_{t-1} * exp(r_t)
        let next_price = current_price * return_t.exp();
        
        path.push(next_price);

        // Update variance for next step: σ²_{t+1} = ω + α·r²_t + β·σ²_t
        variance = omega + alpha * prev_return.powi(2) + beta * variance;
        
        // Prevent variance from becoming too small or negative
        variance = variance.max(1e-6);
        
        prev_return = return_t;
        current_price = next_price;
    }
    
    path
}
