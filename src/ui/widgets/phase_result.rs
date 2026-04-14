use eframe::egui;

use crate::combat::types::PhaseResult;
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

            if self.phase.skipped {
                ui.label("Skipped");
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
