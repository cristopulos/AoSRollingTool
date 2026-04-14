use eframe::egui;

use crate::combat::types::DiceRoll;

const SCROLL_THRESHOLD: usize = 30;

pub struct DiceDisplay<'a> {
    rolls: &'a [DiceRoll],
}

impl<'a> DiceDisplay<'a> {
    pub fn new(rolls: &'a [DiceRoll]) -> Self {
        Self { rolls }
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        if self.rolls.is_empty() {
            ui.label("—");
            return;
        }

        let show_dice = |ui: &mut egui::Ui| {
            for roll in self.rolls {
                let (color, label) = if roll.is_crit {
                    (egui::Color32::GOLD, format!("{}", roll.value))
                } else if roll.success {
                    (egui::Color32::GREEN, format!("{}", roll.value))
                } else {
                    (egui::Color32::RED, format!("{}", roll.value))
                };

                ui.colored_label(
                    color,
                    egui::RichText::new(label).monospace().size(16.0),
                );
            }
        };

        if self.rolls.len() > SCROLL_THRESHOLD {
            egui::ScrollArea::horizontal()
                .max_height(40.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        show_dice(ui);
                    });
                });
        } else {
            ui.horizontal_wrapped(|ui| {
                show_dice(ui);
            });
        }
    }
}
