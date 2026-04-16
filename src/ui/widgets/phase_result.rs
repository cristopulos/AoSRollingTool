use eframe::egui;

use crate::combat::simulation::PhaseSimulation;
use crate::combat::types::{Phase, PhaseResult, VarianceStep};
use crate::ui::widgets::dice_display::DiceDisplay;

pub struct PhaseResultCard<'a> {
    phase: &'a PhaseResult,
    simulation: Option<&'a PhaseSimulation>,
}

impl<'a> PhaseResultCard<'a> {
    pub fn new(phase: &'a PhaseResult, simulation: Option<&'a PhaseSimulation>) -> Self {
        Self { phase, simulation }
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.label(
                egui::RichText::new(&self.phase.description)
                    .strong()
                    .size(14.0),
            );

            if let Some(ref variance) = self.phase.variance_step {
                match variance {
                    VarianceStep::AttackRoll {
                        per_model,
                        results,
                        total,
                    } => {
                        let results_str = results
                            .iter()
                            .map(|r| r.to_string())
                            .collect::<Vec<_>>()
                            .join(", ");
                        ui.label(format!(
                            "Rolling {} per model: [{}] = {} total attacks",
                            per_model, results_str, total
                        ));
                    }
                    VarianceStep::DamageRoll {
                        per_wound,
                        results,
                        total,
                    } => {
                        let results_str = results
                            .iter()
                            .map(|r| r.to_string())
                            .collect::<Vec<_>>()
                            .join(", ");
                        ui.label(format!(
                            "Rolling {} per wound: [{}] = {} damage",
                            per_wound, results_str, total
                        ));
                    }
                }
            }

            if self.phase.skipped {
                ui.label(
                    egui::RichText::new("Pending — roll externally")
                        .weak()
                        .italics(),
                );
            } else if self.phase.auto_fails {
                ui.label(format!(
                    "Auto-fail (target {}+) → {} through",
                    self.phase.successes + self.phase.failures,
                    self.phase.total_output
                ));
            } else if self.phase.crit_extra_count > 0 {
                // Wound phase successes already exclude auto-wounds (they bypass the roll),
                // so we must not subtract crit_extra_count there.
                let normal = match self.phase.phase {
                    Phase::Wound => self.phase.successes,
                    _ => self
                        .phase
                        .successes
                        .saturating_sub(self.phase.crit_extra_count),
                };
                ui.horizontal(|ui| {
                    DiceDisplay::new(&self.phase.rolls).show(ui);
                    let label = match self.phase.phase {
                        Phase::Hit => format!(
                            "→ {} base + {} extra = {} ({} fail)",
                            normal,
                            self.phase.crit_extra_count,
                            self.phase.total_output,
                            self.phase.failures
                        ),
                        Phase::Wound => format!(
                            "→ {} normal + {} extra = {} ({} fail)",
                            normal,
                            self.phase.crit_extra_count,
                            self.phase.total_output,
                            self.phase.failures
                        ),
                        Phase::Damage => format!(
                            "→ {} normal + {} MW = {} ({} fail)",
                            normal,
                            self.phase.crit_extra_count,
                            self.phase.total_output,
                            self.phase.failures
                        ),
                        _ => format!(
                            "→ {} success, {} fail = {}",
                            self.phase.successes, self.phase.failures, self.phase.total_output
                        ),
                    };
                    ui.label(label);
                });
            } else {
                ui.horizontal(|ui| {
                    DiceDisplay::new(&self.phase.rolls).show(ui);
                    ui.label(format!(
                        "→ {} success, {} fail = {}",
                        self.phase.successes, self.phase.failures, self.phase.total_output
                    ));
                });
            }

            if let Some(sim) = self.simulation {
                let color = if sim.percentile >= 0.90 {
                    egui::Color32::from_rgb(100, 200, 100) // Green - great roll
                } else if sim.percentile >= 0.50 {
                    egui::Color32::from_rgb(200, 200, 100) // Yellow - average
                } else {
                    egui::Color32::from_rgb(200, 100, 100) // Red - below average
                };
                ui.label(
                    egui::RichText::new(format!(
                        "{:.0}th percentile (expected {}–{})",
                        sim.percentile * 100.0,
                        sim.percentiles.p25,
                        sim.percentiles.p75
                    ))
                    .color(color)
                    .size(12.0),
                );
            }
        });
    }
}
