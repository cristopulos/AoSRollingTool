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

        if self.result.stopped_after_wound {
            ui.heading("Sequence Stopped - Defender Rolls Saves");
            ui.horizontal(|ui| {
                ui.label("Hits determined:");
                ui.strong(format!("{}", self.result.total_hits));
            });
            ui.horizontal(|ui| {
                ui.label("Wounds to save:");
                ui.strong(format!("{}", self.result.total_wounds));
            });
            if self.result.mortal_wounds > 0 {
                ui.label(format!(
                    "(includes {} mortal wounds from critical hits)",
                    self.result.mortal_wounds
                ));
            }
            ui.label("Save, Damage, and Ward phases are pending — roll these externally.");
        } else {
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
}
