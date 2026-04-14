use eframe::egui;

use crate::app::AoSApp;
use crate::data::models::CritEffect;

fn format_weapon_stats(weapon: &crate::data::models::Weapon) -> String {
    let crit_str = match &weapon.crit_hit {
        Some(CritEffect::AutoWound) => "AutoWnd".to_string(),
        Some(CritEffect::ExtraHit) => "ExtraHit".to_string(),
        Some(CritEffect::MortalWounds(v)) => format!("MW({})", v),
        None => "—".to_string(),
    };
    format!(
        "A:{} Hit:{}+ Wnd:{}+ R:{} D:{} Crit:{}",
        weapon.attack,
        weapon.to_hit,
        weapon.to_wound,
        weapon.rend,
        weapon.damage,
        crit_str
    )
}

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
                                    format!("{} {}", weapon.name, format_weapon_stats(weapon)),
                                );
                            });
                        }
                    });
                }

                // Champion checkbox
                ui.separator();
                ui.checkbox(&mut self.app.has_champion, "Champion (+1 attack)");

                // Attack override toggle
                ui.checkbox(
                    &mut self.app.use_attack_override,
                    "Override total attacks",
                );

                if self.app.use_attack_override {
                    ui.horizontal(|ui| {
                        ui.label("Attacks:");
                        ui.add(
                            egui::DragValue::new(&mut self.app.attack_override)
                                .range(1..=200)
                                .clamp_existing_to_range(true),
                        );
                    });
                    ui.label(format!(
                        "(Ignores models × attack, uses {} attacks directly)",
                        self.app.attack_override
                    ));
                } else {
                    // Model count input (only when not overriding)
                    ui.horizontal(|ui| {
                        ui.label("Models:");
                        ui.add(
                            egui::DragValue::new(&mut self.app.num_models)
                                .range(1..=100)
                                .clamp_existing_to_range(true),
                        );
                    });
                }

                ui.checkbox(
                    &mut self.app.stop_after_wound,
                    "Stop after wound (defender rolls saves)",
                );
            }
        });
    }
}
