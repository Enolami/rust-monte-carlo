# SPRINT 1: MODELS + SAVE/LOAD SETUP

**Duration:** 1-2 weeks
**Goal:** Extend simulation with 3 new models (Mean Reversion, Jump Diffusion, GARCH) and implement save/load configuration functionality

---

## 🎯 SPRINT OBJECTIVES

- [ ] Implement 3 new simulation models with proper mathematical formulas
- [ ] Create dynamic UI for model-specific parameters
- [ ] Implement save/load configuration system
- [ ] Add comprehensive unit tests for all models
- [ ] Update documentation and code structure

---

## 📋 DETAILED TASK BREAKDOWN

### **PHASE 1: CODE STRUCTURE REFACTORING** (Days 1-2)

#### Task 1.1: Refactor Model Architecture
**Priority:** HIGH
**Effort:** 4-6 hours

- [ ] Create `ModelParams` enum in `core_sim.rs`
  ```rust
  pub enum ModelParams {
      GBM { mu: f64, sigma: f64 },
      Bootstrap { returns: Vec<f64> },
      MeanReversion { theta: f64, mu_long_term: f64, sigma: f64 },
      JumpDiffusion { mu: f64, sigma: f64, lambda: f64, mu_j: f64, sigma_j: f64 },
      GARCH { omega: f64, alpha: f64, beta: f64 },
  }
  ```
- [ ] Update `SimParams` struct to use `ModelParams`
- [ ] Refactor existing GBM and Bootstrap to use new enum pattern
- [ ] Update `run_simulation()` function signature
- [ ] Test existing functionality still works

**Deliverable:** Refactored code with backward compatibility

---

#### Task 1.2: Create Configuration Module
**Priority:** HIGH
**Effort:** 3-4 hours

- [ ] Create new file `src/config.rs`
- [ ] Define `SimConfig` struct (serializable with serde)
  ```rust
  #[derive(Serialize, Deserialize, Clone, Debug)]
  pub struct SimConfig {
      pub initial_price: f64,
      pub horizon: usize,
      pub num_paths: usize,
      pub seed: u64,
      pub use_antithetic: bool,
      pub dt: f64,
      pub model_params: ModelParams,
  }
  ```
- [ ] Add to `main.rs`: `mod config;`
- [ ] Add serde derives to all relevant structs

**Deliverable:** Config module ready for save/load

---

### **PHASE 2: IMPLEMENT NEW MODELS** (Days 3-6)

#### Task 2.1: Implement Mean Reversion (Ornstein-Uhlenbeck)
**Priority:** HIGH
**Effort:** 6-8 hours

**Formula:**
`dS_t = θ(μ - S_t)dt + σdW_t`
Discrete: `S_{t+1} = S_t + θ(μ - S_t)Δt + σ√Δt·Z`

- [ ] Add function `generate_mean_reversion_path()` in `core_sim.rs`
  - Parameters: `theta` (speed of reversion), `mu_long_term`, `sigma`
  - Validate: `theta > 0`, `sigma > 0`
- [ ] Implement discretization logic
- [ ] Add antithetic variates support
- [ ] Write unit test: `test_mean_reversion_basic()`
- [ ] Write test: `test_mean_reversion_reproducibility()`
- [ ] Write test: `test_mean_reversion_bounds()` (prices stay around mu)

**Acceptance Criteria:**
- Mean of simulated prices should converge to `mu_long_term`
- Same seed produces same results
- Antithetic variates reduce variance

---

#### Task 2.2: Implement Jump Diffusion (Merton Model)
**Priority:** HIGH
**Effort:** 8-10 hours

**Formula:**
`dS_t = μS_tdt + σS_tdW_t + S_t(J-1)dN_t`
Where `J ~ LogNormal(μ_J, σ_J)`, `N_t ~ Poisson(λ)`

- [ ] Add function `generate_jump_diffusion_path()` in `core_sim.rs`
  - Parameters: `mu`, `sigma`, `lambda` (jump intensity), `mu_j` (jump mean), `sigma_j` (jump std)
  - Validate: `lambda >= 0`, `sigma >= 0`, `sigma_j >= 0`
