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

        ui.separator();

        if self.result.stopped_after_wound {
            ui.heading("Sequence Stopped - Defender Rolls Saves");
            ui.horizontal(|ui| {
                ui.label("Hits determined:");
                ui.strong(format!("{}", self.result.total_hits));
            });
            ui.horizontal(|ui| {
                ui.label("Wounds to save:");
                ui.strong(format!("{}", self.result.total_wounds));
            });
            if self.result.mortal_wounds > 0 {
                ui.label(format!(
                    "(includes {} mortal wounds from critical hits)",
                    self.result.mortal_wounds
                ));
            }
            ui.label("Save, Damage, and Ward phases are pending — roll these externally.");
        } else {
            ui.horizontal(|ui| {
                ui.label("FINAL DAMAGE:");
                ui.strong(format!("{}", self.result.final_damage));
            });

            if self.result.mortal_wounds > 0 {
                ui.label(format!(
                    "(includes {} mortal wounds)",
                    self.result.mortal_wounds
                ));
            }
        }

        if let Some(sim) = self.simulation {
            ui.separator();
            ui.heading("Percentile Analysis");

            let dmg = &sim.damage_stats;
            ui.label(format!(
                "Your damage of {} is at the {:.0}th percentile",
                dmg.actual_value,
                dmg.percentile * 100.0
            ));

            egui::Grid::new("sim_stats")
                .num_columns(2)
                .spacing([20.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Mean:");
                    ui.label(format!("{:.1}", dmg.percentiles.mean));
                    ui.end_row();

                    ui.label("10th percentile:");
                    ui.label(format!("{}", dmg.percentiles.p10));
                    ui.end_row();

                    ui.label("25th percentile:");
                    ui.label(format!("{}", dmg.percentiles.p25));
                    ui.end_row();

                    ui.label("Median (50th):");
                    ui.strong(format!("{}", dmg.percentiles.p50));
                    ui.end_row();

                    ui.label("75th percentile:");
                    ui.label(format!("{}", dmg.percentiles.p75));
                    ui.end_row();

                    ui.label("90th percentile:");
                    ui.label(format!("{}", dmg.percentiles.p90));
                    ui.end_row();
                });

            ui.separator();
            HistogramDisplay::new(
                &sim.histogram_bins,
                dmg.actual_value,
                "Damage Distribution",
            )
            .show(ui);
        }
    }
}
