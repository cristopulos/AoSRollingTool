use eframe::egui;

use crate::app::AoSApp;
use crate::data::models::CritEffect;

fn format_weapon_stats(
    weapon: &crate::data::models::Weapon,
    crit_override: &Option<CritEffect>,
) -> String {
    let crit_str = if let Some(ref crit) = crit_override {
        match crit {
            CritEffect::AutoWound => "[Override] AutoWnd".to_string(),
            CritEffect::ExtraHit => "[Override] ExtraHit".to_string(),
            CritEffect::MortalWounds(Some(v)) => format!("[Override] MW({})", v),
            CritEffect::MortalWounds(None) => "[Override] MW".to_string(),
        }
    } else {
        match &weapon.crit_hit {
            Some(CritEffect::AutoWound) => "AutoWnd".to_string(),
            Some(CritEffect::ExtraHit) => "ExtraHit".to_string(),
            Some(CritEffect::MortalWounds(Some(v))) => format!("MW({})", v),
            Some(CritEffect::MortalWounds(None)) => "MW".to_string(),
            None => "—".to_string(),
        }
    };
    format!(
        "A:{} Hit:{}+ Wnd:{}+ R:{} D:{} Crit:{}",
        weapon.attack, weapon.to_hit, weapon.to_wound, weapon.rend, weapon.damage, crit_str
    )
}

fn crit_effect_label(crit: &Option<CritEffect>) -> String {
    match crit {
        None => "Default (use weapon)".to_string(),
        Some(CritEffect::AutoWound) => "Auto Wound".to_string(),
        Some(CritEffect::ExtraHit) => "Extra Hit".to_string(),
        Some(CritEffect::MortalWounds(v)) => {
            format!(
                "Mortal Wounds{}",
                v.as_ref().map(|s| format!(" ({})", s)).unwrap_or_default()
            )
        }
    }
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
        // Search field filters units by name or faction
        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(&mut self.app.attacker_search);
        });
        ui.separator();

        ui.push_id("attacker_panel", |ui| {
            // Apply search filter to both unit names and faction names
            let query = self.app.attacker_search.to_lowercase();
            let filtered_units: Vec<_> = self
                .app
                .units
                .iter()
                .filter(|u| {
                    let name = u.name.to_lowercase();
                    let faction = u.faction.to_lowercase();
                    name.contains(&query) || faction.contains(&query)
                })
                .collect();

            // Group filtered units by faction
            let mut factions: Vec<String> =
                filtered_units.iter().map(|u| u.faction.clone()).collect();
            factions.sort();
            factions.dedup();

            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), self.app.attacker_panel_height),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("attacker_list")
                        .auto_shrink([false; 2])
                        .hscroll(false)
                        .show(ui, |ui| {
                            for faction in factions {
                                egui::CollapsingHeader::new(faction.clone())
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        for unit in
                                            filtered_units.iter().filter(|u| u.faction == faction)
                                        {
                                            let selected =
                                                self.app.selected_attackers.contains(&unit.id);
                                            if ui.radio(selected, &unit.name).clicked() && !selected
                                            {
                                                // Select this unit (single-select)
                                                self.app.selected_attackers.clear();
                                                self.app.selected_attackers.push(unit.id.clone());
                                                // Auto-select first weapon
                                                if !unit.weapons.is_empty() {
                                                    self.app.selected_weapon =
                                                        unit.weapons[0].name.clone();
                                                } else {
                                                    self.app.selected_weapon.clear();
                                                }
                                            }
                                        }
                                    });
                            }
                        });
                },
            );

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
                    ui.push_id(("weapon_list", selected_unit.id.clone()), |ui| {
                        for weapon in &selected_unit.weapons {
                            ui.horizontal(|ui| {
                                ui.radio_value(
                                    &mut self.app.selected_weapon,
                                    weapon.name.clone(),
                                    format!(
                                        "{} {}",
                                        weapon.name,
                                        format_weapon_stats(weapon, &self.app.crit_effect_override)
                                    ),
                                );
                            });
                        }
                    });
                }

                // Reset crit effect override when the selected weapon changes.
                // This prevents stale overrides from applying to a different weapon that
                // may have a different built-in crit effect or no crit at all.
                if self.app.selected_weapon != self.app.last_selected_weapon {
                    self.app.crit_effect_override = None;
                    self.app.last_selected_weapon = self.app.selected_weapon.clone();
                }

                // Modifiers section
                ui.separator();
                ui.collapsing("Modifiers", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Hit:");
                        ui.add(egui::DragValue::new(&mut self.app.hit_modifier).range(-3..=3));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Wound:");
                        ui.add(egui::DragValue::new(&mut self.app.wound_modifier).range(-3..=3));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Rend:");
                        ui.add(egui::DragValue::new(&mut self.app.rend_modifier).range(-3..=3));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Damage:");
                        ui.add(egui::DragValue::new(&mut self.app.damage_modifier).range(-3..=3));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Attacks:");
                        ui.add(egui::DragValue::new(&mut self.app.attack_modifier).range(-3..=3));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Crit Effect:");
                        let selected_weapon = self
                            .app
                            .units
                            .iter()
                            .find(|u| self.app.selected_attackers.contains(&u.id))
                            .and_then(|u| {
                                u.weapons
                                    .iter()
                                    .find(|w| w.name == self.app.selected_weapon)
                            });
                        // Clone the weapon's MW dice value (if any) for the override option.
                        // If the weapon has no Mortal Wounds crit, this is None and selecting
                        // "Mortal Wounds" will apply MW with no bonus damage.
                        let mw_value = selected_weapon.and_then(|w| {
                            if let Some(CritEffect::MortalWounds(v)) = &w.crit_hit {
                                v.clone()
                            } else {
                                None
                            }
                        });
                        egui::ComboBox::from_id_salt("crit_effect_override")
                            .selected_text(crit_effect_label(&self.app.crit_effect_override))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.app.crit_effect_override,
                                    None,
                                    "Default (use weapon)",
                                );
                                ui.selectable_value(
                                    &mut self.app.crit_effect_override,
                                    Some(CritEffect::AutoWound),
                                    "Auto Wound",
                                );
                                ui.selectable_value(
                                    &mut self.app.crit_effect_override,
                                    Some(CritEffect::ExtraHit),
                                    "Extra Hit",
                                );
                                ui.selectable_value(
                                    &mut self.app.crit_effect_override,
                                    Some(CritEffect::MortalWounds(mw_value.clone())),
                                    "Mortal Wounds",
                                );
                                ui.selectable_value(
                                    &mut self.app.crit_effect_override,
                                    Some(CritEffect::MortalWounds(None)),
                                    "Mortal Wounds (no bonus)",
                                );
                            });
                    });
                });

                // Champion checkbox
                ui.separator();
                ui.checkbox(&mut self.app.has_champion, "Champion (+1 attack)");

                // Attack override toggle
                ui.checkbox(&mut self.app.use_attack_override, "Override total attacks");

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
