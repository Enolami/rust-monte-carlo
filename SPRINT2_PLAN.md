# SPRINT 2: PORTFOLIO ANALYSIS

**Duration:** 2-3 weeks
**Goal:** Extend single-ticker simulation to multi-ticker portfolio with correlation, stop-loss/target tracking, and advanced path visualization

**Prerequisites:** Sprint 1 completed (all 5 models working + save/load)

---

## 🎯 SPRINT OBJECTIVES

- [ ] Support multi-ticker portfolio simulation
- [ ] Implement correlation matrix for correlated price movements
- [ ] Add stop-loss and target price tracking per ticker
- [ ] Calculate portfolio-level statistics (probability of profit/loss, hit rates)
- [ ] Implement advanced path visualization (best/worst/median paths)
- [ ] Extend save/load to support portfolio configurations
- [ ] Create portfolio analytics dashboard

---

## 📋 DETAILED TASK BREAKDOWN

### **PHASE 1: DATA STRUCTURES & ARCHITECTURE** (Days 1-3)

#### Task 1.1: Create Portfolio Data Structures
**Priority:** HIGH
**Effort:** 6-8 hours

- [ ] Create new file `src/portfolio.rs`
- [ ] Define `TickerConfig` struct:
  ```rust
  #[derive(Serialize, Deserialize, Clone, Debug)]
  pub struct TickerConfig {
      pub symbol: String,
      pub initial_price: f64,
      pub weight: f64,              // Portfolio weight (0.0 - 1.0)
      pub stop_loss: Option<f64>,   // Stop loss price
      pub target: Option<f64>,      // Target price
      pub model_params: ModelParams,
  }
  ```

- [ ] Define `Portfolio` struct:
  ```rust
  #[derive(Serialize, Deserialize, Clone, Debug)]
  pub struct Portfolio {
      pub tickers: Vec<TickerConfig>,
      pub correlation_matrix: Option<Vec<Vec<f64>>>,  // NxN matrix
      pub total_capital: f64,
  }
  ```

- [ ] Add validation:
  - Weights sum to 1.0
  - Correlation matrix is symmetric and valid (-1 to 1)
  - Stop loss < initial price < target

- [ ] Add to `main.rs`: `mod portfolio;`

**Deliverable:** Portfolio data structures ready

---

#### Task 1.2: Create Portfolio Statistics Structures
**Priority:** HIGH
**Effort:** 4-5 hours

- [ ] Define `TickerStats` struct:
  ```rust
  #[derive(Debug, Clone)]
  pub struct TickerStats {
      pub symbol: String,
      pub mean_final_price: f64,
      pub median_final_price: f64,
      pub std_dev: f64,

      // Stop loss / Target tracking
      pub prob_hit_stoploss: f64,   // % paths that hit stop loss
      pub prob_hit_target: f64,      // % paths that hit target
      pub avg_time_to_stoploss: Option<f64>,  // Average days to hit SL
      pub avg_time_to_target: Option<f64>,    // Average days to hit target

      // Path indices for visualization
      pub best_path_idx: usize,      // Highest final price
      pub worst_path_idx: usize,     // Lowest final price
      pub median_path_idx: usize,    // Closest to median
  }
  ```

- [ ] Define `PortfolioStats` struct:
  ```rust
  #[derive(Debug, Clone)]
  pub struct PortfolioStats {
      pub ticker_stats: HashMap<String, TickerStats>,

      // Portfolio-level metrics
      pub mean_portfolio_return: f64,
      pub median_portfolio_return: f64,
      pub std_portfolio_return: f64,
      pub sharpe_ratio: f64,

      // Probability metrics
      pub prob_profit: f64,          // % paths with positive return
      pub prob_loss: f64,            // % paths with negative return
      pub mean_profit: f64,          // Average profit when profitable
      pub mean_loss: f64,            // Average loss when loss

      // VaR metrics
      pub var95: f64,
      pub cvar95: f64,               // Conditional VaR (Expected Shortfall)
  }
  ```

**Deliverable:** Statistics structures defined

---

