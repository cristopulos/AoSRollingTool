use eframe::egui;

use crate::combat::types::CombatResult;

pub struct LogPanel<'a> {
    log: &'a [CombatResult],
}

impl<'a> LogPanel<'a> {
    pub fn new(log: &'a [CombatResult]) -> Self {
        Self { log }
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        ui.heading("Combat Log");
        ui.separator();

        egui::ScrollArea::vertical()
            .max_height(150.0)
            .show(ui, |ui| {
                for (i, entry) in self.log.iter().enumerate().rev() {
                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "#{}: {} → {}: {} damage",
                            i + 1,
                            entry.attacker_name,
                            entry.defender_name,
                            entry.final_damage
                        ));
                    });
                }
            });
    }
}
