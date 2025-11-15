use anyhow::{Ok, Result};
use plotters::{backend, prelude::*};
use plotters_bitmap::bitmap_pixel::RGBPixel;
use plotters_bitmap::BitMapBackend;

const CHART_WIDTH: u32 = 800;
const CHART_HEIGHT: u32 = 600;

pub fn plot_price_paths(paths: &[Vec<f64>]) -> Result<(Vec<u8>, u32, u32)> {
    let mut buf = vec![0; (CHART_WIDTH * CHART_HEIGHT * 3) as usize];
    let mut backend = BitMapBackend::<RGBPixel>::with_buffer_and_format(
        &mut buf, (CHART_WIDTH, CHART_HEIGHT))?;
    {
        let root = backend.into_drawing_area();
        root.fill(&RGBColor(30, 30, 46))?;

        if paths.is_empty() || paths[0].is_empty() {
            root.draw(&EmptyElement::at((0,0)))?;
            return Ok((vec![0; (CHART_WIDTH * CHART_HEIGHT * 3) as usize], CHART_WIDTH, CHART_HEIGHT));
        }

        let mut min_price = paths[0][0];
        let mut max_price = paths[0][0];
        for path in paths.iter() {
            for &price in path.iter() {
                if price < min_price {
                    min_price = price;
                }
                if price > max_price {
                    max_price = price;
                }
            }
        }
        
        min_price *= 0.95;
        max_price *= 1.05;

        let max_steps = paths[0].len() - 1;

        let mut chart = ChartBuilder::on(&root)
            .caption(
                "Simulated Price Paths",
                ("Inter", 30, &RGBColor(208, 208, 208)),
            )
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(0..max_steps, min_price..max_price)?;

        chart
            .configure_mesh()
            .axis_style(&RGBColor(208, 208, 208))
            .label_style(("Inter", 15, &RGBColor(208, 208, 208)))
            .draw()?;

        for path in paths.iter().take(50) {
            chart.draw_series(LineSeries::new(
                path.iter().enumerate().map(|(i, &p)| (i, p)),
                &YELLOW.mix(0.1),
            ))?;
        }
    }

    Ok((buf, CHART_WIDTH, CHART_HEIGHT))
}

pub fn plot_histogram(data: &[f64], num_bins: usize) -> Result<(Vec<u8>, u32, u32)> {
    let mut buf = vec![0; (CHART_WIDTH * CHART_HEIGHT * 3) as usize];
    let mut backend = BitMapBackend::<RGBPixel>::with_buffer_and_format(
        &mut buf,
        (CHART_WIDTH, CHART_HEIGHT),
    )?;

    {
        let root = backend.into_drawing_area();
        root.fill(&RGBColor(30, 30, 46))?;

        if data.is_empty() {
            root.draw(&EmptyElement::at((0, 0)))?;
            return Ok((vec![0; (CHART_WIDTH * CHART_HEIGHT * 3) as usize], CHART_WIDTH, CHART_HEIGHT));
        }

        let min_val = *data
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let max_val = *data
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        let mut chart = ChartBuilder::on(&root)
            .caption(
                "Terminal Price Distribution",
                ("Inter", 30, &RGBColor(208, 208, 208)),
            )
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(
                (min_val..max_val).step((max_val - min_val) / num_bins as f64),
                0..1, 
            )?;

        let hist = Histogram::vertical(&chart)
            .style(&GREEN.mix(0.5))
            .data(data.iter().map(|&x| (x, 1)));

        chart.draw_series(hist)?;

        chart
            .configure_mesh()
            .axis_style(&RGBColor(208, 208, 208))
            .label_style(("Inter", 15, &RGBColor(208, 208, 208)))
            .draw()?;
    }

    Ok((buf, CHART_WIDTH, CHART_HEIGHT))
}

#[cfg(test)]
mod tests {
    use crate::{SimParams, core_sim::run_simulation};

    use super::*;

    #[test]
    fn test_gbm_reproducibility() {
        let params = SimParams {
            initial_price: 100.0,
            horizon: 30,
            num_paths: 10,
            mu: 0.0002,
            sigma: 0.015,
            seed: 12345,
            use_antithetic: false,
            dt: 1,
            model_type: "GBM".to_string().into(),
        };
        
        let result1 = run_simulation(params.clone(), vec![]).unwrap();
        let result2 = run_simulation(params, vec![]).unwrap();
        
        assert_eq!(result1.0.mean, result2.0.mean);
    }
}