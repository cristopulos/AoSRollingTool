use eframe::egui;

use std::sync::mpsc::Receiver;

use crate::combat::engine::resolve_combat;
use crate::combat::simulation::SimulationResult;
use crate::combat::types::CombatResult;
use crate::data::loader::load_units_from_path;
use crate::data::models::{CritEffect, Unit};
use crate::ui::panels::combat_view::CombatView;
use crate::ui::panels::log_panel::LogPanel;
use crate::ui::panels::recent_panel::RecentPanel;
use crate::ui::panels::target_panel::TargetPanel;
use crate::ui::panels::unit_panel::UnitPanel;

pub struct AoSApp {
    pub units: Vec<Unit>,
    pub selected_attackers: Vec<String>, // Unit IDs
    pub selected_weapon: String,
    pub selected_defender: String,
    pub num_models: usize,         // Number of attacking models
    pub has_champion: bool,        // Adds +1 to total attacks
    pub use_attack_override: bool, // Toggle between models×attack and fixed attacks
    pub attack_override: usize,    // Fixed attack count when override is enabled
    pub include_ward: bool,
    pub stop_after_wound: bool,
    pub attacker_search: String,
    pub defender_search: String,
    pub attacker_panel_height: f32,
    pub defender_panel_height: f32,
    pub hit_modifier: i8,
    pub wound_modifier: i8,
    pub rend_modifier: i8,
    pub damage_modifier: i8,
    /// Modifies the per-model attack count (e.g., "D6" → "D6+1" or "2" → "4").
    /// Applied per-model before summing, so with modifier +2 and 5 models:
    /// "2" attack becomes 5 × 4 = 20 attacks. Ignored when use_attack_override is true.
    pub attack_modifier: i8,
    /// Overrides the weapon's built-in crit effect when set.
    pub crit_effect_override: Option<CritEffect>,
    /// Tracks the last selected weapon name to detect weapon changes.
    /// When the selected weapon changes, crit_effect_override is reset to None.
    pub last_selected_weapon: String,
    pub current_result: Option<CombatResult>,
    pub combat_log: Vec<CombatResult>,
    pub error_message: Option<String>,
    pub simulation_result: Option<SimulationResult>,
    pub simulation_rx: Option<Receiver<SimulationResult>>,
    pub is_simulating: bool,
    /// All recently used units (unit_id, unit_name), merged from both attacker
    /// and defender roles. Updated after each combat roll. Most recent first.
    /// No entry limit. Selecting a unit from this list lets the user pick
    /// whether it should be attacker or defender.
    pub recent_units: Vec<(String, String)>,
}

