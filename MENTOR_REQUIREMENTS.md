# Yêu Cầu Từ Mentor - Monte Carlo Stock Price Simulator

## 📌 Tổng Quan
Mentor yêu cầu mở rộng ứng dụng Monte Carlo Stock Price Simulator với các tính năng nâng cao để hoàn thiện assignment.

---

## 🎯 YÊU CẦU CHÍNH

### **1. Mở Rộng Mô Hình Mô Phỏng (Sprint 1 - ✅ HOÀN THÀNH)**

#### **1.1. Thêm 3 Mô Hình Mới**
- ✅ **Mean Reversion (Ornstein-Uhlenbeck Process)**
  - Parameters: θ (speed of reversion), μ (long-term mean), σ (volatility)
  - Formula: `dS = θ(μ - S)dt + σdW`

- ✅ **Jump Diffusion (Merton Model)**
  - Parameters: μ (drift), σ (volatility), λ (jump intensity), μ_j (jump mean), σ_j (jump std)
  - Combines GBM with Poisson jump process

- ✅ **GARCH(1,1) Model**
  - Parameters: ω (constant), α (ARCH coefficient), β (GARCH coefficient)
  - Time-varying volatility: `σ²_{t+1} = ω + α·r²_t + β·σ²_t`

#### **1.2. Dynamic GUI Parameters**
- ✅ Conditional parameter panels based on selected model
- ✅ Show/hide model-specific inputs
- ✅ ScrollView for long parameter lists

#### **1.3. Save/Load Configuration**
- ✅ Save current simulation setup to JSON file
- ✅ Load saved configuration from JSON file
- ✅ Persist all model parameters and settings

---

### **2. Portfolio Simulation Features (Sprint 2 - 🔜 PENDING)**

#### **2.1. Multi-Ticker Support**
- [ ] Load và quản lý nhiều tickers từ CSV
- [ ] Weighted portfolio với custom weights cho mỗi ticker
- [ ] Portfolio value calculation: `V(t) = Σ(w_i × S_i(t))`

#### **2.2. Correlation Modeling**
- [ ] Estimate correlation matrix từ historical data
- [ ] Cholesky decomposition cho correlated random variables
- [ ] Visualize correlation heatmap

#### **2.3. Risk Management**
- [ ] Stop-loss thresholds (exit khi giá < threshold)
- [ ] Target profit levels (exit khi giá > threshold)
- [ ] Track exit events trong simulation paths

---

### **3. Advanced Visualization (Sprint 2 - 🔜 PENDING)**

#### **3.1. Enhanced Path Visualization**
- [ ] Highlight best/worst/median paths
- [ ] Color-coded paths by performance
- [ ] Percentile bands (P5-P95 shaded area)

#### **3.2. Portfolio Charts**
- [ ] Portfolio value distribution
- [ ] Individual asset contributions
- [ ] Correlation heatmap

---

## 📊 TRẠNG THÁI HIỆN TẠI

### ✅ Sprint 1 (HOÀN THÀNH - 100%)
- ✅ Phase 1: Refactor architecture with ModelParams enum
- ✅ Phase 2: Implement 3 new models (Mean Reversion, Jump Diffusion, GARCH)
- ✅ Phase 3: Dynamic GUI with conditional parameters
- ✅ Phase 4: Save/Load configuration functionality
- ✅ ScrollView for better UX with long parameter lists

### 🔜 Sprint 2 (CHƯA BẮT ĐẦU)
- Portfolio features (multi-ticker, correlation, risk management)
- Advanced visualization (best/worst paths, percentile bands)
- Detailed plan trong file: [SPRINT2_PLAN.md](SPRINT2_PLAN.md)

---

## 📁 CẤU TRÚC DỰ ÁN

### **Các File Quan Trọng**
```
rust-monte-carlo/
├── src/
│   ├── main.rs              # Main application + callbacks
│   ├── core_sim.rs          # Simulation models (GBM, Bootstrap, MR, JD, GARCH)
│   ├── config.rs            # Configuration save/load (JSON)
│   ├── data_io.rs           # CSV data loading
│   ├── plotting.rs          # Chart generation (plotters)
│   └── gui.slint            # Slint UI definition
├── SPRINT1_PLAN.md          # Sprint 1 detailed plan ✅
├── SPRINT2_PLAN.md          # Sprint 2 detailed plan 🔜
└── MENTOR_REQUIREMENTS.md   # This file
```

---

## 🎓 KẾT QUẢ ĐẠT ĐƯỢC (Sprint 1)

### **1. 5 Simulation Models**
- GBM (Geometric Brownian Motion)
- Bootstrap (Historical resampling)
- Mean Reversion (Ornstein-Uhlenbeck)
- Jump Diffusion (Merton model)
- GARCH(1,1) (Time-varying volatility)

### **2. Features**
- ✅ Dynamic parameter panels per model
- ✅ Save/Load simulation configuration (JSON)
- ✅ Antithetic variates for variance reduction
- ✅ Parallel execution với Rayon
- ✅ Reproducible results với seed control
- ✅ ScrollView cho Control Panel

### **3. Export Functionality**
- ✅ Export summary statistics to CSV
- ✅ Export charts to PNG
- ✅ Export/import configuration to JSON

---

## 📝 GHI CHÚ

### **Assignment Requirements (Week 2)**
- File gốc: `Week2_Assignment_MonteCarlo_EN.txt`
- Yêu cầu ban đầu: 5 models, statistical analysis, visualization
- ✅ Đã hoàn thành tất cả yêu cầu cơ bản

### **Mentor Enhancement Requests**
- Thêm 3 models mới: ✅ Hoàn thành
- Portfolio features: 🔜 Sprint 2
- Advanced visualization: 🔜 Sprint 2
- Save/load setup: ✅ Hoàn thành

---

## 🚀 BƯỚC TIẾP THEO

**Tuỳ chọn:**

1. **Complete Sprint 2** → Implement portfolio features
   - Multi-ticker simulation
   - Correlation modeling
   - Stop-loss/target mechanisms

2. **Testing & Documentation** → Improve code quality
   - Unit tests for reproducibility
   - Code documentation
   - Example config files

3. **Demo Preparation** → Prepare for mentor review
   - Test all features thoroughly
   - Create demo scenarios
   - Document usage examples

---

**Ngày cập nhật:** 2025-11-21
**Trạng thái:** Sprint 1 hoàn thành, sẵn sàng cho Sprint 2
