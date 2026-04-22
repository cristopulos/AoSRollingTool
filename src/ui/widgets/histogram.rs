//! Histogram visualization widget using `egui_plot::BarChart`.
//!
//! Uses `egui_plot`'s `BarChart` for proper axis scaling, grid lines, and zoom support
//! rather than custom painter primitives. This provides better UX with large datasets
//! common in Monte Carlo simulations.

use eframe::egui;
use egui_plot::{Bar, BarChart, Legend, Plot, VLine};

use crate::combat::simulation::HistogramBin;

pub struct HistogramDisplay<'a> {
    bins: &'a [HistogramBin],
    actual_value: usize,
    title: &'a str,
}

impl<'a> HistogramDisplay<'a> {
    pub fn new(bins: &'a [HistogramBin], actual_value: usize, title: &'a str) -> Self {
        Self {
            bins,
            actual_value,
            title,
        }
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        ui.heading(self.title);

        if self.bins.is_empty() {
            ui.label("No data available for histogram.");
            return;
        }

        let bsize = bin_size(self.bins).max(1) as f64;
        let actual_f = self.actual_value as f64;

        let bars: Vec<Bar> = self
            .bins
            .iter()
            .map(|bin| {
                let center = bin.value as f64 + bsize / 2.0;
                let is_actual =
                    self.actual_value >= bin.value && self.actual_value < bin.value + bsize as usize;
                let color = if is_actual {
                    egui::Color32::from_rgb(255, 100, 100)
                } else {
                    egui::Color32::from_rgb(100, 150, 255)
                };
                Bar::new(center, bin.count as f64)
                    .width(bsize)
                    .fill(color)
            })
            .collect();

        let chart = BarChart::new(bars).name("Simulations");

        Plot::new("damage_histogram")
            .legend(Legend::default())
            .x_axis_label("Damage")
            .y_axis_label("Simulations")
            .show(ui, |plot_ui| {
                plot_ui.bar_chart(chart);
                plot_ui.vline(VLine::new(actual_f).name("Your roll"));
            });

        ui.horizontal(|ui| {
            ui.colored_label(egui::Color32::from_rgb(255, 100, 100), "Your roll");
            ui.colored_label(egui::Color32::from_rgb(100, 150, 255), "Expected range");
        });
    }
}

fn bin_size(bins: &[HistogramBin]) -> usize {
    if bins.len() < 2 {
        return 1;
    }
    let first = bins[0].value;
    let second = bins[1].value;
    second.saturating_sub(first)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_bin(value: usize) -> HistogramBin {
        HistogramBin {
            value,
            count: 1,
            percentage: 0.0,
        }
    }

    #[test]
    fn test_bin_size_empty() {
        let bins: [HistogramBin; 0] = [];
        assert_eq!(bin_size(&bins), 1);
    }

    #[test]
    fn test_bin_size_single() {
        let bins = [make_bin(10)];
        assert_eq!(bin_size(&bins), 1);
    }

    #[test]
    fn test_bin_size_two_bins_diff_1() {
        let bins = [make_bin(10), make_bin(11)];
        assert_eq!(bin_size(&bins), 1);
    }

    #[test]
    fn test_bin_size_two_bins_diff_5() {
        let bins = [make_bin(10), make_bin(15)];
        assert_eq!(bin_size(&bins), 5);
    }
}
