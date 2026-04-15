use crate::combat::dice::{parse_dice_string, roll_d6_batch};
use crate::combat::types::{CombatResult, DiceRoll, Phase, PhaseResult, WardResult};
use crate::data::models::{CritEffect, Unit, Weapon};

/// Applies a damage modifier to a damage string, returning a new damage string.
/// e.g., "D3" + 2 -> "D3+2"
/// e.g., "2" + 1  -> "3"
/// e.g., "2" - 1  -> "1"
/// e.g., "D6+2" - 1 -> "D6+1"
fn apply_damage_modifier(damage_str: &str, modifier: i8) -> String {
    if modifier == 0 {
        return damage_str.to_string();
    }
    // Handle pure numeric strings specially
    if let Ok(base) = damage_str.parse::<i16>() {
        let result = (base + modifier as i16).max(1);
        return result.to_string();
    }
    // For dice strings (D6, D3, etc.), append the modifier
    if modifier > 0 {
        format!("{}+{}", damage_str, modifier)
    } else {
        format!("{}{}", damage_str, modifier)
    }
}

/// Calculate the save target number.
/// If the result is > 6, saves auto-fail.
pub fn calculate_save_target(defender_save: u8, weapon_rend: i8, rend_modifier: i8) -> u8 {
    let target = defender_save as i8 - weapon_rend + rend_modifier;
    target.max(0) as u8
}

/// Resolve the hit phase for a given number of attacks.
/// Returns (hits, auto_wounds, extra_hits, mortal_wounds, rolls).
pub fn resolve_hits(
    weapon: &Weapon,
    attacks: usize,
    effective_to_hit: u8,
    provided_rolls: Option<&[u8]>,
) -> (usize, usize, usize, usize, Vec<DiceRoll>) {
    let rolls = match provided_rolls {
        Some(r) => r.to_vec(),
        None => roll_d6_batch(attacks),
    };

    let mut hits = 0;
    let mut auto_wounds = 0;
    let mut extra_hits = 0;
    let mut mortal_wounds = 0;
    let mut dice_rolls = Vec::with_capacity(rolls.len());

    for roll in rolls {
        let is_six = roll == 6;
        let is_hit = roll >= effective_to_hit;

        let mut dice = DiceRoll {
            value: roll,
            success: false,
            is_crit: is_six,
        };

        if is_six {
            match weapon.crit_hit {
                Some(CritEffect::AutoWound) => {
                    dice.success = true;
                    auto_wounds += 1;
                }
                Some(CritEffect::ExtraHit) => {
                    dice.success = true;
                    hits += 1; // Base hit
                    extra_hits += 1; // Extra hit
                }
                Some(CritEffect::MortalWounds(ref opt_dmg)) => {
                    dice.success = true;
                    if let Some(dmg) = opt_dmg {
                        let mw = parse_dice_string(dmg).unwrap_or(1) as usize;
                        mortal_wounds += mw;
                    }
                }
                None => {
                    dice.success = true;
                    hits += 1;
                }
            }
        } else if is_hit {
            dice.success = true;
            hits += 1;
        }

        dice_rolls.push(dice);
    }

    (hits, auto_wounds, extra_hits, mortal_wounds, dice_rolls)
}

/// Resolve the wound phase.
pub fn resolve_wounds(
    _weapon: &Weapon,
    wounds_to_roll: usize,
    effective_to_wound: u8,
    provided_rolls: Option<&[u8]>,
) -> (usize, Vec<DiceRoll>) {
    let rolls = match provided_rolls {
        Some(r) => r.to_vec(),
        None => roll_d6_batch(wounds_to_roll),
    };

    let mut successes = 0;
    let mut dice_rolls = Vec::with_capacity(rolls.len());

    for roll in rolls {
        let success = roll >= effective_to_wound;
        if success {
            successes += 1;
        }
        dice_rolls.push(DiceRoll {
            value: roll,
            success,
            is_crit: false,
        });
    }

    (successes, dice_rolls)
}

/// Resolve the save phase.
pub fn resolve_save(
    save_target: u8,
    wounds: usize,
    provided_rolls: Option<&[u8]>,
) -> (usize, Vec<DiceRoll>, bool) {
    // Auto-fail if save target > 6
    if save_target > 6 {
        return (
            wounds,
            Vec::new(),
            true, // auto_fails
        );
    }

    let rolls = match provided_rolls {
        Some(r) => r.to_vec(),
        None => roll_d6_batch(wounds),
    };

    let mut unsaved = 0;
    let mut dice_rolls = Vec::with_capacity(rolls.len());

    for roll in rolls {
        let saved = roll >= save_target;
        if !saved {
            unsaved += 1;
        }
        dice_rolls.push(DiceRoll {
            value: roll,
            success: !saved, // Success = wound gets through
            is_crit: false,
        });
    }

    (unsaved, dice_rolls, false)
}

/// Resolve ward saves.
pub fn resolve_ward(damage: usize, ward_target: u8, provided_rolls: Option<&[u8]>) -> WardResult {
    let rolls = match provided_rolls {
        Some(r) => r.to_vec(),
        None => roll_d6_batch(damage),
    };

    let mut saved = 0;
    let mut dice_rolls = Vec::with_capacity(rolls.len());

    for roll in rolls {
        let success = roll >= ward_target;
        if success {
            saved += 1;
        }
        dice_rolls.push(DiceRoll {
            value: roll,
            success,
            is_crit: false,
        });
    }

    WardResult {
        final_damage: damage - saved,
        wounds_saved: saved,
        rolls: dice_rolls,
    }
}

