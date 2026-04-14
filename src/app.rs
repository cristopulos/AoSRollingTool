use eframe::egui;

use std::sync::mpsc::Receiver;

use crate::combat::engine::resolve_combat;
use crate::combat::simulation::SimulationResult;
use crate::combat::types::CombatResult;
use crate::data::loader::load_units_from_path;
use crate::data::models::Unit;
use crate::ui::panels::combat_view::CombatView;
use crate::ui::panels::log_panel::LogPanel;
use crate::ui::panels::target_panel::TargetPanel;
use crate::ui::panels::unit_panel::UnitPanel;

pub struct AoSApp {
    pub units: Vec<Unit>,
    pub selected_attackers: Vec<String>, // Unit IDs
    pub selected_weapon: String,
    pub selected_defender: String,
    pub num_models: usize, // Number of attacking models
    pub has_champion: bool, // Adds +1 to total attacks
    pub use_attack_override: bool, // Toggle between models×attack and fixed attacks
    pub attack_override: usize, // Fixed attack count when override is enabled
    pub include_ward: bool,
    pub stop_after_wound: bool,
    pub current_result: Option<CombatResult>,
    pub combat_log: Vec<CombatResult>,
    pub error_message: Option<String>,
    pub simulation_result: Option<SimulationResult>,
    pub simulation_rx: Option<Receiver<SimulationResult>>,
    pub is_simulating: bool,
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
            current_result: None,
            combat_log: Vec::new(),
            error_message: None,
            simulation_result: None,
            simulation_rx: None,
            is_simulating: false,
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
                            None,
                        );
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
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Age of Sigmar 4th Edition - Combat Roller");
            ui.separator();

            if let Some(error) = &self.error_message {
                ui.colored_label(egui::Color32::RED, error);
                ui.separator();
            }

            ui.horizontal(|ui| {
                // Left panel - unit selection
                ui.vertical(|ui| {
                    ui.set_width(250.0);
                    UnitPanel::new(self).show(ui);
                    ui.separator();
                    TargetPanel::new(self).show(ui);
                });

                ui.separator();

                // Right panel - combat display
                ui.vertical(|ui| {
                    ui.set_min_width(500.0);

                    if ui.button("ROLL COMBAT").clicked() {
                        self.roll_combat();
                    }

                    if let Some(result) = &self.current_result {
                        if !self.is_simulating
                            && ui.button("SIMULATE (10,000 runs)").clicked()
                        {
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
                            let defender_unit = if self.stop_after_wound
                                && self.selected_defender.is_empty()
                            {
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
