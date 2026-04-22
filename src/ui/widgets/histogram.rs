//! Histogram visualization widget using `egui_plot::BarChart`.
//!
//! Uses `egui_plot`'s `BarChart` for proper axis scaling, grid lines, and zoom support.
//! Each unique damage value gets its own bar with a count label above it. Vertical
//! lines mark the actual roll and optional 25th/75th percentile boundaries.

use eframe::egui;
use egui_plot::{Bar, BarChart, Legend, Plot, PlotPoint, Text, VLine};

use crate::combat::simulation::HistogramBin;

pub struct HistogramDisplay<'a> {
    bins: &'a [HistogramBin],
    actual_value: usize,
    title: &'a str,
    p25: Option<usize>,
    p75: Option<usize>,
}

impl<'a> HistogramDisplay<'a> {
    pub fn new(
        bins: &'a [HistogramBin],
        actual_value: usize,
        title: &'a str,
        p25: Option<usize>,
        p75: Option<usize>,
    ) -> Self {
        Self {
            bins,
            actual_value,
            title,
            p25,
            p75,
        }
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        ui.heading(self.title);

        if self.bins.is_empty() {
            ui.label("No data available for histogram.");
            return;
        }

        let total: usize = self.bins.iter().map(|b| b.count).sum();
        if total == 0 {
            ui.label("No simulation data to display.");
            return;
        }
        let total_f = total as f64;

        let bsize = bin_size(self.bins).max(1) as f64;
        let actual_f = self.actual_value as f64;
        let label_offset = 3.0_f64.max(2.0);
        let text_color = ui.visuals().text_color();

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
                let pct = (bin.count as f64 / total_f) * 100.0;
                Bar::new(center, pct)
                    .width(bsize)
                    .fill(color)
            })
            .collect();

        let chart = BarChart::new(bars).name("Simulations");

        let plot_height = 400.0;
        ui.allocate_ui(egui::vec2(ui.available_width(), plot_height), |ui| {
            Plot::new("damage_histogram")
                .legend(Legend::default())
                .x_axis_label("Damage")
                .y_axis_label("Probability")
                .auto_bounds(egui::Vec2b::new(true, true))
                .include_y(0.0)
                .allow_drag(false)
                .allow_zoom(false)
                .allow_scroll(false)
                .show(ui, |plot_ui| {
                    plot_ui.bar_chart(chart);

                    // Percentage labels above each bar
                    for bin in self.bins {
                        let center = bin.value as f64 + bsize / 2.0;
                        let pct = (bin.count as f64 / total_f) * 100.0;
                        let label_pos = PlotPoint::new(center, pct + label_offset);
                        plot_ui.text(
                            Text::new(label_pos, format!("{:.1}%", pct))
                                .anchor(egui::Align2::CENTER_BOTTOM)
                                .color(text_color),
                        );
                    }

                    // Vertical lines
                    plot_ui.vline(
                        VLine::new(actual_f)
                            .name("Your roll")
                            .color(egui::Color32::from_rgb(255, 100, 100))
                            .width(2.0),
                    );
                    if let Some(p25) = self.p25 {
                        plot_ui.vline(
                            VLine::new(p25 as f64)
                                .name("25th percentile")
                                .color(egui::Color32::from_rgb(255, 160, 0))
                                .width(2.0),
                        );
                    }
                    if let Some(p75) = self.p75 {
                        plot_ui.vline(
                            VLine::new(p75 as f64)
                                .name("75th percentile")
                                .color(egui::Color32::from_rgb(80, 220, 220))
                                .width(2.0),
                        );
                    }
                });
        });

        ui.horizontal(|ui| {
            ui.colored_label(
                egui::Color32::from_rgb(255, 100, 100),
                "Your roll",
            );
            ui.colored_label(egui::Color32::from_rgb(100, 150, 255), "Expected range");
            ui.colored_label(egui::Color32::from_rgb(255, 160, 0), "25th percentile");
            ui.colored_label(egui::Color32::from_rgb(80, 220, 220), "75th percentile");
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
