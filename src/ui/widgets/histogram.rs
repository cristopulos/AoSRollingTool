use eframe::egui;

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

        let max_count = self.bins.iter().map(|b| b.count).max().unwrap_or(1) as f64;
        let bsize = bin_size(self.bins).max(1) as f64;
        let bar_width = 24.0f32;
        let chart_height = 150.0f32;
        let spacing = 4.0f32;
        let total_width = self.bins.len() as f32 * (bar_width + spacing);

        ui.horizontal(|ui| {
            // Y-axis labels
            ui.vertical(|ui| {
                ui.set_min_width(30.0);
                ui.add_space(4.0);
                ui.label("100%");
                ui.add_space(chart_height / 2.0 - 16.0);
                ui.label("50%");
                ui.add_space(chart_height / 2.0 - 16.0);
                ui.label("0%");
            });

            ui.vertical(|ui| {
                let (rect, _response) = ui.allocate_exact_size(
                    egui::vec2(total_width, chart_height),
                    egui::Sense::hover(),
                );
                let painter = ui.painter_at(rect);

                for (i, bin) in self.bins.iter().enumerate() {
                    let height = (bin.count as f64 / max_count) as f32 * chart_height;
                    let x = rect.min.x + i as f32 * (bar_width + spacing);
                    let y = rect.max.y - height;

                    let is_actual = self.actual_value >= bin.value
                        && self.actual_value < bin.value + bsize as usize;
                    let color = if is_actual {
                        egui::Color32::from_rgb(255, 100, 100)
                    } else {
                        egui::Color32::from_rgb(100, 150, 255)
                    };

                    let bar_rect =
                        egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(bar_width, height));
                    painter.rect_filled(bar_rect, 2.0, color);

                    // X-axis label
                    let label = if bsize > 1.0 {
                        format!("{}-{}", bin.value, bin.value + bsize as usize - 1)
                    } else {
                        bin.value.to_string()
                    };
                    painter.text(
                        egui::pos2(x + bar_width / 2.0, rect.max.y + 14.0),
                        egui::Align2::CENTER_TOP,
                        label,
                        egui::FontId::proportional(10.0),
                        ui.visuals().text_color(),
                    );
                }

                // Need extra space for labels below
                ui.allocate_space(egui::vec2(total_width, 20.0));
            });
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