#### Task 1.3: Update Config Module for Portfolio
**Priority:** HIGH
**Effort:** 3-4 hours

- [ ] Update `SimConfig` in `src/config.rs`:
  ```rust
  #[derive(Serialize, Deserialize, Clone, Debug)]
  pub struct SimConfig {
      // Simulation settings
      pub horizon: usize,
      pub num_paths: usize,
      pub seed: u64,
      pub use_antithetic: bool,
      pub dt: f64,

      // Portfolio configuration
      pub portfolio: Portfolio,
  }
  ```

- [ ] Update save/load functions to handle portfolio
- [ ] Add backward compatibility check (single ticker → portfolio with 1 ticker)
- [ ] Add validation for portfolio configuration

**Deliverable:** Config supports both single and multi-ticker setups

---

### **PHASE 2: CORRELATION IMPLEMENTATION** (Days 4-6)

#### Task 2.1: Implement Cholesky Decomposition
**Priority:** HIGH
**Effort:** 6-8 hours

- [ ] Add dependency to `Cargo.toml`:
  ```toml
  nalgebra = "0.32"  # For matrix operations
  ```

- [ ] Create `src/correlation.rs`
- [ ] Implement correlation matrix validation:
  ```rust
  pub fn validate_correlation_matrix(matrix: &[Vec<f64>]) -> Result<()> {
      // Check square
      // Check symmetric
      // Check diagonal = 1.0
      // Check values in [-1, 1]
      // Check positive semi-definite
  }
  ```

- [ ] Implement Cholesky decomposition:
  ```rust
  pub fn cholesky_decomposition(corr_matrix: &[Vec<f64>]) -> Result<Vec<Vec<f64>>> {
      // Use nalgebra for decomposition
      // Return lower triangular matrix L where L*L^T = corr_matrix
  }
  ```

- [ ] Implement correlated random number generation:
  ```rust
  pub fn generate_correlated_normals(
      cholesky: &[Vec<f64>],
      independent_z: &[f64]
  ) -> Vec<f64> {
      // Multiply L * Z to get correlated normals
  }
  ```

- [ ] Write unit tests:
  - [ ] Test identity matrix (no correlation)
  - [ ] Test perfect correlation (ρ=1)
  - [ ] Test negative correlation (ρ=-1)
  - [ ] Test random correlation matrix

**Deliverable:** Working correlation system with tests

---

#### Task 2.2: Update Simulation Engine for Correlated Paths
**Priority:** HIGH
**Effort:** 8-10 hours

- [ ] Create `run_portfolio_simulation()` in `core_sim.rs`:
  ```rust
  pub fn run_portfolio_simulation(
      config: SimConfig,
      hist_returns_map: HashMap<String, Vec<f64>>
  ) -> Result<(PortfolioStats, HashMap<String, Vec<Vec<f64>>>)> {
      // Returns stats + all paths for each ticker
  }
  ```

- [ ] Implement parallel portfolio simulation:
  - For each path i:
    - Generate N independent Z values (N = number of tickers)
    - Apply Cholesky to get correlated Z values
    - Simulate all tickers simultaneously with correlated Z
    - Track stop-loss/target hits
    - Calculate portfolio value at each step

- [ ] Update existing models to accept external Z values:
  ```rust
  fn generate_gbm_path_with_z(
      init_price: f64,
      mu: f64,
      sigma: f64,
      steps: usize,
      dt: f64,
      z_values: &[f64]  // Pre-generated correlated Z
  ) -> Vec<f64>
  ```

- [ ] Ensure all 5 models support correlated simulation

**Deliverable:** Portfolio simulation with correlation working

---

### **PHASE 3: STOP-LOSS & TARGET TRACKING** (Days 7-8)

#### Task 3.1: Implement Path Barrier Detection
**Priority:** HIGH
**Effort:** 5-6 hours

- [ ] Create `src/barriers.rs`
- [ ] Implement stop-loss detection:
  ```rust
  pub struct BarrierEvent {
      pub hit: bool,
      pub time_step: Option<usize>,
      pub price_at_hit: Option<f64>,
  }

  pub fn check_stop_loss(
      path: &[f64],
      stop_loss: f64
  ) -> BarrierEvent {
      // Find first time price <= stop_loss
      // Return step index and price
  }
  ```

