use eframe::egui;

use crate::combat::types::DiceRoll;

/// Switch to grouped display (e.g. 4×6 3×5) when there are more than this.
const GROUP_THRESHOLD: usize = 20;
/// Wrap grouped display in a horizontal scroll area when there are more than this.
const SCROLL_THRESHOLD: usize = 30;

/// Displays dice rolls with color coding (gold=crit, green=success, red=fail).
/// 
/// For large numbers of rolls (>20), switches to grouped display (e.g., "4×6 3×5")
/// to maintain readability. Groups are ordered by die value in descending order.
pub struct DiceDisplay<'a> {
    rolls: &'a [DiceRoll],
}

impl<'a> DiceDisplay<'a> {
    pub fn new(rolls: &'a [DiceRoll]) -> Self {
        Self { rolls }
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        if self.rolls.is_empty() {
            ui.label("—");
            return;
        }

        if self.rolls.len() > GROUP_THRESHOLD {
            if self.rolls.len() > SCROLL_THRESHOLD {
                egui::ScrollArea::horizontal()
                    .max_height(40.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            self.show_grouped(ui);
                        });
                    });
            } else {
                ui.horizontal_wrapped(|ui| {
                    self.show_grouped(ui);
                });
            }
        } else {
            ui.horizontal_wrapped(|ui| {
                self.show_individual(ui);
            });
        }
    }

    fn show_individual(&self, ui: &mut egui::Ui) {
        // Renders each die separately with color coding
        for roll in self.rolls {
            let (color, label) = if roll.is_crit {
                (egui::Color32::GOLD, format!("{}", roll.value))
            } else if roll.success {
                (egui::Color32::GREEN, format!("{}", roll.value))
            } else {
                (egui::Color32::RED, format!("{}", roll.value))
            };

            ui.colored_label(color, egui::RichText::new(label).monospace().size(16.0));
        }
    }

    fn show_grouped(&self, ui: &mut egui::Ui) {
        // Renders dice as "count×value" groups, colored by the highest-priority roll in each group
        for group in self.grouped_data() {
            let color = group.color.unwrap_or(egui::Color32::WHITE);
            let text = format!("{}×{}", group.count, group.value);
            ui.colored_label(color, egui::RichText::new(text).monospace().size(16.0));
        }
    }

    /// Returns dice grouped by value for display purposes.
    /// Groups are ordered by die value descending (6, 5, 4, 3, 2, 1).
    /// Color is determined by the first roll in each group (crit > success > fail).
    pub(crate) fn grouped_data(&self) -> Vec<GroupedDie> {
        let mut counts: [usize; 6] = [0; 6];
        let mut colors: [Option<egui::Color32>; 6] = [None; 6];

        for roll in self.rolls {
            let idx = (roll.value as usize).saturating_sub(1);
            if idx < 6 {
                counts[idx] += 1;
                if colors[idx].is_none() {
                    colors[idx] = Some(if roll.is_crit {
                        egui::Color32::GOLD
                    } else if roll.success {
                        egui::Color32::GREEN
                    } else {
                        egui::Color32::RED
                    });
                }
            }
        }

        let mut groups: Vec<GroupedDie> = Vec::new();
        for value in (1u8..=6).rev() {
            let idx = (value as usize).saturating_sub(1);
            let count = counts[idx];
            if count > 0 {
                groups.push(GroupedDie {
                    value,
                    count,
                    color: colors[idx],
                });
            }
        }
        groups
    }

    /// Returns true if rolls is empty.
    #[allow(dead_code)]
    pub(crate) fn is_empty(&self) -> bool {
        self.rolls.is_empty()
    }

    /// Returns the number of rolls.
    #[allow(dead_code)]
    pub(crate) fn len(&self) -> usize {
        self.rolls.len()
    }

    /// Returns true if the display should use grouped mode.
    #[allow(dead_code)]
    pub(crate) fn should_group(&self) -> bool {
        self.rolls.len() > GROUP_THRESHOLD
    }
}