impl AoSApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let units = Self::load_default_units();
        Self {
            units,
            selected_attackers: Vec::new(),
            selected_weapon: String::new(),
            selected_defender: String::new(),
            num_models: 1,
            has_champion: false,
            use_attack_override: false,
            attack_override: 10,
            include_ward: true,
            stop_after_wound: false,
            attacker_search: String::new(),
            defender_search: String::new(),
            attacker_panel_height: 260.0,
            defender_panel_height: 140.0,
            hit_modifier: 0,
            wound_modifier: 0,
            rend_modifier: 0,
            damage_modifier: 0,
            attack_modifier: 0,
            crit_effect_override: None,
            last_selected_weapon: String::new(),
            current_result: None,
            combat_log: Vec::new(),
            error_message: None,
            simulation_result: None,
            simulation_rx: None,
            is_simulating: false,
            recent_units: Vec::new(),
        }
    }

    fn load_default_units() -> Vec<Unit> {
        // Try loading from embedded resources first, then from local file
        if let Ok(units) = load_units_from_path("resources/units.json") {
            return units;
        }
        if let Ok(units) = load_units_from_path("src/resources/units.json") {
            return units;
        }
        log::warn!("Could not load units.json from resources/ or src/resources/");
        Vec::new()
    }

    pub fn roll_combat(&mut self) {
        if self.selected_attackers.is_empty() {
            self.error_message = Some("Select at least one attacker".into());
            return;
        }
        if self.selected_defender.is_empty() && !self.stop_after_wound {
            self.error_message = Some("Select a defender".into());
            return;
        }
        if self.selected_weapon.is_empty() {
            self.error_message = Some("Select a weapon".into());
            return;
        }

        // For now, use the first selected attacker
        let attacker = self
            .units
            .iter()
            .find(|u| u.id == self.selected_attackers[0])
            .cloned();
        let defender = if self.stop_after_wound && self.selected_defender.is_empty() {
            Some(crate::data::models::Unit {
                id: "none".into(),
                name: "Defender (not selected)".into(),
                faction: "-".into(),
                save: 7,
                ward: None,
                weapons: vec![],
            })
        } else {
            self.units
                .iter()
                .find(|u| u.id == self.selected_defender)
                .cloned()
        };

        match (attacker, defender) {
            (Some(attacker), Some(defender)) => {
                let weapon = attacker
                    .weapons
                    .iter()
                    .find(|w| w.name == self.selected_weapon)
                    .cloned();

                match weapon {
                    Some(weapon) => {
                        let result = resolve_combat(
                            &attacker,
                            &defender,
                            &weapon,
                            self.num_models,
                            self.has_champion,
                            self.use_attack_override,
                            self.attack_override,
                            self.include_ward,
                            self.stop_after_wound,
                            self.hit_modifier,
                            self.wound_modifier,
                            self.rend_modifier,
                            self.damage_modifier,
                            self.attack_modifier,
                            self.crit_effect_override.clone(),
                            None,
                        );
                        // Update unified recently-used list before storing result.
                        // Insert defender first, then attacker, so attacker ends up at index 0
                        // (most recent) matching the typical selection flow.
                        if let Some(unit) = self.units.iter().find(|u| u.name == defender.name) {
                            self.recent_units.retain(|(id, _)| id != &unit.id);
                            self.recent_units.insert(0, (unit.id.clone(), unit.name.clone()));
                        }
                        if let Some(unit) = self.units.iter().find(|u| u.name == attacker.name) {
                            self.recent_units.retain(|(id, _)| id != &unit.id);
                            self.recent_units.insert(0, (unit.id.clone(), unit.name.clone()));
                        }

                        self.combat_log.push(result.clone());
                        self.current_result = Some(result);
                        self.error_message = None;
                        self.simulation_result = None;
                        self.simulation_rx = None;
                        self.is_simulating = false;
                    }
                    None => {
                        self.error_message = Some("Selected weapon not found".into());
                    }
                }
            }
            _ => {
                self.error_message = Some("Selected units not found".into());
            }
        }
    }
}

