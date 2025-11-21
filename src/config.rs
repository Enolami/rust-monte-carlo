use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::core_sim::ModelParams;

/// Configuration for a single simulation
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SimConfig {
    // Simulation parameters
    pub initial_price: f64,
    pub horizon: usize,
    pub num_paths: usize,
    pub seed: u64,
    pub use_antithetic: bool,
    pub dt: f64,
    
    // Model configuration
    pub model_type: String,  // "GBM", "Bootstrap", "MeanReversion", etc.
    
    // Model-specific parameters (stored as separate fields for JSON compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gbm_params: Option<GBMParams>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mean_reversion_params: Option<MeanReversionParams>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jump_diffusion_params: Option<JumpDiffusionParams>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub garch_params: Option<GARCHParams>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GBMParams {
    pub mu: f64,
    pub sigma: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MeanReversionParams {
    pub theta: f64,
    pub mu_long_term: f64,
    pub sigma: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JumpDiffusionParams {
    pub mu: f64,
    pub sigma: f64,
    pub lambda: f64,
    pub mu_j: f64,
    pub sigma_j: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GARCHParams {
    pub omega: f64,
    pub alpha: f64,
    pub beta: f64,
}

impl SimConfig {
    /// Convert to ModelParams enum
    pub fn to_model_params(&self) -> Result<ModelParams> {
        match self.model_type.as_str() {
            "GBM" => {
                if let Some(ref params) = self.gbm_params {
                    Ok(ModelParams::GBM {
                        mu: params.mu,
                        sigma: params.sigma,
                    })
                } else {
                    Err(anyhow::anyhow!("GBM parameters not found"))
                }
            }
            "Bootstrap" => Ok(ModelParams::Bootstrap {}),
            "MeanReversion" => {
                if let Some(ref params) = self.mean_reversion_params {
                    Ok(ModelParams::MeanReversion {
                        theta: params.theta,
                        mu_long_term: params.mu_long_term,
                        sigma: params.sigma,
                    })
                } else {
                    Err(anyhow::anyhow!("Mean Reversion parameters not found"))
                }
            }
            "JumpDiffusion" => {
                if let Some(ref params) = self.jump_diffusion_params {
                    Ok(ModelParams::JumpDiffusion {
                        mu: params.mu,
                        sigma: params.sigma,
                        lambda: params.lambda,
                        mu_j: params.mu_j,
                        sigma_j: params.sigma_j,
                    })
                } else {
                    Err(anyhow::anyhow!("Jump Diffusion parameters not found"))
                }
            }
            "GARCH" => {
                if let Some(ref params) = self.garch_params {
                    Ok(ModelParams::GARCH {
                        omega: params.omega,
                        alpha: params.alpha,
                        beta: params.beta,
                    })
                } else {
                    Err(anyhow::anyhow!("GARCH parameters not found"))
                }
            }
            _ => Err(anyhow::anyhow!("Unknown model type: {}", self.model_type)),
        }
    }
}

/// Save configuration to JSON file
pub fn save_config(config: &SimConfig, path: &Path) -> Result<()> {
    let json = serde_json::to_string_pretty(config)?;
    fs::write(path, json)?;
    Ok(())
}

/// Load configuration from JSON file
pub fn load_config(path: &Path) -> Result<SimConfig> {
    let json = fs::read_to_string(path)?;
    let config: SimConfig = serde_json::from_str(&json)?;
    Ok(config)
}

/// Validate configuration
pub fn validate_config(config: &SimConfig) -> Result<()> {
    // Basic validations
    if config.initial_price <= 0.0 {
        return Err(anyhow::anyhow!("Initial price must be positive"));
    }
    
    if config.horizon == 0 {
        return Err(anyhow::anyhow!("Horizon must be greater than 0"));
    }
    
    if config.num_paths == 0 {
        return Err(anyhow::anyhow!("Number of paths must be greater than 0"));
    }
    
    if config.dt <= 0.0 {
        return Err(anyhow::anyhow!("dt must be positive"));
    }
    
    // Model-specific validations
    match config.model_type.as_str() {
        "GBM" => {
            if let Some(ref params) = config.gbm_params {
                if params.sigma < 0.0 {
                    return Err(anyhow::anyhow!("GBM sigma must be non-negative"));
                }
            } else {
                return Err(anyhow::anyhow!("GBM parameters missing"));
            }
        }
        "MeanReversion" => {
            if let Some(ref params) = config.mean_reversion_params {
                if params.theta <= 0.0 {
                    return Err(anyhow::anyhow!("Mean Reversion theta must be positive"));
                }
                if params.sigma < 0.0 {
                    return Err(anyhow::anyhow!("Mean Reversion sigma must be non-negative"));
                }
            } else {
                return Err(anyhow::anyhow!("Mean Reversion parameters missing"));
            }
        }
        "JumpDiffusion" => {
            if let Some(ref params) = config.jump_diffusion_params {
                if params.lambda < 0.0 {
                    return Err(anyhow::anyhow!("Jump Diffusion lambda must be non-negative"));
                }
                if params.sigma < 0.0 {
                    return Err(anyhow::anyhow!("Jump Diffusion sigma must be non-negative"));
                }
                if params.sigma_j < 0.0 {
                    return Err(anyhow::anyhow!("Jump Diffusion sigma_j must be non-negative"));
                }
            } else {
                return Err(anyhow::anyhow!("Jump Diffusion parameters missing"));
            }
        }
        "GARCH" => {
            if let Some(ref params) = config.garch_params {
                if params.omega <= 0.0 {
                    return Err(anyhow::anyhow!("GARCH omega must be positive"));
                }
                if params.alpha < 0.0 {
                    return Err(anyhow::anyhow!("GARCH alpha must be non-negative"));
                }
                if params.beta < 0.0 {
                    return Err(anyhow::anyhow!("GARCH beta must be non-negative"));
                }
                if params.alpha + params.beta >= 1.0 {
                    return Err(anyhow::anyhow!("GARCH stationarity condition failed: alpha + beta must be < 1"));
                }
            } else {
                return Err(anyhow::anyhow!("GARCH parameters missing"));
            }
        }
        "Bootstrap" => {
            // No additional validation needed
        }
        _ => {
            return Err(anyhow::anyhow!("Unknown model type: {}", config.model_type));
        }
    }
    
    Ok(())
}