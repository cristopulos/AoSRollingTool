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
        ui.heading("Attackers");
        ui.separator();

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
                    let mut selected = self.app.selected_attackers.contains(&unit.id);
                    if ui.checkbox(&mut selected, &unit.name).changed() {
                        if selected {
                            if !self.app.selected_attackers.contains(&unit.id) {
                                self.app.selected_attackers.push(unit.id.clone());
                            }
                            // Auto-select first weapon if none selected
                            if self.app.selected_weapon.is_empty() && !unit.weapons.is_empty() {
                                self.app.selected_weapon = unit.weapons[0].name.clone();
                            }
                        } else {
                            self.app.selected_attackers.retain(|id| id != &unit.id);
                            self.app.selected_weapon.clear();
                        }
                    }
                }
            });
        }

        if !self.app.selected_attackers.is_empty() {
            ui.separator();
            ui.heading("Weapon");

            // Collect weapons from selected attackers
            let selected_units: Vec<_> = self
                .app
                .units
                .iter()
                .filter(|u| self.app.selected_attackers.contains(&u.id))
                .collect();

            for unit in selected_units {
                ui.label(unit.name.to_string());
                for weapon in &unit.weapons {
                    ui.horizontal(|ui| {
                        ui.radio_value(
                            &mut self.app.selected_weapon,
                            weapon.name.clone(),
                            format!(
                                "{} (A:{}, Hit:{}+",
                                weapon.name, weapon.attacks, weapon.to_hit
                            ),
                        );
                    });
                }
            }
        }
    }
}
