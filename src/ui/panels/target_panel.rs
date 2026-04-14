use eframe::egui;

use crate::app::AoSApp;

pub struct TargetPanel<'a> {
    app: &'a mut AoSApp,
}

impl<'a> TargetPanel<'a> {
    pub fn new(app: &'a mut AoSApp) -> Self {
        Self { app }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("Defender");
        ui.separator();

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
                    ui.horizontal(|ui| {
                        ui.radio_value(
                            &mut self.app.selected_defender,
                            unit.id.clone(),
                            unit.name.to_string(),
                        );
                    });
                }
            });
        }

        ui.separator();
        ui.checkbox(&mut self.app.include_ward,
            "Include Ward Saves",
        );

        // Show defender stats if selected
        if !self.app.selected_defender.is_empty() {
            if let Some(defender) = self
                .app
                .units
                .iter()
                .find(|u| u.id == self.app.selected_defender)
            {
                ui.separator();
                ui.label(format!("Save: {}+", defender.save));
                if let Some(ward) = defender.ward {
                    ui.label(format!("Ward: {}+", ward));
                } else {
                    ui.label("Ward: None");
                }
            }
        }
    }
}
