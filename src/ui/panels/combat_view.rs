use eframe::egui;

use crate::combat::types::CombatResult;
use crate::ui::widgets::phase_result::PhaseResultCard;

pub struct CombatView<'a> {
    result: &'a CombatResult,
}

impl<'a> CombatView<'a> {
    pub fn new(result: &'a CombatResult) -> Self {
        Self { result }
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        ui.heading("Combat Sequence");
        ui.separator();

        ui.label(format!(
            "{} attacking {} with {}",
            self.result.attacker_name, self.result.defender_name, self.result.weapon_name
        ));

        ui.separator();

        egui::Grid::new("combat_phases")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .show(ui, |ui| {
                for phase in &self.result.phases {
                    PhaseResultCard::new(phase).show(ui);
                    ui.end_row();
                }
            });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("FINAL DAMAGE:");
            ui.strong(format!("{}", self.result.final_damage));
        });

        if self.result.mortal_wounds > 0 {
            ui.label(format!(
                "(includes {} mortal wounds)",
                self.result.mortal_wounds
            ));
        }
    }
}
