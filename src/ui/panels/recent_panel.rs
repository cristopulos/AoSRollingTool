use eframe::egui;

use crate::app::AoSApp;

pub struct RecentPanel<'a> {
    app: &'a mut AoSApp,
}

impl<'a> RecentPanel<'a> {
    pub fn new(app: &'a mut AoSApp) -> Self {
        Self { app }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("Recently Used");
        ui.separator();

        // --- Attackers ---
        ui.label("Attackers:");
        if self.app.recent_attackers.is_empty() {
            ui.label(egui::RichText::new("None yet").weak());
        } else {
            egui::ScrollArea::vertical()
                .id_salt("recent_attackers")
                .max_height(300.0)
                .show(ui, |ui| {
                    for (id, name) in self.app.recent_attackers.iter() {
                        let selected = self.app.selected_attackers.contains(id);
                        if ui.radio(selected, name).clicked() {
                            self.app.selected_attackers.clear();
                            self.app.selected_attackers.push(id.clone());
                            // Auto-select first weapon
                            if let Some(unit) = self.app.units.iter().find(|u| &u.id == id) {
                                if !unit.weapons.is_empty() {
                                    self.app.selected_weapon = unit.weapons[0].name.clone();
                                }
                            }
                        }
                    }
                });
        }

        ui.separator();

        // --- Defenders ---
        ui.label("Defenders:");
        if self.app.recent_defenders.is_empty() {
            ui.label(egui::RichText::new("None yet").weak());
        } else {
            egui::ScrollArea::vertical()
                .id_salt("recent_defenders")
                .max_height(300.0)
                .show(ui, |ui| {
                    for (id, name) in self.app.recent_defenders.iter() {
                        let selected = self.app.selected_defender == *id;
                        if ui.radio(selected, name).clicked() {
                            self.app.selected_defender = id.clone();
                        }
                    }
                });
        }
    }
}