- [ ] Implement target detection:
  ```rust
  pub fn check_target(
      path: &[f64],
      target: f64
  ) -> BarrierEvent {
      // Find first time price >= target
  }
  ```

- [ ] Implement combined barrier logic:
  ```rust
  pub fn check_barriers(
      path: &[f64],
      stop_loss: Option<f64>,
      target: Option<f64>
  ) -> (Option<BarrierEvent>, Option<BarrierEvent>) {
      // Return which barrier hit first (if any)
  }
  ```

- [ ] Write unit tests for edge cases:
  - [ ] Never hit barriers
  - [ ] Hit stop-loss only
  - [ ] Hit target only
  - [ ] Hit both (return first hit)

**Deliverable:** Barrier detection system working

---

#### Task 3.2: Calculate Hit Probabilities
**Priority:** HIGH
**Effort:** 4-5 hours

- [ ] Implement probability calculations:
  ```rust
  pub fn calculate_hit_probabilities(
      paths: &[Vec<f64>],
      stop_loss: Option<f64>,
      target: Option<f64>
  ) -> (f64, f64, Option<f64>, Option<f64>) {
      // Returns:
      // - prob_hit_stoploss
      // - prob_hit_target
      // - avg_time_to_stoploss
      // - avg_time_to_target
  }
  ```

- [ ] Integrate into `calculate_ticker_stats()`
- [ ] Add to portfolio statistics aggregation

**Deliverable:** Hit probability metrics calculated correctly

---

### **PHASE 4: PORTFOLIO ANALYTICS** (Days 9-11)

#### Task 4.1: Implement Portfolio Return Calculation
**Priority:** HIGH
**Effort:** 6-7 hours

- [ ] Implement weighted portfolio value calculation:
  ```rust
  pub fn calculate_portfolio_values(
      ticker_paths: &HashMap<String, Vec<Vec<f64>>>,
      weights: &HashMap<String, f64>,
      total_capital: f64
  ) -> Vec<Vec<f64>> {
      // For each path, calculate portfolio value at each time step
      // Portfolio_value = Σ(weight_i * price_i * shares_i)
  }
  ```

- [ ] Calculate portfolio returns:
  ```rust
  pub fn calculate_portfolio_returns(
      portfolio_values: &[Vec<f64>],
      initial_value: f64
  ) -> Vec<f64> {
      // Return for each path: (final - initial) / initial
  }
  ```

- [ ] Implement Sharpe ratio calculation:
  ```rust
  pub fn calculate_sharpe_ratio(
      returns: &[f64],
      risk_free_rate: f64,
      dt: f64
  ) -> f64 {
      // Sharpe = (mean_return - rf) / std_return
      // Annualize if needed
  }
  ```

**Deliverable:** Portfolio-level metrics calculated

---

#### Task 4.2: Implement Profit/Loss Analysis
**Priority:** HIGH
**Effort:** 5-6 hours

- [ ] Calculate profit/loss probabilities:
  ```rust
  pub fn calculate_pnl_stats(
      returns: &[f64]
  ) -> (f64, f64, f64, f64) {
      let positive_returns: Vec<f64> = returns.iter()
          .filter(|&&r| r > 0.0)
          .copied()
          .collect();

      let negative_returns: Vec<f64> = returns.iter()
          .filter(|&&r| r < 0.0)
          .copied()
          .collect();

      // Returns:
      // - prob_profit
      // - prob_loss
      // - mean_profit
      // - mean_loss
  }
  ```

- [ ] Calculate Conditional VaR (CVaR/Expected Shortfall):
  ```rust
  pub fn calculate_cvar(
      returns: &mut [f64],
      confidence: f64
  ) -> f64 {
      // Average of worst (1-confidence)% returns
  }
  ```

- [ ] Integrate into `PortfolioStats`

**Deliverable:** Complete profit/loss analytics

---