/// Represents a group of dice with the same value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupedDie {
    pub value: u8,
    pub count: usize,
    pub color: Option<egui::Color32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::types::DiceRoll;

    fn make_roll(value: u8, success: bool, is_crit: bool) -> DiceRoll {
        DiceRoll { value, success, is_crit }
    }

    // Helper to create a slice reference for DiceDisplay
    fn make_display(rolls: &[DiceRoll]) -> DiceDisplay<'_> {
        DiceDisplay::new(rolls)
    }

    #[test]
    fn empty_rolls_shows_dash() {
        let display = make_display(&[]);
        assert!(display.is_empty(), "Empty rolls should report is_empty as true");
    }

    #[test]
    fn individual_display_under_threshold() {
        // Exactly at threshold (20) should use individual display
        let rolls: Vec<DiceRoll> = (1..=20)
            .map(|v| make_roll(v, true, false))
            .collect();
        let display = make_display(&rolls);
        assert_eq!(display.len(), 20);
        assert!(!display.should_group(), "20 rolls should not be grouped");

        // Just under threshold (19)
        let rolls_19: Vec<DiceRoll> = (1..=19)
            .map(|v| make_roll(v, true, false))
            .collect();
        let display_19 = make_display(&rolls_19);
        assert!(!display_19.should_group(), "19 rolls should not be grouped");

        // One over threshold (21) should group
        let mut rolls_21: Vec<DiceRoll> = (1..=21)
            .map(|v| make_roll(v, true, false))
            .collect();
        rolls_21.push(make_roll(6, true, false));
        let display_21 = make_display(&rolls_21);
        assert!(display_21.should_group(), "21 rolls should be grouped");
    }

    #[test]
    fn grouped_display_counts_correctly() {
        // 21 rolls: three 6s, five 5s, four 4s, three 3s, two 2s, four 1s
        let rolls = vec![
            make_roll(6, true, false),
            make_roll(6, true, false),
            make_roll(6, true, false),
            make_roll(5, true, false),
            make_roll(5, true, false),
            make_roll(5, true, false),
            make_roll(5, true, false),
            make_roll(5, true, false),
            make_roll(4, true, false),
            make_roll(4, true, false),
            make_roll(4, true, false),
            make_roll(4, true, false),
            make_roll(3, true, false),
            make_roll(3, true, false),
            make_roll(3, true, false),
            make_roll(2, true, false),
            make_roll(2, true, false),
            make_roll(1, false, false),
            make_roll(1, false, false),
            make_roll(1, false, false),
            make_roll(1, false, false),
        ];
        let display = make_display(&rolls);
        let groups = display.grouped_data();

        // Should have 6 groups
        assert_eq!(groups.len(), 6);

        // Check counts in descending order
        assert_eq!(groups[0], GroupedDie { value: 6u8, count: 3, color: Some(egui::Color32::GREEN) });
        assert_eq!(groups[1], GroupedDie { value: 5u8, count: 5, color: Some(egui::Color32::GREEN) });
        assert_eq!(groups[2], GroupedDie { value: 4u8, count: 4, color: Some(egui::Color32::GREEN) });
        assert_eq!(groups[3], GroupedDie { value: 3u8, count: 3, color: Some(egui::Color32::GREEN) });
        assert_eq!(groups[4], GroupedDie { value: 2u8, count: 2, color: Some(egui::Color32::GREEN) });
        assert_eq!(groups[5], GroupedDie { value: 1u8, count: 4, color: Some(egui::Color32::RED) });
    }

    #[test]
    fn grouped_display_omits_zero_counts() {
        // Only rolls of 6 and 4 - should not show 1, 2, 3, 5
        let rolls = vec![
            make_roll(6, true, false),
            make_roll(6, true, false),
            make_roll(4, true, false),
        ];
        let display = make_display(&rolls);
        let groups = display.grouped_data();

        // Should only have 2 groups (6 and 4)
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].value, 6);
        assert_eq!(groups[1].value, 4);
    }

    #[test]
    fn grouped_display_descending_order() {
        // Rolls in random order
        let rolls = vec![
            make_roll(3, true, false),
            make_roll(1, false, false),
            make_roll(5, true, false),
            make_roll(2, false, false),
            make_roll(6, true, false),
            make_roll(4, true, false),
        ];
        let display = make_display(&rolls);
        let groups = display.grouped_data();

        // Should be in descending order: 6, 5, 4, 3, 2, 1
        assert_eq!(groups.len(), 6);
        assert_eq!(groups[0].value, 6);
        assert_eq!(groups[1].value, 5);
        assert_eq!(groups[2].value, 4);
        assert_eq!(groups[3].value, 3);
        assert_eq!(groups[4].value, 2);
        assert_eq!(groups[5].value, 1);
    }

    #[test]
    fn grouped_display_preserves_colors() {
        // Mix of crits, successes, and failures
        let rolls = vec![
            make_roll(6, true, true),   // GOLD (crit)
            make_roll(6, true, false),  // GREEN (success)
            make_roll(5, true, false),  // GREEN (success)
            make_roll(4, false, false), // RED (failure)
            make_roll(1, false, false), // RED (failure)
            make_roll(2, false, false), // RED (failure)
        ];
        let display = make_display(&rolls);
        let groups = display.grouped_data();

        // Check that colors are preserved correctly
        // First group (6): should be GOLD (crit takes priority)
        assert_eq!(groups[0].value, 6);
        assert_eq!(groups[0].color, Some(egui::Color32::GOLD));

        // Second group (5): should be GREEN
        assert_eq!(groups[1].value, 5);
        assert_eq!(groups[1].color, Some(egui::Color32::GREEN));

        // Third group (4): should be RED
        assert_eq!(groups[2].value, 4);
        assert_eq!(groups[2].color, Some(egui::Color32::RED));

        // Fourth group (2): should be RED
        assert_eq!(groups[3].value, 2);
        assert_eq!(groups[3].color, Some(egui::Color32::RED));

        // Fifth group (1): should be RED
        assert_eq!(groups[4].value, 1);
        assert_eq!(groups[4].color, Some(egui::Color32::RED));
    }

    #[test]
    fn value_one_dice_counted() {
        // Single roll of 1 - this was a bug where 1s weren't counted
        let rolls = vec![make_roll(1, false, false)];
        let display = make_display(&rolls);
        let groups = display.grouped_data();

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].value, 1);
        assert_eq!(groups[0].count, 1);
        assert_eq!(groups[0].color, Some(egui::Color32::RED));
    }

    #[test]
    fn value_one_dice_in_larger_set() {
        // Multiple 1s in a larger set
        let rolls = vec![
            make_roll(6, true, false),
            make_roll(1, false, false),
            make_roll(1, false, false),
            make_roll(4, true, false),
            make_roll(1, false, false),
        ];
        let display = make_display(&rolls);
        let groups = display.grouped_data();

        // Find the group for value 1
        let ones_group = groups.iter().find(|g| g.value == 1).expect("Should have a group for 1");
        assert_eq!(ones_group.count, 3, "Should count 3 ones");
        assert_eq!(ones_group.color, Some(egui::Color32::RED));
    }

    #[test]
    fn all_same_value_grouped() {
        // All rolls are 4s
        let rolls = vec![
            make_roll(4, true, false),
            make_roll(4, true, false),
            make_roll(4, true, false),
            make_roll(4, true, false),
            make_roll(4, true, false),
        ];
        let display = make_display(&rolls);
        let groups = display.grouped_data();

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].value, 4);
        assert_eq!(groups[0].count, 5);
    }

    #[test]
    fn invalid_dice_values_ignored() {
        // Dice values should be 1-6, values outside this range are ignored
        let rolls = vec![
            make_roll(0, false, false),  // Invalid: 0
            make_roll(7, false, false),  // Invalid: 7
            make_roll(1, false, false),  // Valid
            make_roll(6, true, false),   // Valid
        ];
        let display = make_display(&rolls);
        let groups = display.grouped_data();

        // Should only have 2 groups (1 and 6)
        assert_eq!(groups.len(), 2);
        // Values 0 and 7 are ignored
    }

    #[test]
    fn threshold_constant_values() {
        assert_eq!(GROUP_THRESHOLD, 20);
        assert_eq!(SCROLL_THRESHOLD, 30);
    }
}
