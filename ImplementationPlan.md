# AoS4 Combat Roller - Implementation Plan

Age of Sigmar 4th edition combat sequence roller with GUI visualization.

---

## Overview

A Rust desktop application using `egui`/`eframe` to visualize full Age of Sigmar 4th edition combat sequences with unit profiles, weapon selection, and animated dice roll visualization.

---

## Combat Sequence Rules

| Phase | Input | Roll | Target | Output |
|-------|-------|------|--------|--------|
| **1. Hit** | Attacks × D6 | Hit roll | Weapon's `to_hit` | Hits + Crit effects |
| **2. Wound** | Hits × D6 | Wound roll | Weapon's `to_wound` | Wounds |
| **3. Save** | Wounds × D6 | Save roll | `Defender.save - weapon.rend` (>6 = auto-fail) | Saved/Damaged |
| **4. Damage** | Damaged × weapon damage | Resolve | Auto-calculate | Wounds to allocate |
| **5. Ward** *(optional)* | Damage points × D6 | Ward roll | Defender's ward target | Final wounds dealt |

### Crit Hit Resolution

| Crit Type | Effect |
|-----------|--------|
| **Auto-Wound** | Wound roll auto-succeeds (continue to Save → Damage) |
| **Extra Hit** | Counts as 2 hits (resolve normally) |
| **Mortal Wounds** | Skip to Damage phase (no wound/save), bypass save, apply ward if present |

### Key Rules
- **Save Auto-Fail**: If `(defender.save - weapon.rend) > 6` → auto-fail without rolling
- **Mortal Wounds**: Deal damage directly, skipping wound/save phases but NOT ward
- **Ward Saves**: D6 per damage point, ward target value to prevent damage

---

## Project Structure

```
aos4-combat-roller/
├── Cargo.toml
├── src/
│   ├── main.rs              # eframe entry point
│   ├── app.rs               # App state, UI orchestration
│   ├── combat/
│   │   ├── mod.rs
│   │   ├── dice.rs          # D6/D3 rolling, string parsing
│   │   ├── engine.rs        # Full combat resolution
│   │   └── types.rs         # RollResult, CombatPhase enums
│   ├── data/
│   │   ├── mod.rs
│   │   ├── models.rs        # Unit, Weapon, CritEffect structs
│   │   └── loader.rs        # JSON loading with error handling
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── panels/
│   │   │   ├── unit_panel.rs    # Unit/weapon selection (multi-select)
│   │   │   ├── target_panel.rs  # Defender profile (save, ward)
│   │   │   ├── combat_view.rs   # Animated combat display
│   │   │   └── log_panel.rs     # Scrollable combat history
│   │   └── widgets/
│   │       ├── dice_display.rs  # Animated dice with color coding
│   │       └── phase_result.rs  # Phase summary card
│   └── resources/
│       └── units.json           # Sample profiles
└── resources/
    └── units.json
```

---

## Data Models

```rust
enum CritEffect {
    AutoWound,              // Wound auto-succeeds
    ExtraHit,               // Count as 2 hits
    MortalWounds(String),   // Skip to damage (e.g., "D6" or "2")
}

struct Weapon {
    name: String,
    range: Option<String>,  // None = melee
    attacks: String,        // "3", "D6", "2D6"
    to_hit: u8,              // e.g., 4 for 4+
    to_wound: u8,            // e.g., 3 for 3+
    rend: i8,                // -1, -2, or 0
    damage: String,          // "1", "D6", "D3+2"
    crit_hit: Option<CritEffect>,
}

struct Unit {
    id: String,
    name: String,
    faction: String,
    weapons: Vec<Weapon>,
    save: u8,                // e.g., 4 for 4+
    ward: Option<u8>,        // e.g., Some(5) for 5+ ward, None = no ward
}
```

---

## Testing Framework

| Component | Tool | Purpose |
|-----------|------|---------|
| **Unit Tests** | Built-in `#[test]` | Core combat logic, dice rolling, parsing |
| **Property Tests** | `proptest` crate | Random attack/damage distributions |
| **Integration Tests** | `tests/` directory | Full combat sequences |
| **Doc Tests** | `///` comments | Ensure documentation examples work |

### Pre-commit Hook

```bash
#!/bin/sh
cargo test --quiet
if [ $? -ne 0 ]; then
    echo "❌ Tests failed. Commit aborted."
    exit 1
fi
cargo clippy --quiet -D warnings
if [ $? -ne 0 ]; then
    echo "❌ Clippy warnings found. Commit aborted."
    exit 1
fi
echo "✅ Tests and clippy passed"
```