#### Task 4.3: Implement Per-Ticker Statistics
**Priority:** MEDIUM
**Effort:** 4-5 hours

- [ ] Calculate statistics for each ticker:
  ```rust
  pub fn calculate_ticker_stats(
      ticker: &str,
      paths: &[Vec<f64>],
      stop_loss: Option<f64>,
      target: Option<f64>
  ) -> TickerStats {
      // Calculate all metrics from TickerStats struct
  }
  ```

- [ ] Find best/worst/median path indices:
  ```rust
  pub fn find_representative_paths(
      paths: &[Vec<f64>]
  ) -> (usize, usize, usize) {
      let final_prices: Vec<f64> = paths.iter()
          .map(|p| *p.last().unwrap())
          .collect();

      let best_idx = argmax(&final_prices);
      let worst_idx = argmin(&final_prices);
      let median_idx = find_closest_to_median(&final_prices);

      (best_idx, worst_idx, median_idx)
  }
  ```

**Deliverable:** Detailed per-ticker analytics

---

### **PHASE 5: ADVANCED PATH VISUALIZATION** (Days 12-15)

#### Task 5.1: Update Plotting Module
**Priority:** HIGH
**Effort:** 8-10 hours

- [ ] Update `src/plotting.rs` to support multi-ticker plots
- [ ] Implement ticker selection for path display:
  ```rust
  pub struct PathVisualizationConfig {
      pub ticker_selections: HashMap<String, TickerPlotConfig>,
  }

  pub struct TickerPlotConfig {
      pub show_all_paths: bool,
      pub show_best: bool,
      pub show_worst: bool,
      pub show_median: bool,
      pub color: RGBColor,
  }
  ```

- [ ] Create `plot_portfolio_paths()`:
  ```rust
  pub fn plot_portfolio_paths(
      ticker_paths: &HashMap<String, Vec<Vec<f64>>>,
      ticker_stats: &HashMap<String, TickerStats>,
      viz_config: &PathVisualizationConfig
  ) -> Result<(Vec<u8>, u32, u32)> {
      // Plot selected paths for selected tickers
      // Use different colors per ticker
      // Highlight best/worst/median paths
      // Add legend
  }
  ```

- [ ] Add barrier lines to plot (stop-loss, target):
  ```rust
  // Draw horizontal lines for stop-loss and target
  // Use dashed lines
  // Add labels
  ```

- [ ] Implement multi-panel plot option:
  - Option 1: All tickers on same chart (with legend)
  - Option 2: Separate subplot per ticker

**Deliverable:** Multi-ticker path visualization with best/worst/median

---

#### Task 5.2: Create Portfolio Value Chart
**Priority:** MEDIUM
**Effort:** 4-5 hours

- [ ] Implement `plot_portfolio_value_paths()`:
  ```rust
  pub fn plot_portfolio_value_paths(
      portfolio_values: &[Vec<f64>],
      best_idx: usize,
      worst_idx: usize,
      median_idx: usize,
      show_sample_paths: bool
  ) -> Result<(Vec<u8>, u32, u32)> {
      // Plot portfolio total value over time
      // Highlight best/worst/median paths
      // Optionally show 20-50 sample paths
  }
  ```

- [ ] Add initial capital line as reference
- [ ] Show profit/loss zones (green/red)

**Deliverable:** Portfolio value visualization

---

#### Task 5.3: Update Histogram for Portfolio Returns
**Priority:** MEDIUM
**Effort:** 3-4 hours

- [ ] Update `plot_histogram()` to accept custom title and labels
- [ ] Create portfolio return distribution histogram
- [ ] Add vertical lines for:
  - Mean return
  - Median return
  - VaR95
  - CVaR95
- [ ] Color bars by profit (green) vs loss (red)

**Deliverable:** Enhanced histogram visualization

---

### **PHASE 6: GUI UPDATES FOR PORTFOLIO** (Days 16-19)

#### Task 6.1: Create Portfolio Configuration Panel
**Priority:** HIGH
**Effort:** 10-12 hours

