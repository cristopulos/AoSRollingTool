use eframe::egui;

use crate::combat::simulation::SimulationResult;
use crate::combat::types::{CombatResult, Phase};
use crate::ui::widgets::histogram::HistogramDisplay;
use crate::ui::widgets::phase_result::PhaseResultCard;

pub struct CombatView<'a> {
    result: &'a CombatResult,
    simulation: Option<&'a SimulationResult>,
}

impl<'a> CombatView<'a> {
    pub fn new(result: &'a CombatResult, simulation: Option<&'a SimulationResult>) -> Self {
        Self { result, simulation }
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        ui.heading("Combat Sequence");
        ui.separator();

        ui.label(format!(
            "{} attacking {} with {}",
            self.result.attacker_name, self.result.defender_name, self.result.weapon_name
        ));

        ui.separator();

        // Prominent final result card at the top
        self.show_result_summary(ui);

        ui.separator();

        egui::Grid::new("combat_phases")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .show(ui, |ui| {
                for phase in &self.result.phases {
                    let sim = match phase.phase {
                        Phase::Hit => self.simulation.map(|s| &s.hits_stats),
                        Phase::Wound => self.simulation.map(|s| &s.wounds_stats),
                        Phase::Damage => self.simulation.map(|s| &s.damage_stats),
                        _ => None,
                    };
                    PhaseResultCard::new(phase, sim).show(ui);
                    ui.end_row();
                }
            });

        if let Some(sim) = self.simulation {
            ui.separator();
            ui.heading("Percentile Analysis");

            let dmg = &sim.damage_stats;
            ui.label(format!(
                "Your damage of {} is at the {:.0}th percentile",
                dmg.actual_value,
                dmg.percentile * 100.0
            ));

            ui.horizontal_wrapped(|ui| {
                let stat = |ui: &mut egui::Ui, label: &str, value: String, strong: bool| {
                    ui.vertical(|ui| {
                        ui.set_min_width(70.0);
                        ui.label(egui::RichText::new(label).size(11.0).weak());
                        if strong {
                            ui.strong(egui::RichText::new(value).size(14.0));
                        } else {
                            ui.label(egui::RichText::new(value).size(14.0));
                        }
                    });
                };

                stat(ui, "Mean", format!("{:.1}", dmg.percentiles.mean), false);
                stat(ui, "10th", format!("{}", dmg.percentiles.p10), false);
                stat(ui, "25th", format!("{}", dmg.percentiles.p25), false);
                stat(ui, "Median", format!("{}", dmg.percentiles.p50), true);
                stat(ui, "75th", format!("{}", dmg.percentiles.p75), false);
                stat(ui, "90th", format!("{}", dmg.percentiles.p90), false);
            });

            ui.separator();
            HistogramDisplay::new(
                &sim.histogram_bins,
                dmg.actual_value,
                "Damage Distribution",
                Some(dmg.percentiles.p25),
                Some(dmg.percentiles.p75),
            )
            .show(ui);
        }
    }

    /// Renders the prominent final damage card displayed at the top of the combat view.
    /// The damage value is colored based on its percentile ranking from the simulation:
    /// green (≥90th), light green (≥75th), yellow (≥50th), orange (≥25th), or red (<25th).
    fn show_result_summary(&self, ui: &mut egui::Ui) {
        if self.result.stopped_after_wound {
            self.show_stopped_summary(ui);
            return;
        }

        let damage_color = self
            .simulation
            .map(|sim| percentile_color(sim.damage_stats.percentile))
            .unwrap_or(egui::Color32::WHITE);

        egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::same(16.0))
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("Final Damage Dealt")
                            .size(14.0)
                            .weak(),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(format!("{}", self.result.final_damage))
                            .size(40.0)
                            .strong()
                            .color(damage_color),
                    );
                });
            });

        if self.result.mortal_wounds > 0 {
            ui.horizontal(|ui| {
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(format!(
                        "includes {} mortal wounds",
                        self.result.mortal_wounds
                    ))
                    .weak()
                    .size(12.0),
                );
            });
        }
    }

    fn show_stopped_summary(&self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::same(12.0))
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("Sequence Stopped")
                            .size(14.0)
                            .weak(),
                    );
                    ui.label(
                        egui::RichText::new(format!(
                            "{} hits / {} wounds to save",
                            self.result.total_hits, self.result.total_wounds
                        ))
                        .size(20.0)
                        .strong(),
                    );
                    if self.result.mortal_wounds > 0 {
                        ui.label(
                            egui::RichText::new(format!(
                                "(includes {} mortal wounds)",
                                self.result.mortal_wounds
                            ))
                            .size(12.0)
                            .weak(),
                        );
                    }
                    ui.label(
                        egui::RichText::new(
                            "Save, Damage, and Ward phases are pending — roll externally.",
                        )
                        .size(11.0)
                        .weak(),
                    );
                });
            });
    }
}

/// Map a percentile (0.0–1.0) to a color indicating roll quality.
fn percentile_color(pct: f64) -> egui::Color32 {
    if pct >= 0.90 {
        egui::Color32::from_rgb(100, 220, 100) // Green — great roll
    } else if pct >= 0.75 {
        egui::Color32::from_rgb(180, 220, 100) // Light green — good
    } else if pct >= 0.50 {
        egui::Color32::from_rgb(220, 220, 100) // Yellow — average
    } else if pct >= 0.25 {
        egui::Color32::from_rgb(220, 180, 100) // Orange — below average
    } else {
        egui::Color32::from_rgb(220, 100, 100) // Red — poor roll
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percentile_color_green_high() {
        assert_eq!(percentile_color(0.95), egui::Color32::from_rgb(100, 220, 100));
    }

    #[test]
    fn test_percentile_color_light_green() {
        assert_eq!(percentile_color(0.80), egui::Color32::from_rgb(180, 220, 100));
    }

    #[test]
    fn test_percentile_color_yellow() {
        assert_eq!(percentile_color(0.60), egui::Color32::from_rgb(220, 220, 100));
    }

    #[test]
    fn test_percentile_color_orange() {
        assert_eq!(percentile_color(0.40), egui::Color32::from_rgb(220, 180, 100));
    }

    #[test]
    fn test_percentile_color_red_low() {
        assert_eq!(percentile_color(0.10), egui::Color32::from_rgb(220, 100, 100));
    }

    #[test]
    fn test_percentile_color_boundary_50() {
        // 0.50 exactly should be yellow (>= 0.50)
        assert_eq!(percentile_color(0.50), egui::Color32::from_rgb(220, 220, 100));
    }

    #[test]
    fn test_percentile_color_boundary_25() {
        // 0.25 exactly should be orange (>= 0.25)
        assert_eq!(percentile_color(0.25), egui::Color32::from_rgb(220, 180, 100));
    }

    #[test]
    fn test_percentile_color_boundary_90() {
        // 0.90 exactly should be green (>= 0.90)
        assert_eq!(percentile_color(0.90), egui::Color32::from_rgb(100, 220, 100));
    }

    #[test]
    fn test_percentile_color_boundary_75() {
        // 0.75 exactly should be light green (>= 0.75)
        assert_eq!(percentile_color(0.75), egui::Color32::from_rgb(180, 220, 100));
    }

    #[test]
    fn test_percentile_color_zero() {
        assert_eq!(percentile_color(0.0), egui::Color32::from_rgb(220, 100, 100));
    }

    #[test]
    fn test_percentile_color_one() {
        assert_eq!(percentile_color(1.0), egui::Color32::from_rgb(100, 220, 100));
    }
}
