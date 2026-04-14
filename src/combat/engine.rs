use crate::combat::dice::{parse_dice_string, roll_d6_batch};
use crate::combat::types::{CombatResult, DiceRoll, Phase, PhaseResult, WardResult};
use crate::data::models::{CritEffect, Unit, Weapon};

/// Calculate the save target number.
/// If the result is > 6, saves auto-fail.
pub fn calculate_save_target(defender_save: u8, weapon_rend: i8) -> u8 {
    let target = defender_save as i8 - weapon_rend;
    target.max(0) as u8
}

/// Resolve the hit phase for a given number of attacks.
/// Returns (hits, auto_wounds, extra_hits, mortal_wounds, rolls).
pub fn resolve_hits(
    weapon: &Weapon,
    attacks: usize,
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
        let is_hit = roll >= weapon.to_hit;

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
                Some(CritEffect::MortalWounds(ref dmg)) => {
                    dice.success = true;
                    let mw = parse_dice_string(dmg).unwrap_or(1) as usize;
                    mortal_wounds += mw;
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
    weapon: &Weapon,
    wounds_to_roll: usize,
    provided_rolls: Option<&[u8]>,
) -> (usize, Vec<DiceRoll>) {
    let rolls = match provided_rolls {
        Some(r) => r.to_vec(),
        None => roll_d6_batch(wounds_to_roll),
    };

    let mut successes = 0;
    let mut dice_rolls = Vec::with_capacity(rolls.len());

    for roll in rolls {
        let success = roll >= weapon.to_wound;
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

/// Resolve damage.
pub fn resolve_damage(weapon: &Weapon, unsaved_wounds: usize) -> (usize, Vec<DiceRoll>) {
    let mut total = 0;
    let mut rolls = Vec::new();

    for _ in 0..unsaved_wounds {
        let dmg = parse_dice_string(&weapon.damage).unwrap_or(1) as usize;
        total += dmg;
        rolls.push(DiceRoll {
            value: dmg as u8,
            success: true,
            is_crit: false,
        });
    }

    (total, rolls)
}

/// Resolve ward saves.
pub fn resolve_ward(
    damage: usize,
    ward_target: u8,
    provided_rolls: Option<&[u8]>,
) -> WardResult {
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

/// Resolve a full combat sequence.
pub fn resolve_combat(
    attacker: &Unit,
    defender: &Unit,
    weapon: &Weapon,
    include_ward: bool,
    provided_rolls: Option<&[u8]>, // For testing only
) -> CombatResult {
    // Determine number of attacks
    let attacks = parse_dice_string(&weapon.attacks).unwrap_or(1) as usize;

    // Phase 1: Hit
    let (hits, auto_wounds, extra_hits, mortal_wounds_from_crits, hit_rolls) =
        resolve_hits(weapon, attacks, provided_rolls);

    let hit_phase = PhaseResult {
        phase: Phase::Hit,
        rolls: hit_rolls,
        successes: hits + auto_wounds + extra_hits,
        failures: attacks.saturating_sub(hits + auto_wounds + extra_hits),
        total_output: hits + auto_wounds + extra_hits,
        auto_fails: false,
        skipped: false,
        description: format!("Hit ({}+)", weapon.to_hit),
    };

    // Phase 2: Wound
    let total_wounds_to_roll = hits + extra_hits;
    let (wounds, wound_rolls) = if total_wounds_to_roll > 0 {
        resolve_wounds(weapon, total_wounds_to_roll, None)
    } else {
        (0, Vec::new())
    };

    let total_wounds = wounds + auto_wounds;

    let wound_phase = PhaseResult {
        phase: Phase::Wound,
        rolls: wound_rolls,
        successes: wounds,
        failures: total_wounds_to_roll.saturating_sub(wounds),
        total_output: total_wounds,
        auto_fails: false,
        skipped: false,
        description: format!("Wound ({}+)", weapon.to_wound),
    };

    // Phase 3: Save
    let save_target = calculate_save_target(defender.save, weapon.rend);
    let (unsaved, save_rolls, auto_fails) = if total_wounds > 0 {
        resolve_save(save_target, total_wounds, None)
    } else {
        (0, Vec::new(), false)
    };

    let save_phase = PhaseResult {
        phase: Phase::Save,
        rolls: save_rolls,
        successes: unsaved,
        failures: total_wounds.saturating_sub(unsaved),
        total_output: unsaved,
        auto_fails,
        skipped: false,
        description: format!("Save ({}+)", save_target),
    };

    // Phase 4: Damage
    let (normal_damage, damage_rolls) = if unsaved > 0 {
        resolve_damage(weapon, unsaved)
    } else {
        (0, Vec::new())
    };

    let total_damage = normal_damage + mortal_wounds_from_crits;

    let damage_phase = PhaseResult {
        phase: Phase::Damage,
        rolls: damage_rolls,
        successes: total_damage,
        failures: 0,
        total_output: total_damage,
        auto_fails: false,
        skipped: false,
        description: format!("Damage ({} per wound)", weapon.damage),
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_weapon() -> Weapon {
        Weapon {
            name: "Test Weapon".into(),
            range: None,
            attacks: "5".into(),
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
            resolve_hits(&weapon, 5, Some(&[4, 3, 2, 5, 6]));
        assert_eq!(hits, 4); // 3, 4, 5, 6 all hit (3+)
        assert_eq!(auto_wounds, 0);
        assert_eq!(extra_hits, 0);
        assert_eq!(mortal_wounds, 0);
        assert_eq!(rolls.len(), 5);
    }

    #[test]
    fn calculate_save_target_basic() {
        assert_eq!(calculate_save_target(4, 0), 4);
        assert_eq!(calculate_save_target(4, -1), 5);
        assert_eq!(calculate_save_target(4, -3), 7);
    }

    #[test]
    fn save_auto_fails_when_target_exceeds_6() {
        let save_target = calculate_save_target(4, -3);
        assert!(save_target > 6);
    }

    #[test]
    fn extreme_rend_all_wounds_pass() {
        let save_target = calculate_save_target(3, -5);
        assert!(save_target > 6);

        let (unsaved, rolls, auto_fails) = resolve_save(save_target, 5, Some(&[]));
        assert_eq!(unsaved, 5);
        assert!(auto_fails);
        assert_eq!(rolls.len(), 0);
    }

    #[test]
    fn weapon_with_zero_rend_normal_save() {
        let save_target = calculate_save_target(4, 0);
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
            resolve_hits(&weapon, 5, Some(&[4, 3, 2, 5, 6]));

        // 6 = auto-wound (not counted as normal hit)
        assert_eq!(hits, 3); // 3, 4, 5 = 3 normal hits
        assert_eq!(auto_wounds, 1); // 6 = auto-wound
    }

    #[test]
    fn crit_mortal_wounds_bypass_save() {
        let mut weapon = test_weapon();
        weapon.crit_hit = Some(CritEffect::MortalWounds("2".into()));

        let (_hits, _auto_wounds, _extra_hits, mortal_wounds, _rolls) =
            resolve_hits(&weapon, 5, Some(&[4, 3, 2, 5, 6]));

        assert_eq!(mortal_wounds, 2);
    }

    #[test]
    fn crit_extra_hit_generates_two_hits() {
        let mut weapon = test_weapon();
        weapon.crit_hit = Some(CritEffect::ExtraHit);

        let (hits, _auto_wounds, extra_hits, _mortal_wounds, _rolls) =
            resolve_hits(&weapon, 5, Some(&[4, 3, 2, 5, 6]));

        // 6 = extra hit: 1 base hit + 1 extra
        assert_eq!(hits, 4); // 3, 4, 5, and 6 base = 4 hits
        assert_eq!(extra_hits, 1); // 1 extra from 6
    }

    #[test]
    fn full_combat_sequence() {
        let attacker = test_attacker();
        let defender = test_defender(4, None);
        let weapon = test_weapon();

        let result = resolve_combat(&attacker, &defender, &weapon, false, Some(&[4, 5, 6, 3, 2]));

        assert_eq!(result.phases.len(), 4); // No ward
        assert_eq!(result.phases[0].phase, Phase::Hit);
        assert_eq!(result.phases[1].phase, Phase::Wound);
        assert_eq!(result.phases[2].phase, Phase::Save);
        assert_eq!(result.phases[3].phase, Phase::Damage);
    }

    #[test]
    fn mortal_wounds_bypass_save_but_not_ward() {
        let mut weapon = test_weapon();
        weapon.crit_hit = Some(CritEffect::MortalWounds("2".into()));
        weapon.attacks = "1".into();

        let attacker = test_attacker();
        let defender = test_defender(4, Some(5));

        let result = resolve_combat(&attacker, &defender, &weapon, true, Some(&[6]));

        // Mortal wounds should go straight to damage
        assert_eq!(result.mortal_wounds, 2);
        // With ward, final damage could be less
        assert!(result.phases.len() == 5); // Includes ward phase
        assert_eq!(result.phases[4].phase, Phase::Ward);
    }
}
