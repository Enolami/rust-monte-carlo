use anyhow::Result;
use image::{ImageEncoder, codecs::png::PngEncoder};
use rayon::vec;
use rfd::FileDialog;
use slint::{Image, ModelRc, PlatformError, SharedString, VecModel};
use std::{cell::RefCell, collections::HashMap, fs::{self, File}, rc::Rc, thread, time::Instant};

use crate::{core_sim::{SimStats as rustSimStats, estimate_paramaters, run_portfolio_simulation, run_simulation}, porfolio::build_portfolio_config};
use crate::data_io::{get_ticker_info, load_all_records}; 
use crate::slint_generatedAppWindow::SimStats as slintSimStats;


slint::include_modules!();
mod data_io;
mod core_sim;
mod plotting;
mod porfolio;

#[derive(Default, Debug, Clone)]
struct AppState {
    all_data_map: HashMap<String, Vec<crate::data_io::StockRecord>>,
    tickers: Vec<String>,

    selected_ticker: String,
    selected_ticker_last_price: f64,
    selected_ticker_log_returns: Vec<f64>,

    portfolio_tickers: Vec<(String, f64)>,

    last_paths_chart_png_raw: (Vec<u8>, u32, u32),
    last_hist_chart_png_raw: (Vec<u8>, u32, u32),
}

fn main() -> Result<(), PlatformError> {
    let main_window = AppWindow::new()?;
    let app_state = Rc::new(RefCell::new(AppState::default()));

    setup_callbacks(&main_window, app_state);

    main_window.run()
}