impl eframe::App for AoSApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Keyboard zoom controls: = to zoom in, - to zoom out
        {
            let zoom_in = ctx.input(|i| i.key_pressed(egui::Key::Equals));
            let zoom_out = ctx.input(|i| i.key_pressed(egui::Key::Minus));
            if zoom_in || zoom_out {
                let current = ctx.input(|i| i.pixels_per_point());
                let factor = if zoom_in { 1.2 } else { 1.0 / 1.2 };
                let new_scale = (current * factor).clamp(0.5, 3.0);
                ctx.set_pixels_per_point(new_scale);
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Age of Sigmar 4th Edition - Combat Roller");
            ui.separator();

            if let Some(error) = &self.error_message {
                ui.colored_label(egui::Color32::RED, error);
                ui.separator();
            }

            let full_height = ui.available_rect_before_wrap().height();
            ui.horizontal(|ui| {
                // Left panel - unit selection
                ui.vertical(|ui| {
                    ui.set_width(250.0);
                    let max_column_height = full_height;
                    UnitPanel::new(self).show(ui);

                    // Draggable splitter between attacker and defender panels
                    let splitter_response = ui.allocate_response(
                        egui::vec2(ui.available_width(), 6.0),
                        egui::Sense::click_and_drag(),
                    );
                    if splitter_response.hovered() || splitter_response.dragged() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
                    }
                    if splitter_response.dragged() {
                        let delta = splitter_response.drag_delta().y;
                        self.attacker_panel_height += delta;
                        self.defender_panel_height -= delta;

                        let splitter_height = 6.0;
                        let max_combined = (max_column_height - splitter_height).max(160.0);
                        let total = self.attacker_panel_height + self.defender_panel_height;
                        if total > max_combined {
                            let overflow = total - max_combined;
                            let attacker_ratio = self.attacker_panel_height / total;
                            self.attacker_panel_height -= overflow * attacker_ratio;
                            self.defender_panel_height -= overflow * (1.0 - attacker_ratio);
                        }
                        self.attacker_panel_height =
                            self.attacker_panel_height.clamp(80.0, max_combined - 80.0);
                        self.defender_panel_height =
                            self.defender_panel_height.clamp(80.0, max_combined - 80.0);
                    }
                    ui.painter().rect_filled(
                        splitter_response.rect,
                        0.0,
                        if splitter_response.hovered() || splitter_response.dragged() {
                            ui.visuals().selection.bg_fill
                        } else {
                            ui.visuals().widgets.inactive.weak_bg_fill
                        },
                    );

                    TargetPanel::new(self).show(ui);
                });

                ui.separator();

                // Middle panel - recently used units
                ui.vertical(|ui| {
                    ui.set_width(200.0);
                    RecentPanel::new(self).show(ui);
                });

                ui.separator();

                // Right panel - combat display
                ui.vertical(|ui| {
                    ui.set_min_width(500.0);

                    if ui.button("ROLL COMBAT").clicked() {
                        self.roll_combat();
                    }

                    if let Some(result) = &self.current_result {
                        if !self.is_simulating && ui.button("SIMULATE (10,000 runs)").clicked() {
                            self.is_simulating = true;
                            self.simulation_result = None;
                            let (tx, rx) = std::sync::mpsc::channel();
                            self.simulation_rx = Some(rx);

                            let attacker = result.attacker_name.clone();
                            let defender = result.defender_name.clone();
                            let weapon = result.weapon_name.clone();

                            // Look up the original units and weapon from state
                            let attacker_unit = self
                                .units
                                .iter()
                                .find(|u| u.name == attacker)
                                .cloned()
                                .unwrap();
                            let defender_unit =
                                if self.stop_after_wound && self.selected_defender.is_empty() {
                                    crate::data::models::Unit {
                                        id: "none".into(),
                                        name: "Defender (not selected)".into(),
                                        faction: "-".into(),
                                        save: 7,
                                        ward: None,
                                        weapons: vec![],
                                    }
                                } else {
                                    self.units
                                        .iter()
                                        .find(|u| u.name == defender)
                                        .cloned()
                                        .unwrap()
                                };
                            let weapon_obj = attacker_unit
                                .weapons
                                .iter()
                                .find(|w| w.name == weapon)
                                .cloned()
                                .unwrap();
                            let actual_result = result.clone();
                            let num_models = self.num_models;
                            let has_champion = self.has_champion;
                            let use_attack_override = self.use_attack_override;
                            let attack_override = self.attack_override;
                            let include_ward = self.include_ward;
                            let hit_modifier = self.hit_modifier;
                            let wound_modifier = self.wound_modifier;
                            let rend_modifier = self.rend_modifier;
                            let damage_modifier = self.damage_modifier;
                            let attack_modifier = self.attack_modifier;
                            let crit_effect_override = self.crit_effect_override.clone();

                            std::thread::spawn(move || {
                                let sim = crate::combat::simulation::run_simulation(
                                    &attacker_unit,
                                    &defender_unit,
                                    &weapon_obj,
                                    num_models,
                                    has_champion,
                                    use_attack_override,
                                    attack_override,
                                    include_ward,
                                    hit_modifier,
                                    wound_modifier,
                                    rend_modifier,
                                    damage_modifier,
                                    attack_modifier,
                                    crit_effect_override,
                                    &actual_result,
                                    10_000,
                                );
                                let _ = tx.send(sim);
                            });
                        }
                    }

                    // Poll for completed simulation
                    if let Some(rx) = self.simulation_rx.take() {
                        match rx.try_recv() {
                            Ok(result) => {
                                self.simulation_result = Some(result);
                                self.is_simulating = false;
                            }
                            Err(std::sync::mpsc::TryRecvError::Empty) => {
                                self.simulation_rx = Some(rx);
                            }
                            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                                self.is_simulating = false;
                            }
                        }
                    }

                    if self.is_simulating {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Running 10,000 simulations...");
                        });
                    }

                    ui.separator();

                    if let Some(result) = &self.current_result {
                        CombatView::new(result, self.simulation_result.as_ref()).show(ui);
                    } else {
                        ui.label("Select units and weapon, then click ROLL COMBAT");
                    }

                    ui.separator();
                    LogPanel::new(&self.combat_log).show(ui);
                });
            });
        });
    }
}