- [ ] Implement GBM component
- [ ] Add Poisson jump process using `rand_distr::Poisson`
- [ ] Add log-normal jump sizes
- [ ] Add antithetic variates support (for diffusion part)
- [ ] Write unit test: `test_jump_diffusion_no_jumps()` (λ=0 → pure GBM)
- [ ] Write test: `test_jump_diffusion_reproducibility()`
- [ ] Write test: `test_jump_diffusion_jump_count()` (average jumps ≈ λ·T)

**Acceptance Criteria:**
- With λ=0, behaves like GBM
- Jump frequency matches Poisson(λ)
- Jump sizes are log-normally distributed

---

#### Task 2.3: Implement GARCH(1,1)
**Priority:** MEDIUM
**Effort:** 10-12 hours

**Formula:**
`r_t = σ_t·ε_t`, `ε_t ~ N(0,1)`
`σ_t² = ω + α·r_{t-1}² + β·σ_{t-1}²`

- [ ] Add function `generate_garch_path()` in `core_sim.rs`
  - Parameters: `omega`, `alpha`, `beta`, `initial_variance`
  - Validate: `omega > 0`, `alpha >= 0`, `beta >= 0`, `alpha + beta < 1`
- [ ] Implement variance updating mechanism
- [ ] Convert returns to prices: `S_t = S_{t-1} * exp(r_t)`
- [ ] Add antithetic variates support
- [ ] Write unit test: `test_garch_variance_persistence()`
- [ ] Write test: `test_garch_reproducibility()`
- [ ] Write test: `test_garch_stationarity()` (α+β < 1)

**Acceptance Criteria:**
- Variance shows clustering (high/low volatility periods)
- Unconditional variance = ω/(1-α-β)
- Same seed produces same results

---

### **PHASE 3: UPDATE GUI FOR DYNAMIC PARAMETERS** (Days 7-9)

#### Task 3.1: Create Dynamic Parameter Panel in Slint
**Priority:** HIGH
**Effort:** 8-10 hours

- [ ] Update `gui.slint`: Add model selection ComboBox
  ```slint
  ComboBox {
      model: ["GBM", "Bootstrap", "Mean Reversion", "Jump Diffusion", "GARCH"];
      selected => { root.model_type = self.current-value; }
  }
  ```

- [ ] Create conditional parameter panels for each model:

  **GBM Parameters:**
  - [ ] LineEdit for `mu` (drift)
  - [ ] LineEdit for `sigma` (volatility)

  **Mean Reversion Parameters:**
  - [ ] LineEdit for `theta` (speed of reversion)
  - [ ] LineEdit for `mu_long_term` (long-term mean)
  - [ ] LineEdit for `sigma` (volatility)

  **Jump Diffusion Parameters:**
  - [ ] LineEdit for `mu` (drift)
  - [ ] LineEdit for `sigma` (diffusion volatility)
  - [ ] LineEdit for `lambda` (jump intensity)
  - [ ] LineEdit for `mu_j` (jump mean)
  - [ ] LineEdit for `sigma_j` (jump std dev)

  **GARCH Parameters:**
  - [ ] LineEdit for `omega` (constant term)
  - [ ] LineEdit for `alpha` (ARCH coefficient)
  - [ ] LineEdit for `beta` (GARCH coefficient)
  - [ ] Display validation: α + β < 1

- [ ] Add property bindings to show/hide panels based on selected model
- [ ] Add input validation and error messages
- [ ] Add tooltips explaining each parameter

**Deliverable:** Dynamic UI that changes based on model selection

---

#### Task 3.2: Update Callback Logic
**Priority:** HIGH
**Effort:** 4-5 hours

- [ ] Update `on_run_simulation_pressed()` in `main.rs`
- [ ] Collect parameters based on selected model type
- [ ] Create appropriate `ModelParams` enum variant
- [ ] Pass to `run_simulation()`
- [ ] Handle validation errors gracefully (show error dialog)

