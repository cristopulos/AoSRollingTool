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
        // Search field filters units by name or faction
        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(&mut self.app.defender_search);
        });
        ui.separator();

        ui.push_id("defender_panel", |ui| {
            // Apply search filter to both unit names and faction names
            let query = self.app.defender_search.to_lowercase();
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
                egui::vec2(ui.available_width(), self.app.defender_panel_height),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("defender_list")
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
                        });
                },
            );

            ui.separator();
            ui.checkbox(&mut self.app.include_ward, "Include Ward Saves");

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
        });
    }
}
