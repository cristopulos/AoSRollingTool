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

        if self.app.recent_units.is_empty() {
            ui.label(egui::RichText::new("No recent units yet").weak());
        } else {
            egui::ScrollArea::vertical()
                .id_salt("recent_units")
                .max_height(600.0)
                .show(ui, |ui| {
                    for (id, name) in self.app.recent_units.iter() {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(name).strong());
                            });
                            ui.horizontal(|ui| {
                                // Attacker role radio
                                let is_attacker = self.app.selected_attackers.contains(id);
                                if ui.radio(is_attacker, "Attacker").clicked() {
                                    self.app.selected_attackers.clear();
                                    self.app.selected_attackers.push(id.clone());
                                    // Prevent dual-role: clear defender if same unit
                                    if self.app.selected_defender == *id {
                                        self.app.selected_defender.clear();
                                    }
                                    // Auto-select first weapon
                                    if let Some(unit) =
                                        self.app.units.iter().find(|u| &u.id == id)
                                    {
                                        if !unit.weapons.is_empty() {
                                            self.app.selected_weapon_index = Some(0);
                                        } else {
                                            self.app.selected_weapon_index = None;
                                        }
                                    }
                                }

                                // Defender role radio
                                let is_defender = self.app.selected_defender == *id;
                                if ui.radio(is_defender, "Defender").clicked() {
                                    self.app.selected_defender = id.clone();
                                    // Prevent dual-role: clear attacker if same unit
                                    self.app.selected_attackers.retain(|a| a != id);
                                }
                            });
                        });
                    }
                });
        }
    }
}