**Deliverable:** GUI properly triggers simulations with model-specific params

---

### **PHASE 4: IMPLEMENT SAVE/LOAD FUNCTIONALITY** (Days 10-11)

#### Task 4.1: Implement Save Configuration
**Priority:** HIGH
**Effort:** 4-5 hours

- [ ] Add `serde = { version = "1.0", features = ["derive"] }` to Cargo.toml
- [ ] Add `serde_json = "1.0"` to Cargo.toml
- [ ] Implement `save_config()` function in `config.rs`
  ```rust
  pub fn save_config(config: &SimConfig, path: &Path) -> Result<()> {
      let json = serde_json::to_string_pretty(config)?;
      fs::write(path, json)?;
      Ok(())
  }
  ```
- [ ] Add "Save Setup" button in GUI
- [ ] Add callback `on_save_setup_pressed()` in `main.rs`
- [ ] Use `FileDialog` to select save location (.json)
- [ ] Collect all current parameters into `SimConfig`
- [ ] Call `save_config()`
- [ ] Show success/error notification

**Deliverable:** Users can save current setup to JSON file

---

#### Task 4.2: Implement Load Configuration
**Priority:** HIGH
**Effort:** 4-5 hours

- [ ] Implement `load_config()` function in `config.rs`
  ```rust
  pub fn load_config(path: &Path) -> Result<SimConfig> {
      let json = fs::read_to_string(path)?;
      let config: SimConfig = serde_json::from_str(&json)?;
      Ok(config)
  }
  ```
- [ ] Add "Load Setup" button in GUI
- [ ] Add callback `on_load_setup_pressed()` in `main.rs`
- [ ] Use `FileDialog` to select config file (.json)
- [ ] Call `load_config()`
- [ ] Validate loaded parameters
- [ ] Update all GUI fields with loaded values
- [ ] Switch to correct model type in UI
- [ ] Show success/error notification

**Deliverable:** Users can load previously saved setup

---

#### Task 4.3: Add Config Validation
**Priority:** MEDIUM
**Effort:** 2-3 hours

- [ ] Create `validate_config()` function in `config.rs`
- [ ] Validate numeric ranges (sigma > 0, prices > 0, etc.)
- [ ] Validate model-specific constraints:
  - GARCH: α + β < 1
  - Mean Reversion: θ > 0
  - Jump Diffusion: λ >= 0
- [ ] Return detailed error messages
- [ ] Show validation errors in GUI

**Deliverable:** Robust validation prevents invalid configurations

---

### **PHASE 5: TESTING & DOCUMENTATION** (Days 12-14)

#### Task 5.1: Write Comprehensive Unit Tests
**Priority:** HIGH
**Effort:** 8-10 hours

- [ ] Create `tests/model_tests.rs` for model-specific tests
- [ ] Test reproducibility for all 5 models (same seed → same results)
- [ ] Test antithetic variates reduce variance
- [ ] Test statistical properties:
  - Mean Reversion: convergence to mean
  - Jump Diffusion: jump frequency
  - GARCH: variance clustering
- [ ] Test edge cases (σ=0, λ=0, etc.)
- [ ] Run `cargo test` and ensure all pass
- [ ] Aim for >80% code coverage

**Deliverable:** All tests pass, high coverage

---

#### Task 5.2: Integration Testing
**Priority:** MEDIUM
**Effort:** 4-5 hours

- [ ] Test save/load round-trip (save → load → same config)
- [ ] Test GUI parameter updates
- [ ] Test all models produce valid paths (no NaN, no negative prices for GBM/JD)
- [ ] Test performance with large num_paths (10,000+)
- [ ] Profile for bottlenecks if needed

**Deliverable:** End-to-end workflows tested

---

#### Task 5.3: Update Documentation
**Priority:** MEDIUM
**Effort:** 3-4 hours

- [ ] Update README.md with:
  - New models explanation
  - Parameter descriptions
  - Save/load instructions
  - Example JSON config