- [ ] Update `gui.slint`: Add new "Portfolio Setup" tab
- [ ] Create ticker list/table component:
  ```slint
  component TickerRow {
      in-out property <string> symbol;
      in-out property <float> initial_price;
      in-out property <float> weight;
      in-out property <float> stop_loss;
      in-out property <float> target;

      callback remove_ticker();
  }
  ```

- [ ] Add "Add Ticker" button with popup dialog:
  - Text input for symbol
  - Number inputs for price, weight, stop-loss, target
  - Model selection dropdown
  - Parameter inputs (dynamic based on model)

- [ ] Add "Remove Ticker" button for each row

- [ ] Display total weight (should = 100%)
  - Warning if ≠ 100%
  - Auto-normalize option

- [ ] Add validation indicators (red/green)

**Deliverable:** Portfolio configuration UI working

---

#### Task 6.2: Create Correlation Matrix Input
**Priority:** HIGH
**Effort:** 8-10 hours

- [ ] Design correlation matrix UI (challenging!)

  **Option A: Grid Input**
  ```slint
  GridBox {
      // NxN grid of LineEdit fields
      // Upper triangle editable
      // Lower triangle mirrored automatically
      // Diagonal always 1.0
  }
  ```

  **Option B: Pairwise Input**
  ```slint
  // For each pair (i,j):
  // Slider from -1.0 to 1.0
  // Text: "Corr(AAPL, GOOGL): 0.75"
  ```

- [ ] Implement correlation matrix input
- [ ] Add "Use Default" button (identity matrix)
- [ ] Add "Estimate from Historical Data" button
  - Calculate correlation from loaded CSV data
  - Display estimated values

- [ ] Validate matrix (positive semi-definite)
- [ ] Show validation errors

**Deliverable:** Correlation matrix input working

---

#### Task 6.3: Create Portfolio Results Panel
**Priority:** HIGH
**Effort:** 8-10 hours

- [ ] Add new "Portfolio Results" tab in GUI
- [ ] Create portfolio-level metrics display:
  ```slint
  GroupBox {
      title: "Portfolio Metrics";
      GridBox {
          Row { Text { text: "Mean Return:"; } Text { text: "..."; } }
          Row { Text { text: "Std Deviation:"; } Text { text: "..."; } }
          Row { Text { text: "Sharpe Ratio:"; } Text { text: "..."; } }
          Row { Text { text: "Prob Profit:"; } Text { text: "..."; } }
          Row { Text { text: "Prob Loss:"; } Text { text: "..."; } }
          Row { Text { text: "VaR 95%:"; } Text { text: "..."; } }
          Row { Text { text: "CVaR 95%:"; } Text { text: "..."; } }
      }
  }
  ```

- [ ] Create per-ticker metrics table:
  ```slint
  TableView {
      columns: [
          "Ticker",
          "Mean Price",
          "Prob Hit SL",
          "Prob Hit Target",
          "Avg Time to SL",
          "Avg Time to Target"
      ]
  }
  ```

- [ ] Add export button for portfolio results

**Deliverable:** Portfolio results display complete

---

#### Task 6.4: Create Path Visualization Controls
**Priority:** MEDIUM
**Effort:** 6-8 hours

- [ ] Add path visualization control panel:
  ```slint
  GroupBox {
      title: "Path Display Options";

      // For each ticker in portfolio:
      HorizontalBox {
          Text { text: "AAPL:"; }
          CheckBox { text: "Show"; }
          CheckBox { text: "Best"; }
          CheckBox { text: "Worst"; }
          CheckBox { text: "Median"; }
          CheckBox { text: "Sample Paths"; }
      }
  }
  ```

- [ ] Add callback `on_path_viz_changed()`
- [ ] Re-render charts when options change
- [ ] Add "Show All" / "Hide All" buttons
- [ ] Add color picker per ticker (optional)

**Deliverable:** Interactive path visualization controls

---

### **PHASE 7: SAVE/LOAD PORTFOLIO CONFIG** (Days 20-21)

#### Task 7.1: Update Save/Load for Portfolio
**Priority:** HIGH
**Effort:** 4-5 hours