---

## Mandatory Unit Tests

### 1. Dice Rolling Tests
- `roll_d6_returns_values_between_1_and_6`
- `roll_multiple_d6_returns_correct_count`

### 2. Dice Parsing Tests
- `parse_attacks_fixed`, `parse_attacks_d6`, `parse_attacks_2d6`
- `parse_damage_with_modifier`
- `parse_invalid_string_returns_error`

### 3. Combat Engine Tests
- `simple_attack_hit_phase`
- **Edge Case**: `ward_can_save_all_damage`
- **Edge Case**: `ward_partial_save`
- **Edge Case**: `save_auto_fails_when_target_exceeds_6`
- **Edge Case**: `extreme_rend_all_wounds_pass`
- **Edge Case**: `weapon_with_zero_rend_normal_save`
- `crit_auto_wound_skips_wound_roll`
- `crit_mortal_wounds_bypass_save`
- `crit_extra_hit_generates_two_hits`
- `ward_saves_prevent_damage`
- `mortal_wounds_bypass_save_but_not_ward`

### 4. Integration Tests
- `full_combat_sequence_stormcast_vs_orruks`
- `ward_saving_all_damage_test`
- `extreme_rend_no_save_needed`

### 5. JSON Loading Tests
- `load_valid_unit_json`
- `load_units_from_file`
- `unit_with_crit_weapon_parses`
- `unit_with_ward_parses`

---

## Dependencies

```toml
[dependencies]
eframe = "0.29"        # egui GUI framework
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"     # JSON parsing
rand = "0.8"           # Random dice rolls
uuid = { version = "1.0", features = ["v4"] }
log = "0.4"            # Logging
env_logger = "0.11"    # Dev logging

[dev-dependencies]
proptest = "1.4"       # Property-based testing
tempfile = "3.10"       # Temp files for JSON loading tests
```

---

## UI Layout

```
┌─────────────────────────────────────────────────────────────────────┐
│  AoS4 Combat Roller                                    [⚙] [?]     │
├────────────────┬────────────────────────────────────────────────────┤
│                │                                                    │
│  ATTACKERS     │   COMBAT SEQUENCE                                  │
│  ──────────    │   ─────────────────────────────────────────────    │
│                │                                                    │
│  ☑ Megaboss    │   Hit (3+)     [⚅][⚅][⚅][⚅][⚅] → 4 hits        │
│  ☑ Brutes      │   │                                                       │
│  ☑ Savage      │   Wound (4+)  [⚅][⚅][⚅][⚅] → 3 wounds          │
│                │   │                                                       │
│  WEAPON        │   Save (3+)   [⚅][⚅][⚅] → 1 saved, 2 damage     │
│  ──────────    │   │                                                       │
│  ◎ Gore-choppa │   Damage (2)  2 × 2 = 4 wounds                    │
│  ○ Jagged Gore │   │                                                       │
│  ○ Boss Klaw   │   Ward (5+)   [⚅][⚅][⚅][⚅] → 1 saved           │
│                │   │                                                       │
│  ──────────    │   ████████                                         │
│                │   FINAL: 3 wounds dealt                             │
│  DEFENDER      │                                                    │
│  ──────────    │   ─────────────────────────────────────────────    │
│  ☑ Ironjawz    │                                                    │
│    Save: 4+    │   [  ROLL COMBAT  ]                                │
│    Ward: 5+    │                                                    │
│  ☑ Mortis      │                                                    │
│    Save: 3+    │                                                    │
│    Ward: None  │                                                    │
│                │                                                    │
├────────────────┴────────────────────────────────────────────────────┤
│  Combat Log: Swipe/scroll of all previous rolls                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Implementation Steps

1. Git repo setup
2. Add testing framework
3. Create Cargo.toml with dependencies
4. Create data models (Unit, Weapon, CritEffect)
5. Create JSON loader for unit profiles
6. Create dice rolling utilities with tests
7. Create combat engine with crit handling and tests
8. Create eframe app shell
9. Create unit selection panel
10. Create target panel
11. Create dice display widget with animations
12. Create combat view panel
13. Create combat log panel
14. Create sample unit data
15. Add pre-commit hook for tests

---

## Edge Cases Covered

1. **Ward saves all damage**: Tested with 6+ ward rolls
2. **Extreme rend**: Rend -5 vs save 3+ → target 8+ → all wounds auto-fail
3. **Mortal wounds**: Skip save, apply ward
4. **Zero rend**: Normal save mechanics

---

## Date: 2026-04-14