fn has_dice(s: &str) -> bool {
    s.to_uppercase().contains('D')
}

/// Resolve a full combat sequence.
///
/// Set `stop_after_wound` to true to stop the sequence after the Wound phase.
/// This returns the hit and wound totals and marks subsequent phases as pending,
/// allowing the defender to roll saves externally (useful for in-person games).
#[allow(clippy::too_many_arguments)]
pub fn resolve_combat(
    attacker: &Unit,
    defender: &Unit,
    weapon: &Weapon,
    num_models: usize,
    has_champion: bool,
    use_attack_override: bool,
    attack_override: usize,
    include_ward: bool,
    /* When true, only process Hit and Wound phases. Save, Damage, and Ward phases
    are marked as pending, and the defender rolls saves externally. */
    stop_after_wound: bool,
    hit_modifier: i8,
    wound_modifier: i8,
    rend_modifier: i8,
    damage_modifier: i8,
    provided_rolls: Option<&[u8]>, // For testing only
) -> CombatResult {
    // Compute effective values with modifiers
    let effective_to_hit = (weapon.to_hit as i8 - hit_modifier).clamp(1, 6) as u8;
    let effective_to_wound = (weapon.to_wound as i8 - wound_modifier).clamp(1, 6) as u8;
    let effective_damage = apply_damage_modifier(&weapon.damage, damage_modifier);

    // Determine number of attacks
    let (attacks, attack_variance, hit_description) = if use_attack_override {
        let desc = if hit_modifier != 0 {
            format!(
                "Hit ({}+ → {}+) - {} fixed attacks ({} to hit)",
                weapon.to_hit,
                effective_to_hit,
                attack_override,
                if hit_modifier > 0 {
                    format!("+{}", hit_modifier)
                } else {
                    hit_modifier.to_string()
                }
            )
        } else {
            format!(
                "Hit ({}+) - {} fixed attacks",
                weapon.to_hit, attack_override
            )
        };
        (attack_override, None, desc)
    } else {
        let (base_attacks, variance) = if has_dice(&weapon.attack) {
            let results: Vec<u8> = (0..num_models)
                .map(|_| parse_dice_string(&weapon.attack).unwrap_or(1) as u8)
                .collect();
            let total = results.iter().map(|&x| x as usize).sum();
            (
                total,
                Some(crate::combat::types::VarianceStep::AttackRoll {
                    per_model: weapon.attack.clone(),
                    results,
                    total,
                }),
            )
        } else {
            let fixed = parse_dice_string(&weapon.attack).unwrap_or(1) as usize;
            (num_models.saturating_mul(fixed), None)
        };

        let total_attacks = if has_champion {
            base_attacks.saturating_add(1)
        } else {
            base_attacks
        };

        let desc = if hit_modifier != 0 {
            if has_champion {
                format!(
                    "Hit ({}+ → {}+) - {} models × {} attacks + 1 champion = {} total ({} to hit)",
                    weapon.to_hit,
                    effective_to_hit,
                    num_models,
                    weapon.attack,
                    total_attacks,
                    if hit_modifier > 0 {
                        format!("+{}", hit_modifier)
                    } else {
                        hit_modifier.to_string()
                    }
                )
            } else {
                format!(
                    "Hit ({}+ → {}+) - {} models × {} attacks = {} total ({} to hit)",
                    weapon.to_hit,
                    effective_to_hit,
                    num_models,
                    weapon.attack,
                    total_attacks,
                    if hit_modifier > 0 {
                        format!("+{}", hit_modifier)
                    } else {
                        hit_modifier.to_string()
                    }
                )
            }
        } else if has_champion {
            format!(
                "Hit ({}+) - {} models × {} attacks + 1 champion = {} total",
                weapon.to_hit, num_models, weapon.attack, total_attacks
            )
        } else {
            format!(
                "Hit ({}+) - {} models × {} attacks = {} total",
                weapon.to_hit, num_models, weapon.attack, total_attacks
            )
        };

        (total_attacks, variance, desc)
    };

    // Phase 1: Hit
    let (hits, auto_wounds, extra_hits, mortal_wounds_from_crits, hit_rolls) =
        resolve_hits(weapon, attacks, effective_to_hit, provided_rolls);

    let hit_phase = PhaseResult {
        phase: Phase::Hit,
        rolls: hit_rolls,
        successes: hits + auto_wounds + extra_hits,
        failures: attacks.saturating_sub(hits + auto_wounds + extra_hits),
        total_output: hits + auto_wounds + extra_hits,
        auto_fails: false,
        skipped: false,
        description: hit_description,
        variance_step: attack_variance,
    };

    // Phase 2: Wound
    let total_wounds_to_roll = hits + extra_hits;
    let (wounds, wound_rolls) = if total_wounds_to_roll > 0 {
        resolve_wounds(weapon, total_wounds_to_roll, effective_to_wound, None)
    } else {
        (0, Vec::new())
    };

    let total_wounds = wounds + auto_wounds;

    let wound_desc = if wound_modifier != 0 {
        format!(
            "Wound ({}+ → {}+) ({} to wound)",
            weapon.to_wound,
            effective_to_wound,
            if wound_modifier > 0 {
                format!("+{}", wound_modifier)
            } else {
                wound_modifier.to_string()
            }
        )
    } else {
        format!("Wound ({}+)", weapon.to_wound)
    };

    let wound_phase = PhaseResult {
        phase: Phase::Wound,
        rolls: wound_rolls,
        successes: wounds,
        failures: total_wounds_to_roll.saturating_sub(wounds),
        total_output: total_wounds,
        auto_fails: false,
        skipped: false,
        description: wound_desc,
        variance_step: None,
    };

    // Early stop: only process hit and wound phases; mortal wounds from crits are still counted.
    if stop_after_wound {
        let save_target = calculate_save_target(defender.save, weapon.rend, rend_modifier);
        let save_desc = if rend_modifier != 0 {
            format!(
                "Save ({}+ → {}+) - Pending ({} rend)",
                calculate_save_target(defender.save, weapon.rend, 0),
                save_target,
                if rend_modifier > 0 {
                    format!("+{}", rend_modifier)
                } else {
                    rend_modifier.to_string()
                }
            )
        } else {
            format!("Save ({}+) - Pending", save_target)
        };
        let save_phase = PhaseResult {
            phase: Phase::Save,
            rolls: Vec::new(),
            successes: 0,
            failures: 0,
            total_output: 0,
            auto_fails: false,
            skipped: true,
            description: save_desc,
            variance_step: None,
        };
        let damage_phase = PhaseResult {
            phase: Phase::Damage,
            rolls: Vec::new(),
            successes: 0,
            failures: 0,
            total_output: 0,
            auto_fails: false,
            skipped: true,
            description: if damage_modifier != 0 {
                format!(
                    "Damage ({} → {}) per wound - Pending ({} damage)",
                    weapon.damage,
                    effective_damage,
                    if damage_modifier > 0 {
                        format!("+{}", damage_modifier)
                    } else {
                        damage_modifier.to_string()
                    }
                )
            } else {
                format!("Damage ({} per wound) - Pending", weapon.damage)
            },
            variance_step: None,
        };
        let ward_phase = PhaseResult {
            phase: Phase::Ward,
            rolls: Vec::new(),
            successes: 0,
            failures: 0,
            total_output: 0,
            auto_fails: false,
            skipped: true,
            description: defender.ward.map_or("Ward (-) - Pending".to_string(), |w| {
                format!("Ward ({}+) - Pending", w)
            }),
            variance_step: None,
        };
        return CombatResult {
            attacker_name: attacker.name.clone(),
            weapon_name: weapon.name.clone(),
            defender_name: defender.name.clone(),
            phases: vec![hit_phase, wound_phase, save_phase, damage_phase, ward_phase],
            final_damage: 0,
            mortal_wounds: mortal_wounds_from_crits,
            stopped_after_wound: true,
            total_hits: hits + auto_wounds + extra_hits,
            total_wounds,
        };
    }

    // Phase 3: Save
    let save_target = calculate_save_target(defender.save, weapon.rend, rend_modifier);
    let (unsaved, save_rolls, auto_fails) = if total_wounds > 0 {
        resolve_save(save_target, total_wounds, None)
    } else {
        (0, Vec::new(), false)
    };

    let save_desc = if rend_modifier != 0 {
        format!(
            "Save ({}+ → {}+) ({} rend)",
            calculate_save_target(defender.save, weapon.rend, 0),
            save_target,
            if rend_modifier > 0 {
                format!("+{}", rend_modifier)
            } else {
                rend_modifier.to_string()
            }
        )
    } else {
        format!("Save ({}+)", save_target)
    };

    let save_phase = PhaseResult {
        phase: Phase::Save,
        rolls: save_rolls,
        successes: unsaved,
        failures: total_wounds.saturating_sub(unsaved),
        total_output: unsaved,
        auto_fails,
        skipped: false,
        description: save_desc,
        variance_step: None,
    };

    // Phase 4: Damage
    let (normal_damage, damage_rolls, damage_variance) = if unsaved > 0 {
        if has_dice(&effective_damage) {
            let per_wound = effective_damage.clone();
            let results: Vec<u8> = (0..unsaved)
                .map(|_| parse_dice_string(&per_wound).unwrap_or(1) as u8)
                .collect();
            let total: usize = results.iter().map(|&x| x as usize).sum();
            let variance = Some(crate::combat::types::VarianceStep::DamageRoll {
                per_wound,
                results: results.clone(),
                total,
            });
            let dice_rolls = results
                .into_iter()
                .map(|v| DiceRoll {
                    value: v,
                    success: true,
                    is_crit: false,
                })
                .collect();
            (total, dice_rolls, variance)
        } else {
            let fixed_dmg = parse_dice_string(&effective_damage).unwrap_or(1) as usize;
            let total = unsaved * fixed_dmg;
            let dice_rolls = vec![
                DiceRoll {
                    value: fixed_dmg as u8,
                    success: true,
                    is_crit: false,
                };
                unsaved
            ];
            (total, dice_rolls, None)
        }
    } else {
        (0, Vec::new(), None)
    };

    let total_damage = normal_damage + mortal_wounds_from_crits;

    let damage_desc = if damage_modifier != 0 {
        format!(
            "Damage ({} → {}) per wound ({} damage)",
            weapon.damage,
            effective_damage,
            if damage_modifier > 0 {
                format!("+{}", damage_modifier)
            } else {
                damage_modifier.to_string()
            }
        )
    } else {
        format!("Damage ({} per wound)", weapon.damage)
    };

    let damage_phase = PhaseResult {
        phase: Phase::Damage,
        rolls: damage_rolls,
        successes: total_damage,
        failures: 0,
        total_output: total_damage,
        auto_fails: false,
        skipped: false,
        description: damage_desc,
        variance_step: damage_variance,
    };

    let mut phases = vec![hit_phase, wound_phase, save_phase, damage_phase];

    // Phase 5: Ward
    let mut final_damage = total_damage;
    let mut ward_phase = None;

    if include_ward && total_damage > 0 {
        if let Some(ward_target) = defender.ward {
            let ward_result = resolve_ward(total_damage, ward_target, None);
            final_damage = ward_result.final_damage;

            ward_phase = Some(PhaseResult {
                phase: Phase::Ward,
                rolls: ward_result.rolls,
                successes: ward_result.wounds_saved,
                failures: total_damage - ward_result.wounds_saved,
                total_output: final_damage,
                auto_fails: false,
                skipped: false,
                description: format!("Ward ({}+)", ward_target),
                variance_step: None,
            });
        }
    }

    if let Some(wp) = ward_phase {
        phases.push(wp);
    }

    CombatResult {
        attacker_name: attacker.name.clone(),
        weapon_name: weapon.name.clone(),
        defender_name: defender.name.clone(),
        phases,
        final_damage,
        mortal_wounds: mortal_wounds_from_crits,
        stopped_after_wound: false,
        total_hits: 0,
        total_wounds: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_weapon() -> Weapon {
        Weapon {
            name: "Test Weapon".into(),
            range: None,
            attack: "5".into(),
            to_hit: 3,
            to_wound: 4,
            rend: 0,
            damage: "1".into(),
            crit_hit: None,
        }
    }

    fn test_defender(save: u8, ward: Option<u8>) -> Unit {
        Unit {
            id: "defender".into(),
            name: "Test Defender".into(),
            faction: "Test".into(),
            save,
            ward,
            weapons: vec![],
        }
    }

    fn test_attacker() -> Unit {
        Unit {
            id: "attacker".into(),
            name: "Test Attacker".into(),
            faction: "Test".into(),
            save: 4,
            ward: None,
            weapons: vec![test_weapon()],
        }
    }

    #[test]
    fn simple_attack_hit_phase() {
        let weapon = test_weapon();
        let (hits, auto_wounds, extra_hits, mortal_wounds, rolls) =
            resolve_hits(&weapon, 5, 3, Some(&[4, 3, 2, 5, 6]));
        assert_eq!(hits, 4); // 3, 4, 5, 6 all hit (3+)
        assert_eq!(auto_wounds, 0);
        assert_eq!(extra_hits, 0);
        assert_eq!(mortal_wounds, 0);
        assert_eq!(rolls.len(), 5);
    }

    #[test]
    fn calculate_save_target_basic() {
        // Rend is stored as negative values (-1, -2, etc.)
        // Save 4+ with rend -1 = need 5+ to save (harder)
        assert_eq!(calculate_save_target(4, 0, 0), 4);
        assert_eq!(calculate_save_target(4, -1, 0), 5);
        assert_eq!(calculate_save_target(4, -3, 0), 7);
    }

    #[test]
    fn save_auto_fails_when_target_exceeds_6() {
        let save_target = calculate_save_target(4, -3, 0);
        assert!(save_target > 6);
    }

    #[test]
    fn extreme_rend_all_wounds_pass() {
        let save_target = calculate_save_target(3, -5, 0);
        assert!(save_target > 6);

        let (unsaved, rolls, auto_fails) = resolve_save(save_target, 5, Some(&[]));
        assert_eq!(unsaved, 5);
        assert!(auto_fails);
        assert_eq!(rolls.len(), 0);
    }

    #[test]
    fn weapon_with_zero_rend_normal_save() {
        let save_target = calculate_save_target(4, 0, 0);
        assert_eq!(save_target, 4);

        let (unsaved, _rolls, auto_fails) = resolve_save(save_target, 3, Some(&[3, 4, 5]));
        assert!(!auto_fails);
        assert_eq!(unsaved, 1); // Only 3 fails (below 4)
    }

    #[test]
    fn ward_can_save_all_damage() {
        let result = resolve_ward(1, 6, Some(&[6]));
        assert_eq!(result.final_damage, 0);
        assert_eq!(result.wounds_saved, 1);
    }

    #[test]
    fn ward_partial_save() {
        let result = resolve_ward(3, 4, Some(&[1, 4, 6]));
        assert_eq!(result.final_damage, 1);
        assert_eq!(result.wounds_saved, 2);
    }

    #[test]
    fn crit_auto_wound_skips_wound_roll() {
        let mut weapon = test_weapon();
        weapon.crit_hit = Some(CritEffect::AutoWound);

        let (hits, auto_wounds, _extra_hits, _mortal_wounds, _rolls) =
            resolve_hits(&weapon, 5, 3, Some(&[4, 3, 2, 5, 6]));

        // 6 = auto-wound (not counted as normal hit)
        assert_eq!(hits, 3); // 3, 4, 5 = 3 normal hits
        assert_eq!(auto_wounds, 1); // 6 = auto-wound
    }

    #[test]
    fn crit_mortal_wounds_bypass_save() {
        let mut weapon = test_weapon();
        weapon.crit_hit = Some(CritEffect::MortalWounds(Some("2".into())));

        let (_hits, _auto_wounds, _extra_hits, mortal_wounds, _rolls) =
            resolve_hits(&weapon, 5, 3, Some(&[4, 3, 2, 5, 6]));

        assert_eq!(mortal_wounds, 2);
    }

    #[test]
    fn crit_mortal_wounds_none_deals_zero() {
        let mut weapon = test_weapon();
        weapon.crit_hit = Some(CritEffect::MortalWounds(None));

        let (hits, _auto_wounds, _extra_hits, mortal_wounds, _rolls) =
            resolve_hits(&weapon, 5, 3, Some(&[4, 3, 2, 5, 6]));

        // 6 is a crit mortal wound (no bonus value) and does not add a normal hit
        assert_eq!(hits, 3); // 4, 3, 5 = 3 normal hits
        assert_eq!(mortal_wounds, 0); // None means no bonus mortal wounds
    }

    #[test]
    fn crit_mortal_wounds_multiple_sixes() {
        let mut weapon = test_weapon();
        weapon.crit_hit = Some(CritEffect::MortalWounds(Some("2".into())));

        // Three 6s should generate 3 * 2 = 6 mortal wounds
        let (_hits, _auto_wounds, _extra_hits, mortal_wounds, rolls) =
            resolve_hits(&weapon, 5, 3, Some(&[6, 6, 6, 4, 5]));

        assert_eq!(mortal_wounds, 6); // 3 sixes * 2 damage each
        assert_eq!(rolls.len(), 5);
        // Verify all three 6s are marked as crits
        let crit_rolls: Vec<_> = rolls.iter().filter(|r| r.is_crit).collect();
        assert_eq!(crit_rolls.len(), 3);
    }

    #[test]
    fn crit_mortal_wounds_dice_expression() {
        let mut weapon = test_weapon();
        // Crit with dice expression "D3" - should parse and roll
        weapon.crit_hit = Some(CritEffect::MortalWounds(Some("D3".into())));

        // Use deterministic rolls: 6 (crit) + 4 (normal hit) = 1 D3 roll + 1 normal hit
        let (hits, _auto_wounds, _extra_hits, _mortal_wounds, rolls) =
            resolve_hits(&weapon, 2, 3, Some(&[6, 4]));

        // 6 = crit with D3 mortal wounds (1-3), 4 = normal hit
        assert_eq!(hits, 1);
        // mortal_wounds is between 1-3 (the parsed D3 result)
        // We can't assert exact value since D3 is random, but verify crit is tracked
        let crit_rolls: Vec<_> = rolls.iter().filter(|r| r.is_crit).collect();
        assert_eq!(crit_rolls.len(), 1);
        assert!(crit_rolls[0].success); // Crit should succeed
    }

    #[test]
    fn crit_mortal_wounds_d6_plus_modifier() {
        let mut weapon = test_weapon();
        // Crit with "D6+1" should parse correctly
        weapon.crit_hit = Some(CritEffect::MortalWounds(Some("D6+1".into())));

        let (_hits, _auto_wounds, _extra_hits, _mortal_wounds, rolls) =
            resolve_hits(&weapon, 2, 3, Some(&[6, 5]));

        // 6 = crit, 5 = normal hit
        // D6+1 result is 2-7 (D6 is 1-6, plus 1)
        let crit_rolls: Vec<_> = rolls.iter().filter(|r| r.is_crit).collect();
        assert_eq!(crit_rolls.len(), 1);
    }

    #[test]
    fn crit_sixes_marked_as_crit_in_rolls() {
        let mut weapon = test_weapon();
        weapon.crit_hit = Some(CritEffect::AutoWound);

        let (_hits, _auto_wounds, _extra_hits, _mortal_wounds, rolls) =
            resolve_hits(&weapon, 5, 3, Some(&[6, 5, 4, 3, 2]));

        assert_eq!(rolls.len(), 5);
        // Only the 6 should be marked as crit
        assert!(rolls[0].is_crit); // 6 is crit
        assert!(!rolls[1].is_crit); // 5 is not
        assert!(!rolls[2].is_crit); // 4 is not
        assert!(!rolls[3].is_crit); // 3 is not
        assert!(!rolls[4].is_crit); // 2 is not (and misses)
    }

    #[test]
    fn crit_mortal_wounds_invalid_dice_defaults_to_one() {
        let mut weapon = test_weapon();
        // Crit with invalid dice string - should default to 1
        weapon.crit_hit = Some(CritEffect::MortalWounds(Some("INVALID".into())));

        let (_hits, _auto_wounds, _extra_hits, mortal_wounds, rolls) =
            resolve_hits(&weapon, 1, 3, Some(&[6]));

        // Invalid dice string should default to 1 mortal wound
        assert_eq!(mortal_wounds, 1);
        assert!(rolls[0].is_crit);
        assert!(rolls[0].success);
    }

    #[test]
    fn crit_mortal_wounds_empty_string_defaults_to_one() {
        let mut weapon = test_weapon();
        // Crit with empty string - should default to 1
        weapon.crit_hit = Some(CritEffect::MortalWounds(Some("".into())));

        let (_hits, _auto_wounds, _extra_hits, mortal_wounds, rolls) =
            resolve_hits(&weapon, 1, 3, Some(&[6]));

        // Empty string fails to parse as valid dice, should default to 1
        assert_eq!(mortal_wounds, 1);
        assert!(rolls[0].is_crit);
    }

    #[test]
    fn stop_after_wound_with_crit_mortal_wounds_none() {
        let mut weapon = test_weapon();
        weapon.crit_hit = Some(CritEffect::MortalWounds(None));
        weapon.attack = "1".into();

        let attacker = test_attacker();
        let defender = test_defender(4, None);

        let result = resolve_combat(
            &attacker,
            &defender,
            &weapon,
            1,
            false,
            false,
            0,
            false,
            true, // stop_after_wound
            0,
            0,
            0,
            0,
            Some(&[6]), // One crit roll
        );

        assert!(result.stopped_after_wound);
        // Crit with None mortal wounds - no bonus damage
        assert_eq!(result.mortal_wounds, 0);
        assert_eq!(result.total_hits, 0); // Crit doesn't count as hit
        assert_eq!(result.total_wounds, 0); // Crit doesn't wound
        assert_eq!(result.final_damage, 0);
    }

    #[test]
    fn crit_extra_hit_generates_two_hits() {
        let mut weapon = test_weapon();
        weapon.crit_hit = Some(CritEffect::ExtraHit);

        let (hits, _auto_wounds, extra_hits, _mortal_wounds, _rolls) =
            resolve_hits(&weapon, 5, 3, Some(&[4, 3, 2, 5, 6]));

        // 6 = extra hit: 1 base hit + 1 extra
        assert_eq!(hits, 4); // 3, 4, 5, and 6 base = 4 hits
        assert_eq!(extra_hits, 1); // 1 extra from 6
    }

    #[test]
    fn full_combat_sequence() {
        let attacker = test_attacker();
        let defender = test_defender(4, None);
        let weapon = test_weapon();

        let result = resolve_combat(
            &attacker,
            &defender,
            &weapon,
            1,
            false,
            false,
            0,
            false,
            false,
            0,
            0,
            0,
            0,
            Some(&[4, 5, 6, 3, 2]),
        );

        assert_eq!(result.phases.len(), 4); // No ward
        assert_eq!(result.phases[0].phase, Phase::Hit);
        assert_eq!(result.phases[1].phase, Phase::Wound);
        assert_eq!(result.phases[2].phase, Phase::Save);
        assert_eq!(result.phases[3].phase, Phase::Damage);
    }

    #[test]
    fn mortal_wounds_bypass_save_but_not_ward() {
        let mut weapon = test_weapon();
        weapon.crit_hit = Some(CritEffect::MortalWounds(Some("2".into())));
        weapon.attack = "1".into();

        let attacker = test_attacker();
        let defender = test_defender(4, Some(5));

        let result = resolve_combat(
            &attacker,
            &defender,
            &weapon,
            1,
            false,
            false,
            0,
            true,
            false,
            0,
            0,
            0,
            0,
            Some(&[6]),
        );

        // Mortal wounds should go straight to damage
        assert_eq!(result.mortal_wounds, 2);
        // With ward, final damage could be less
        assert!(result.phases.len() == 5); // Includes ward phase
        assert_eq!(result.phases[4].phase, Phase::Ward);
    }

    #[test]
    fn resolve_combat_with_attack_override() {
        let weapon = Weapon {
            name: "Override Test Weapon".into(),
            range: None,
            attack: "3".into(),
            to_hit: 1,
            to_wound: 1,
            rend: -10,
            damage: "1".into(),
            crit_hit: None,
        };
        let attacker = test_attacker();
        let defender = test_defender(4, None);

        // Override to exactly 25 attacks
        let result = resolve_combat(
            &attacker,
            &defender,
            &weapon,
            5,
            false,
            true,
            25,
            false,
            false,
            0,
            0,
            0,
            0,
            Some(&[1; 25]),
        );

        assert_eq!(result.phases[0].rolls.len(), 25);
        let desc = &result.phases[0].description;
        assert!(desc.contains("25 fixed attacks"));
        assert!(!desc.contains("models"));
    }

    #[test]
    fn resolve_combat_with_champion() {
        let weapon = Weapon {
            name: "Champion Test Weapon".into(),
            range: None,
            attack: "3".into(),
            to_hit: 1, // Auto-hit
            to_wound: 1,
            rend: -10,
            damage: "1".into(),
            crit_hit: None,
        };
        let attacker = test_attacker();
        let defender = test_defender(4, None);

        // 5 models × 3 + 1 champion = 16 attacks
        let result = resolve_combat(
            &attacker,
            &defender,
            &weapon,
            5,
            true,
            false,
            0,
            false,
            false,
            0,
            0,
            0,
            0,
            Some(&[1; 16]),
        );

        assert_eq!(result.phases[0].rolls.len(), 16);
        let desc = &result.phases[0].description;
        assert!(desc.contains("+ 1 champion"));
        assert!(desc.contains("16 total"));
    }

    #[test]
    fn champion_ignored_when_attack_override_enabled() {
        let weapon = Weapon {
            name: "Champion Override Test Weapon".into(),
            range: None,
            attack: "3".into(),
            to_hit: 1,
            to_wound: 1,
            rend: -10,
            damage: "1".into(),
            crit_hit: None,
        };
        let attacker = test_attacker();
        let defender = test_defender(4, None);

        // Champion + override = override wins, exactly 20 attacks
        let result = resolve_combat(
            &attacker,
            &defender,
            &weapon,
            5,
            true,
            true,
            20,
            false,
            false,
            0,
            0,
            0,
            0,
            Some(&[1; 20]),
        );

        assert_eq!(result.phases[0].rolls.len(), 20);
        assert!(result.phases[0].description.contains("20 fixed attacks"));
    }

    #[test]
    fn stop_after_wound_skips_save_damage_ward() {
        let attacker = test_attacker();
        let defender = test_defender(4, Some(5));
        let weapon = test_weapon();

        let result = resolve_combat(
            &attacker,
            &defender,
            &weapon,
            1,
            false,
            false,
            0,
            true,
            true,
            0,
            0,
            0,
            0,
            Some(&[4, 5, 6, 3, 2]),
        );

        assert_eq!(result.phases.len(), 5);
        assert!(!result.phases[0].skipped);
        assert!(!result.phases[1].skipped);
        assert!(result.phases[2].skipped);
        assert!(result.phases[3].skipped);
        assert!(result.phases[4].skipped);
        assert!(result.stopped_after_wound);
        assert_eq!(result.final_damage, 0);
    }

    #[test]
    fn stop_after_wound_tracks_totals() {
        let weapon = Weapon {
            name: "Test Weapon".into(),
            range: None,
            attack: "5".into(),
            to_hit: 3,
            to_wound: 4,
            rend: 0,
            damage: "1".into(),
            crit_hit: None,
        };
        let attacker = test_attacker();
        let defender = test_defender(4, None);

        let result = resolve_combat(
            &attacker,
            &defender,
            &weapon,
            1,
            false,
            false,
            0,
            false,
            true,
            0,
            0,
            0,
            0,
            Some(&[4, 5, 6, 3, 2]),
        );

        assert!(result.stopped_after_wound);
        // Rolls: 4,5,6,3 hit (3+); 2 misses = 4 total hits
        assert_eq!(result.total_hits, 4);
        assert_eq!(result.total_wounds, result.phases[1].total_output);
        assert_eq!(result.final_damage, 0);
    }

    #[test]
    fn stop_after_wound_with_crit_auto_wound() {
        let mut weapon = test_weapon();
        weapon.crit_hit = Some(CritEffect::AutoWound);

        let attacker = test_attacker();
        let defender = test_defender(4, None);

        let result = resolve_combat(
            &attacker,
            &defender,
            &weapon,
            1,
            false,
            false,
            0,
            false,
            true,
            0,
            0,
            0,
            0,
            Some(&[1, 2, 3, 4, 6]),
        );

        assert!(result.stopped_after_wound);
        // Rolls: 1,2 miss; 3,4 = 2 normal hits; 6 = 1 auto-wound = 3 total hits
        assert_eq!(result.total_hits, 3);
    }

    #[test]
    fn stop_after_wound_with_crit_mortal_wounds() {
        let mut weapon = test_weapon();
        weapon.crit_hit = Some(CritEffect::MortalWounds(Some("2".into())));
        weapon.attack = "1".into();

        let attacker = test_attacker();
        let defender = test_defender(4, None);

        let result = resolve_combat(
            &attacker,
            &defender,
            &weapon,
            1,
            false,
            false,
            0,
            false,
            true,
            0,
            0,
            0,
            0,
            Some(&[6]),
        );

        assert!(result.stopped_after_wound);
        // Mortal wounds do not count as hits for wound rolls
        assert_eq!(result.total_hits, 0);
        assert_eq!(result.total_wounds, 0);
        assert_eq!(result.mortal_wounds, 2);
        assert_eq!(result.final_damage, 0);
    }

    #[test]
    fn hit_modifier_reduces_target() {
        let weapon = Weapon {
            name: "Test Weapon".into(),
            range: None,
            attack: "5".into(),
            to_hit: 3,
            to_wound: 4,
            rend: 0,
            damage: "2".into(),
            crit_hit: None,
        };
        let attacker = test_attacker();
        let defender = test_defender(4, None);

        let result = resolve_combat(
            &attacker,
            &defender,
            &weapon,
            1,
            false,
            false,
            0,
            false,
            true,                   // stop_after_wound
            1,                      // hit_modifier: +1
            0,                      // wound_modifier
            0,                      // rend_modifier
            0,                      // damage_modifier
            Some(&[2, 3, 4, 5, 6]), // All 5 hit on 2+
        );

        // With hit_modifier: +1, effective to_hit = 2+ (was 3+)
        // Rolls [2, 3, 4, 5, 6] all >= 2, so 5 hits
        assert_eq!(result.total_hits, 5);
    }

    #[test]
    fn hit_modifier_increases_target() {
        let weapon = Weapon {
            name: "Test Weapon".into(),
            range: None,
            attack: "4".into(),
            to_hit: 3,
            to_wound: 4,
            rend: 0,
            damage: "1".into(),
            crit_hit: None,
        };
        let attacker = test_attacker();
        let defender = test_defender(4, None);

        let result = resolve_combat(
            &attacker,
            &defender,
            &weapon,
            1,
            false,
            false,
            0,
            false,
            true,                // stop_after_wound
            -1,                  // hit_modifier: -1
            0,                   // wound_modifier
            0,                   // rend_modifier
            0,                   // damage_modifier
            Some(&[3, 4, 5, 6]), // Only [4, 5, 6] hit on 4+
        );

        // With hit_modifier: -1, effective to_hit = 4+ (was 3+)
        // Rolls [3, 4, 5, 6], only [4, 5, 6] >= 4, so 3 hits
        assert_eq!(result.total_hits, 3);
    }

    #[test]
    fn wound_modifier_reduces_target() {
        // Test that wound_modifier affects the effective to_wound value
        // Use to_wound: 1 (auto-wound) so all hits wound regardless of rolls
        let weapon = Weapon {
            name: "Test Weapon".into(),
            range: None,
            attack: "4".into(),
            to_hit: 3,
            to_wound: 1, // Auto-wound, so all hits succeed
            rend: 0,
            damage: "1".into(),
            crit_hit: None,
        };
        let attacker = test_attacker();
        let defender = test_defender(4, None);

        // With wound_modifier: +1, wound description should show modified target
        let result = resolve_combat(
            &attacker,
            &defender,
            &weapon,
            1,
            false,
            false,
            0,
            false,
            true,                // stop_after_wound
            0,                   // hit_modifier
            1,                   // wound_modifier: +1
            0,                   // rend_modifier
            0,                   // damage_modifier
            Some(&[3, 4, 5, 6]), // 4 hits on 3+
        );

        // All 4 hits wound (to_wound is 1, so all wounds succeed)
        assert_eq!(result.total_hits, 4);
        assert_eq!(result.total_wounds, 4);
        // Verify wound description shows modifier
        assert!(result.phases[1].description.contains("+1"));
        assert!(result.phases[1].description.contains("1+"));
    }

    #[test]
    fn rend_modifier_affects_save() {
        // rend_modifier: +1 on rend: -1 with save: 4+ should produce save target 6+
        // Formula: defender_save - weapon_rend + rend_modifier
        // Calculation: 4 - (-1) + 1 = 6
        assert_eq!(calculate_save_target(4, -1, 1), 6);
        // rend_modifier: +2 on rend: -1 with save: 4+ should produce save target 7+ (auto-fail)
        assert_eq!(calculate_save_target(4, -1, 2), 7);
        // rend_modifier: -1 on rend: -1 with save: 4+ should produce save target 4+
        assert_eq!(calculate_save_target(4, -1, -1), 4);
    }

    #[test]
    fn damage_modifier_on_flat_damage() {
        // Use to_wound: 1 (auto-wound) so all hits wound regardless of random rolls
        let weapon = Weapon {
            name: "Test Weapon".into(),
            range: None,
            attack: "3".into(),
            to_hit: 3,
            to_wound: 1, // Auto-wound
            rend: -10,   // Auto-fail saves
            damage: "2".into(),
            crit_hit: None,
        };
        let attacker = test_attacker();
        let defender = test_defender(4, None);

        let result = resolve_combat(
            &attacker,
            &defender,
            &weapon,
            1,
            false,
            false,
            0,
            false,
            false,            // Process full combat
            0,                // hit_modifier
            0,                // wound_modifier
            0,                // rend_modifier
            2,                // damage_modifier: +2
            Some(&[3, 4, 5]), // 3 hits on 3+
        );

        // 3 hits, 3 wounds (auto-wound), rend -10 auto-fails saves = 3 unsaved wounds
        // damage_modifier: +2 on damage "2" = 4 per wound
        // Total: 3 * 4 = 12
        assert_eq!(result.final_damage, 12);
    }

    #[test]
    fn damage_modifier_on_dice_damage() {
        // apply_damage_modifier("D3", 2) should return "D3+2"
        assert_eq!(apply_damage_modifier("D3", 2), "D3+2");
    }

    #[test]
    fn apply_damage_modifier_on_numeric_negative() {
        // apply_damage_modifier("2", -1) should return "1"
        assert_eq!(apply_damage_modifier("2", -1), "1");
    }

    #[test]
    fn hit_modifier_clamped_at_1() {
        let weapon = Weapon {
            name: "Test Weapon".into(),
            range: None,
            attack: "5".into(),
            to_hit: 3,
            to_wound: 4,
            rend: 0,
            damage: "1".into(),
            crit_hit: None,
        };
        let attacker = test_attacker();
        let defender = test_defender(4, None);

        let result = resolve_combat(
            &attacker,
            &defender,
            &weapon,
            1,
            false,
            false,
            0,
            false,
            true,                   // stop_after_wound
            5,                      // hit_modifier: +5
            0,                      // wound_modifier
            0,                      // rend_modifier
            0,                      // damage_modifier
            Some(&[2, 3, 4, 5, 6]), // All 5 should hit (clamped to 1+)
        );

        // With hit_modifier: +5, effective to_hit = max(1, 3-5) = 1
        // All rolls >= 1, so all 5 hit
        assert_eq!(result.total_hits, 5);
    }
}