- [ ] Update `save_config()` to handle portfolio
- [ ] Ensure correlation matrix serializes correctly
- [ ] Add version field to config for backward compatibility:
  ```rust
  #[derive(Serialize, Deserialize)]
  pub struct SimConfig {
      pub version: u32,  // 1 = single ticker, 2 = portfolio
      // ...
  }
  ```

- [ ] Test save/load with:
  - Single ticker (backward compat)
  - Multiple tickers
  - With correlation matrix
  - Without correlation matrix

**Deliverable:** Portfolio configs can be saved and loaded

---

#### Task 7.2: Create Example Portfolio Configs
**Priority:** LOW
**Effort:** 2-3 hours

- [ ] Create `examples/portfolio_tech_stocks.json`:
  - AAPL, GOOGL, MSFT
  - Realistic correlation (~0.7)
  - GBM model

- [ ] Create `examples/portfolio_diversified.json`:
  - Stocks, bonds (mean reversion)
  - Low correlation (~0.2)
  - Mixed models

- [ ] Create `examples/portfolio_with_barriers.json`:
  - Multiple tickers with stop-loss and targets
  - Demonstrate risk management

**Deliverable:** Example configs for demo

---

### **PHASE 8: TESTING & OPTIMIZATION** (Days 22-24)

#### Task 8.1: Write Portfolio Unit Tests
**Priority:** HIGH
**Effort:** 8-10 hours

- [ ] Test correlation matrix operations:
  - [ ] Cholesky decomposition
  - [ ] Correlated random generation
  - [ ] Matrix validation

- [ ] Test barrier detection:
  - [ ] Stop-loss hit
  - [ ] Target hit
  - [ ] Both hit
  - [ ] Neither hit

- [ ] Test portfolio statistics:
  - [ ] Weighted returns
  - [ ] Sharpe ratio
  - [ ] Profit/loss probabilities

- [ ] Test with edge cases:
  - [ ] Single ticker portfolio
  - [ ] Equal-weighted portfolio
  - [ ] Portfolio with one dominant ticker (99% weight)

**Deliverable:** Comprehensive test suite

---

#### Task 8.2: Integration Testing
**Priority:** HIGH
**Effort:** 6-8 hours

- [ ] Test full workflow:
  1. Load CSV with multiple tickers
  2. Create portfolio
  3. Set correlation
  4. Run simulation
  5. View results
  6. Save config
  7. Load config
  8. Re-run simulation (should match)

- [ ] Test performance with:
  - [ ] 2 tickers, 1000 paths
  - [ ] 5 tickers, 1000 paths
  - [ ] 10 tickers, 1000 paths
  - [ ] 5 tickers, 10000 paths

- [ ] Profile performance bottlenecks
- [ ] Optimize if needed (parallel correlation?)

**Deliverable:** End-to-end workflows tested

---

#### Task 8.3: Performance Optimization
**Priority:** MEDIUM
**Effort:** 6-8 hours

- [ ] Profile correlation matrix operations
  - Consider caching Cholesky decomposition

- [ ] Optimize portfolio value calculation
  - Pre-allocate vectors
  - Use SIMD if possible

- [ ] Parallelize per-ticker statistics calculation
  - Use rayon for ticker_stats map

- [ ] Benchmark improvements
- [ ] Target: <5 seconds for 5 tickers, 10000 paths

**Deliverable:** Optimized performance

---

#### Task 8.4: Update Documentation
**Priority:** MEDIUM
**Effort:** 4-5 hours

- [ ] Update README.md:
  - Portfolio usage guide
  - Correlation matrix explanation
  - Stop-loss/target examples
  - Path visualization guide

- [ ] Add inline documentation:
  - Correlation functions
  - Barrier detection
  - Portfolio statistics

- [ ] Create user guide: `PORTFOLIO_GUIDE.md`
  - Step-by-step tutorial
  - Screenshots (after implementation)
  - Example use cases

**Deliverable:** Complete documentation

---

### **PHASE 9: POLISH & DEMO PREPARATION** (Days 25-26)