- [ ] Add inline code comments for complex formulas
- [ ] Add docstrings to public functions
- [ ] Create example config files in `examples/` folder:
  - `example_gbm.json`
  - `example_mean_reversion.json`
  - `example_jump_diffusion.json`
  - `example_garch.json`

**Deliverable:** Clear documentation for all features

---

#### Task 5.4: Code Review & Refactoring
**Priority:** LOW
**Effort:** 2-3 hours

- [ ] Review all new code for consistency
- [ ] Check error handling coverage
- [ ] Ensure no unwrap() in production code (use ? or proper error handling)
- [ ] Run `cargo clippy` and fix warnings
- [ ] Run `cargo fmt` for consistent formatting
- [ ] Remove any debug print statements

**Deliverable:** Clean, production-ready code

---

## 📊 SPRINT 1 CHECKLIST

### Core Functionality
- [ ] Mean Reversion model working
- [ ] Jump Diffusion model working
- [ ] GARCH model working
- [ ] Dynamic parameter UI for all 5 models
- [ ] Save configuration to JSON
- [ ] Load configuration from JSON
- [ ] Parameter validation

### Quality Assurance
- [ ] All unit tests pass
- [ ] Integration tests pass
- [ ] No compiler warnings
- [ ] No clippy warnings
- [ ] Code coverage >80%

### Documentation
- [ ] README updated
- [ ] Code comments added
- [ ] Example config files created
- [ ] Parameter descriptions documented

### Performance
- [ ] All models support parallel execution (rayon)
- [ ] Antithetic variates implemented for all models
- [ ] No performance regression from baseline

---

## 🎯 DEFINITION OF DONE

Sprint 1 is complete when:

1. ✅ All 5 models (GBM, Bootstrap, Mean Reversion, Jump Diffusion, GARCH) produce valid simulation results
2. ✅ GUI dynamically shows parameters for selected model
3. ✅ Users can save current setup to JSON file
4. ✅ Users can load setup from JSON file
5. ✅ All unit tests pass with >80% coverage
6. ✅ Documentation is complete and accurate
7. ✅ Code passes `cargo clippy` and `cargo fmt`
8. ✅ Demo-ready: can show all 5 models working with save/load

---

## 📈 ESTIMATED EFFORT

| Phase | Tasks | Hours | Days |
|-------|-------|-------|------|
| Phase 1: Refactoring | 2 | 7-10h | 1-2 |
| Phase 2: New Models | 3 | 24-30h | 3-4 |
| Phase 3: GUI Updates | 2 | 12-15h | 2-3 |
| Phase 4: Save/Load | 3 | 10-13h | 2-3 |
| Phase 5: Testing/Docs | 4 | 17-22h | 2-3 |
| **TOTAL** | **14** | **70-90h** | **10-15 days** |

---

## 🚀 GETTING STARTED

**First Step:** Run this command to verify current state
```bash
cargo test
cargo clippy
cargo build --release
```

**Then:** Start with Phase 1, Task 1.1 (Refactor Model Architecture)

---

## 📝 NOTES

- Focus on one model at a time in Phase 2
- Test each model thoroughly before moving to next
- Keep backward compatibility with existing GBM/Bootstrap
- Commit frequently with clear messages
- If stuck on GARCH (most complex), implement Mean Reversion and Jump Diffusion first

---

## 🔗 RESOURCES

### Mean Reversion
- Ornstein-Uhlenbeck process: https://en.wikipedia.org/wiki/Ornstein%E2%80%93Uhlenbeck_process
- Discretization: Euler-Maruyama method

### Jump Diffusion
- Merton Model: https://en.wikipedia.org/wiki/Jump_diffusion
- Poisson process in Rust: `rand_distr::Poisson`

### GARCH
- GARCH(1,1): https://en.wikipedia.org/wiki/Autoregressive_conditional_heteroskedasticity
- Volatility clustering, stationarity condition

### Rust Resources
- serde JSON: https://serde.rs/
- rayon parallel: https://docs.rs/rayon/
- rand_distr: https://docs.rs/rand_distr/

---

**Good luck! 🚀**
