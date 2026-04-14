use eframe::egui;

use crate::combat::types::{PhaseResult, VarianceStep};
use crate::ui::widgets::dice_display::DiceDisplay;

pub struct PhaseResultCard<'a> {
    phase: &'a PhaseResult,
}

impl<'a> PhaseResultCard<'a> {
    pub fn new(phase: &'a PhaseResult) -> Self {
        Self { phase }
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
                    VarianceStep::AttackRoll { per_model, results, total } => {
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
                    VarianceStep::DamageRoll { per_wound, results, total } => {
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
            } else {
                ui.horizontal(|ui| {
                    DiceDisplay::new(&self.phase.rolls).show(ui);
                    ui.label(format!(
                        "→ {} success, {} fail = {}",
                        self.phase.successes, self.phase.failures, self.phase.total_output
                    ));
                });
            }
        });
    }
}
