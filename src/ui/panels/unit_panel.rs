use eframe::egui;

use crate::app::AoSApp;

pub struct UnitPanel<'a> {
    app: &'a mut AoSApp,
}

impl<'a> UnitPanel<'a> {
    pub fn new(app: &'a mut AoSApp) -> Self {
        Self { app }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("Attacker");
        ui.separator();

        ui.push_id("attacker_panel", |ui| {
            // Group units by faction
            let mut factions: Vec<String> = self
                .app
                .units
                .iter()
                .map(|u| u.faction.clone())
                .collect();
            factions.sort();
            factions.dedup();

            for faction in factions {
                ui.collapsing(faction.clone(), |ui| {
                    for unit in self.app.units.iter().filter(|u| u.faction == faction) {
                        let selected = self.app.selected_attackers.contains(&unit.id);
                        if ui.radio(selected, &unit.name).clicked() && !selected {
                            // Select this unit (single-select)
                            self.app.selected_attackers.clear();
                            self.app.selected_attackers.push(unit.id.clone());
                            // Auto-select first weapon
                            if !unit.weapons.is_empty() {
                                self.app.selected_weapon = unit.weapons[0].name.clone();
                            } else {
                                self.app.selected_weapon.clear();
                            }
                        }
                    }
                });
            }

            if !self.app.selected_attackers.is_empty() {
                ui.separator();
                ui.heading("Weapon");

                // Show weapons for selected attacker
                if let Some(selected_unit) = self
                    .app
                    .units
                    .iter()
                    .find(|u| self.app.selected_attackers.contains(&u.id))
                {
                    ui.label(selected_unit.name.to_string());
                    ui.push_id("weapon_list", |ui| {
                        for weapon in &selected_unit.weapons {
                            ui.horizontal(|ui| {
                                ui.radio_value(
                                    &mut self.app.selected_weapon,
                                    weapon.name.clone(),
                                    format!(
                                        "{} (A:{}, Hit:{}+)",
                                        weapon.name, weapon.attacks, weapon.to_hit
                                    ),
                                );
                            });
                        }
                    });
                }
            }
        });
    }
}