fn setup_callbacks(main_window: &AppWindow, app_state: Rc<RefCell<AppState>>) {
    let main_window_weak = main_window.as_weak();

    //Read csv file
    main_window.on_load_csv_pressed({
        let mw_weak = main_window_weak.clone();
        let app_state = app_state.clone();
        move || {
            if let Some(path) = FileDialog::new().add_filter("CSV file", &["csv"]).pick_file() {
                match load_all_records(path) {
                    Ok((all_records, tickers)) => {
                        let mut state = app_state.borrow_mut();
                        state.all_data_map = all_records;
                        state.tickers = tickers.clone();

                        let ticker_shared: Vec<SharedString> = tickers.into_iter().map(SharedString::from).collect();
                        let model: ModelRc<SharedString> = ModelRc::from(Rc::new(VecModel::from(ticker_shared)));

                        if let Some(mw) = mw_weak.upgrade() {
                            mw.set_ticker_list(model);
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to load CSV {:?}",e);
                    }
                }
            }
        }
    });

    //read selected ticker
    main_window.on_select_ticker_changed({
        let mw_weak = main_window_weak.clone();
        let app_state = app_state.clone();
        move || {
            if let Some(mw) = mw_weak.upgrade() {
                let app_state_clone = app_state.clone();
                let mut state = app_state_clone.borrow_mut();

                let selected_ticker = mw.get_selected_ticker();
                state.selected_ticker = selected_ticker.to_string().clone();

                let (info, log_returns) = get_ticker_info(&state.all_data_map, &selected_ticker);
                
                if let Some(records) = state.all_data_map.get(&state.selected_ticker) {
                    if let Some(last) = records.last() {
                        state.selected_ticker_last_price = last.close;
                    }
                }

                state.selected_ticker_log_returns = log_returns;
                
                let lines: Vec<&str> = info.lines().collect();
                let date_range: SharedString = SharedString::from(*lines.get(1).unwrap_or(&""));
                let record_count: SharedString = SharedString::from(*lines.get(2).unwrap_or(&""));

                mw.set_date_range(date_range);
                mw.set_record_count(record_count);
            }
        }
    });

    //calculate sigma and mu from last log returns
    main_window.on_estimate_params_pressed({
        let mw_weak = main_window_weak.clone();
        let app_state = app_state.clone();
        move || {
            if let Some(mw) = mw_weak.upgrade() {
                let state = app_state.borrow();
                if state.selected_ticker_log_returns.is_empty() {
                    return;
                }

                match estimate_paramaters(&state.selected_ticker_log_returns) {
                    Ok((mu, sigma)) => {
                        mw.set_mu(mu as f32);
                        mw.set_sigma(sigma as f32);

                        mw.set_initial_price(state.selected_ticker_last_price as f32);
                    }
                    Err(e) => {
                        eprintln!("Error estimating: {}", e);
                    }
                }
            }
        }
    });

    main_window.on_add_ticker_to_portfolio({
        let mw_weak = main_window_weak.clone();
        let app_state = app_state.clone();
        move |ticker, weight| {
            if let Some(mw) = mw_weak.upgrade() {
                let mut state = app_state.borrow_mut();
                if ticker.is_empty() {return;}
                if !state.portfolio_tickers.iter().any(|(t, _)| t == &ticker.to_string()) {
                    state.portfolio_tickers.push((ticker.to_string(), weight as f64));
                }

                let ui_model: Vec<PortfolioItem> = state.portfolio_tickers.iter().map(|(t, w)| {
                    PortfolioItem { ticker:SharedString::from(t), weight: *w as f32 }
                }).collect();
                mw.set_portfolio_model(ModelRc::from(Rc::new(VecModel::from(ui_model))));

                let total_w: f64 = state.portfolio_tickers.iter().map(|(_,w)| w).sum();
                mw.set_current_portfolio_weight(total_w as f32);
            }
        }
    });

    main_window.on_clear_portfolio({
        let mw_weak = main_window_weak.clone();
        let app_state = app_state.clone();
        move || {
            if let Some(mw) = mw_weak.upgrade() {
                let mut state = app_state.borrow_mut();
                state.portfolio_tickers.clear();
                mw.set_portfolio_model(ModelRc::from(Rc::new(VecModel::from(vec![]))));
                mw.set_current_portfolio_weight(0.0);
            }
        }
    });

    //run sim and display png
     main_window.on_run_simulation_pressed({
        let mw_weak = main_window_weak.clone();
        let app_state = app_state.clone();
        move |params| {
            if let Some(mw) = mw_weak.upgrade() {
                let start_time = Instant::now();
                let state = app_state.borrow();

                // SWITCH: Single vs Portfolio
                let result = if params.is_portfolio {
                    if state.portfolio_tickers.is_empty() {
                        eprintln!("Portfolio empty");
                        return;
                    }
                    // Build config and run portfolio sim
                    match build_portfolio_config(&state.portfolio_tickers, params.initial_price as f64, &state.all_data_map) {
                        Ok(config) => run_portfolio_simulation(params, config),
                        Err(e) => Err(e),
                    }
                } else {
                    // Original Single Sim
                    let hist_log_returns = state.selected_ticker_log_returns.clone();
                    if hist_log_returns.is_empty() && params.model_type == "Bootstrap" {
                        return;
                    }
                    run_simulation(params, hist_log_returns)
                };

                match result {
                    Ok((stats, (paths_buf, paths_w, paths_h), (hist_buf, hist_w, hist_h))) => {
                        let duration = start_time.elapsed().as_millis();
                        mw.set_exec_time(format!("{} ms", duration).into());

                        let ui_stats = slintSimStats{
                            mean: stats.mean as f32,
                            std_dev: stats.std_dev as f32,
                            median: stats.median as f32,
                            p5: stats.p5 as f32,
                            p25: stats.p25 as f32,
                            p75: stats.p75 as f32,
                            p95: stats.p95 as f32,
                            var95: stats.var95 as f32,
                        };
                        mw.set_stats(ui_stats);

                        let paths_pixel_buffer = slint::SharedPixelBuffer::clone_from_slice(&paths_buf, paths_w, paths_h);
                        mw.set_price_chart(Image::from_rgb8(paths_pixel_buffer));

                        let hist_pixel_buffer = slint::SharedPixelBuffer::clone_from_slice(&hist_buf, hist_w, hist_h);
                        mw.set_hist_chart(Image::from_rgb8(hist_pixel_buffer));
                        
                        // (Ideally update app_state.last_... here for exports)
                    }
                    Err(e) => {
                        eprintln!("Simulation error: {}", e);
                    }
                }
            }
        }
    });

    //save summary.csv file
    main_window.on_export_summary_pressed({
        let mw_weak = main_window_weak.clone();
        move || {
            if let Some(mw) = mw_weak.upgrade() {
                let stats = mw.get_stats();
                let exec_time = mw.get_exec_time();
                let mw_weak_clone = mw.as_weak();

                let horizons = mw.get_horizon();
                let num_paths = mw.get_num_paths();
                let model = mw.get_model_type().to_string();

                let full_stats = rustSimStats {
                    horizon: horizons as usize,
                    paths: num_paths as usize,
                    model: model,
                    mean: stats.mean as f64,
                    std_dev: stats.std_dev as f64,
                    median: stats.median as f64,
                    p5: stats.p5 as f64,
                    p25: stats.p25 as f64,
                    p75: stats.p75 as f64,
                    p95: stats.p95 as f64,
                    var95: stats.var95 as f64,
                };

                //avoid freeze
                thread::spawn(move || {
                    let summary_csv = format!(
                        "Metric,Value\nExecTime,{}\nModel,{}\nHorizon,{}\nPaths,{}\nMean,{:.4}\nStdDev,{:.4}\nMedian,{:.4}\nP5,{:.4}\nP25,{:.4}\nP75,{:.4}\nP95,{:.4}\nVaR95,{:.4}\n",
                        exec_time, full_stats.model, full_stats.horizon, full_stats.paths, full_stats.mean, full_stats.std_dev, full_stats.median, full_stats.p5, full_stats.p25, full_stats.p75, full_stats.p95, full_stats.var95
                    );
                    
                    let file = FileDialog::new()
                        .add_filter("CSV", &["csv"])
                        .set_file_name("simulation_summary.csv")
                        .save_file();

                    if let Some(path) = file {
                        match fs::write(path, summary_csv) {
                            Ok(_) => {}
                            Err(e) => {eprintln!("Error save summary file: {}", e)}
                        }
                    }

                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(mw) = mw_weak_clone.upgrade() {
                        }
                    });
                });
            }
        }
    });

    //save png files
    main_window.on_export_charts_pressed({
        let mw_weak = main_window_weak.clone();
        let app_state = app_state.clone();
        move || {
            if let Some(mw) = mw_weak.upgrade() {
                let state = app_state.borrow();
                if state.last_hist_chart_png_raw.0.is_empty() || state.last_hist_chart_png_raw.0.is_empty() {
                    return;
                }

                let file = FileDialog::new().add_filter("PNG", &["png"]).set_file_name("simulation_charts.png").save_file();

                if let Some(path) = file {
                    let mut paths_path = path.clone();
                    paths_path.set_file_name(format!("{}_paths.png", paths_path.file_stem().unwrap().to_str().unwrap()));

                    let mut hist_path = path.clone();
                    hist_path.set_file_name(format!("{}_hist.png",hist_path.file_stem().unwrap().to_str().unwrap()));

                    let (buf, w, h) = &state.last_paths_chart_png_raw;
                    let p_res = encode_and_save_png(&paths_path, buf, *w, *h);

                    let (buf, w, h) = &state.last_hist_chart_png_raw;
                    let h_res = encode_and_save_png(&hist_path, buf, *w, *h);

                    match (p_res, h_res) {
                        (Ok(_), Ok(_)) => {}
                        (Err(e), _) | (_, Err(e)) => {eprintln!("Error saving charts: {}", e);}
                    }
                    
                }
            }
        }
    });
}

//encode from rgb<u8> to png
fn encode_and_save_png(path: &std::path::Path, buf: &[u8], width: u32, height: u32) -> Result<()> {
    let file = File::create(path)?;
    let encoder = PngEncoder::new(file);
    encoder.write_image(buf, width, height, image::ColorType::Rgb8.into())?;
    Ok(())
}