#### Task 9.1: UI/UX Improvements
**Priority:** MEDIUM
**Effort:** 4-6 hours

- [ ] Add loading indicators for long simulations
- [ ] Add progress bar for portfolio simulation
- [ ] Improve error messages (user-friendly)
- [ ] Add tooltips explaining metrics
- [ ] Add input validation feedback (real-time)
- [ ] Test keyboard navigation
- [ ] Test with different window sizes

**Deliverable:** Polished user experience

---

#### Task 9.2: Create Demo Scenarios
**Priority:** LOW
**Effort:** 3-4 hours

- [ ] Prepare demo data:
  - Real historical data CSV (if available)
  - Or generate realistic synthetic data

- [ ] Create demo scripts:
  - Scenario 1: Tech portfolio (high correlation)
  - Scenario 2: Diversified portfolio (low correlation)
  - Scenario 3: Risk management with barriers

- [ ] Test demo flow end-to-end
- [ ] Time each demo (<5 minutes)

**Deliverable:** Ready-to-demo scenarios

---

#### Task 9.3: Final Code Review
**Priority:** HIGH
**Effort:** 3-4 hours

- [ ] Run `cargo clippy --all-targets`
- [ ] Run `cargo fmt --all`
- [ ] Review all error handling
- [ ] Check for panics/unwraps
- [ ] Verify all TODOs resolved
- [ ] Check code comments
- [ ] Review commit messages

**Deliverable:** Production-ready code

---

## 📊 SPRINT 2 CHECKLIST

### Core Functionality
- [ ] Multi-ticker portfolio simulation
- [ ] Correlation matrix support
- [ ] Stop-loss tracking
- [ ] Target tracking
- [ ] Portfolio-level statistics
- [ ] Per-ticker statistics
- [ ] Best/worst/median path identification

### Visualization
- [ ] Multi-ticker path plots
- [ ] Portfolio value plots
- [ ] Enhanced histograms
- [ ] Path visualization controls
- [ ] Barrier lines on charts

### GUI
- [ ] Portfolio configuration panel
- [ ] Correlation matrix input
- [ ] Portfolio results display
- [ ] Path visualization controls
- [ ] Ticker management (add/remove)

### Save/Load
- [ ] Portfolio config save
- [ ] Portfolio config load
- [ ] Backward compatibility
- [ ] Example configs

### Quality Assurance
- [ ] All unit tests pass
- [ ] Integration tests pass
- [ ] Performance targets met
- [ ] No compiler warnings
- [ ] No clippy warnings
- [ ] Code coverage >75%

### Documentation
- [ ] README updated
- [ ] Portfolio guide created
- [ ] Code documented
- [ ] Example configs documented

---

## 🎯 DEFINITION OF DONE

Sprint 2 is complete when:

1. ✅ Users can create portfolios with multiple tickers
2. ✅ Correlation between tickers is properly simulated
3. ✅ Stop-loss and target prices are tracked
4. ✅ Portfolio-level statistics are calculated and displayed
5. ✅ Users can visualize best/worst/median paths per ticker
6. ✅ Users can toggle which tickers/paths to display
7. ✅ Portfolio configs can be saved and loaded
8. ✅ All tests pass with >75% coverage
9. ✅ Performance is acceptable (5 tickers, 10k paths <5s)
10. ✅ Documentation is complete
11. ✅ Demo scenarios prepared and tested

---

## 📈 ESTIMATED EFFORT

| Phase | Tasks | Hours | Days |
|-------|-------|-------|------|
| Phase 1: Architecture | 3 | 13-17h | 2-3 |
| Phase 2: Correlation | 2 | 14-18h | 2-3 |
| Phase 3: Barriers | 2 | 9-11h | 1-2 |
| Phase 4: Analytics | 3 | 15-18h | 2-3 |
| Phase 5: Visualization | 3 | 15-19h | 2-3 |
| Phase 6: GUI Updates | 4 | 32-40h | 4-5 |
| Phase 7: Save/Load | 2 | 6-8h | 1-2 |
| Phase 8: Testing | 4 | 24-31h | 3-4 |
| Phase 9: Polish | 3 | 10-14h | 2-3 |
| **TOTAL** | **26** | **138-176h** | **19-28 days** |

---

## 🚀 GETTING STARTED

**Prerequisites:**
- Sprint 1 completed and tested
- All 5 models working
- Save/load functional

**First Step:** Run this to verify Sprint 1 is ready
```bash
cargo test
cargo build --release
# Test all 5 models in GUI
# Test save/load config
```

**Then:** Start with Phase 1, Task 1.1 (Portfolio Data Structures)

---

## 📝 NOTES

### Technical Challenges:
1. **Correlation Matrix UI** - Most challenging part, consider simple pairwise sliders first
2. **Performance** - Portfolio simulation is computationally expensive, profile early
3. **Complexity** - GUI will be complex, consider wireframes before coding

### Recommendations:
- Implement correlation without GUI first (test with code)
- Start with 2-ticker portfolio, then scale to N tickers
- Test performance early and often
- Consider async UI updates for long simulations

### Dependencies on Sprint 1:
- `ModelParams` enum must be complete
- `SimConfig` save/load must work
- All 5 models must be stable
- GUI structure should be established

---

## 🔗 RESOURCES

### Correlation & Portfolio Theory
- Cholesky decomposition: https://en.wikipedia.org/wiki/Cholesky_decomposition
- Portfolio theory: https://en.wikipedia.org/wiki/Modern_portfolio_theory
- Correlation matrix: https://en.wikipedia.org/wiki/Covariance_matrix

### Rust Libraries
- nalgebra (matrix operations): https://docs.rs/nalgebra/
- statrs (statistics): https://docs.rs/statrs/
- rayon (parallelism): https://docs.rs/rayon/

### Risk Metrics
- Value at Risk (VaR): https://en.wikipedia.org/wiki/Value_at_risk
- Conditional VaR (CVaR): https://en.wikipedia.org/wiki/Expected_shortfall
- Sharpe Ratio: https://en.wikipedia.org/wiki/Sharpe_ratio

### Testing
- Property-based testing: Consider `proptest` crate for correlation matrix validation
- Benchmark: Use `criterion` crate for performance benchmarks

---

## 🎨 WIREFRAME IDEAS

### Portfolio Configuration Panel:
```
┌─────────────────────────────────────────────────┐
│ Portfolio Setup                                 │
├─────────────────────────────────────────────────┤
│ Ticker │ Price │ Weight │ Stop Loss │ Target   │
├────────┼───────┼────────┼───────────┼──────────┤
│ AAPL   │ 150   │  30%   │   140     │   170    │
│ GOOGL  │ 120   │  40%   │   110     │   140    │
│ MSFT   │ 300   │  30%   │   280     │   330    │
├────────┴───────┴────────┴───────────┴──────────┤
│ Total Weight: 100% ✓          [Add Ticker]     │
└─────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────┐
│ Correlation Matrix                              │
├─────────────────────────────────────────────────┤
│ AAPL ↔ GOOGL:  [========|===] 0.75              │
│ AAPL ↔ MSFT:   [=======|====] 0.70              │
│ GOOGL ↔ MSFT:  [=========|==] 0.80              │
├─────────────────────────────────────────────────┤
│ [Use Default] [Estimate from Data]              │
└─────────────────────────────────────────────────┘
```

### Path Visualization Controls:
```
┌─────────────────────────────────────────────────┐
│ Path Display                                    │
├─────────────────────────────────────────────────┤
│ ☑ AAPL:   ☑ Best  ☑ Worst  ☑ Median  ☐ Samples │
│ ☑ GOOGL:  ☑ Best  ☑ Worst  ☑ Median  ☐ Samples │
│ ☐ MSFT:   ☐ Best  ☐ Worst  ☐ Median  ☐ Samples │
├─────────────────────────────────────────────────┤
│ [Show All] [Hide All] [Only Best/Worst]         │
└─────────────────────────────────────────────────┘
```

---

**Good luck with Sprint 2! 🚀**

*This is a complex sprint - don't hesitate to break tasks down further if needed*
