# AoS4 Combat Roller

A Rust-based desktop application with a GUI for visualizing combat sequence rolls in **Age of Sigmar 4th Edition**.

## Features

- **Full Combat Sequence Visualization**: Hit → Wound → Save → Damage → Ward
- **Unit & Weapon Profiles**: Pre-loaded with sample units from major factions
- **Dice Roll Display**: Color-coded results (green = success, red = fail, gold = crit)
- **Combat Log**: History of all previous rolls
- **Critical Hit Support**: Auto-wound, Extra Hit, and Mortal Wounds
- **Ward Saves**: Optional ward phase per unit profile
- **Stop After Wound**: Checkbox to stop the combat sequence after Hit and Wound phases, allowing the defender to roll saves externally (useful for in-person games where each player rolls their own dice)

## Supported Factions

- **Skaven**
- **Nighthaunt**
- **Kruleboyz**
- **Kharadron Overlords**
- **Slaves to Darkness**
- **Sylvaneth**

## How to Run

```bash
cargo run --release
```

## How to Test

```bash
cargo test
cargo clippy
```

## Adding Custom Units

Edit `resources/units.json` to add your own units and weapons:

```json
{
  "units": [
    {
      "id": "my_unit",
      "name": "My Unit",
      "faction": "My Faction",
      "save": 4,
      "ward": 5,
      "weapons": [
        {
          "name": "My Weapon",
          "range": null,
          "attack": "D6",
          "to_hit": 3,
          "to_wound": 4,
          "rend": -1,
          "damage": "2",
          "crit_hit": { "type": "auto_wound" }
        }
      ]
    }
  ]
}
```

## Converting Faction Data

Faction data can be converted using the schema defined in [`.opencode/extraction/SCHEMA.md`](.opencode/extraction/SCHEMA.md). Intermediate JSON files go in `.opencode/extraction/units/`, named after each faction (e.g., `skaven.json`, `nighthaunt.json`). The `tools/convert` workspace member merges them into `resources/units.json`.

Example:

```bash
cargo run -p convert -- --input-dir .opencode/extraction/units --output resources/units.json
```

## Combat Rules Implemented

1. **Hit**: Roll D6 per attack. 6 = critical hit (weapon-specific effect).
2. **Wound**: Roll D6 per hit. Target is the weapon's `to_wound` value.
3. **Save**: Roll D6 per wound. Target is `defender.save - weapon.rend`. If target > 6, auto-fail.
4. **Damage**: Unsaved wounds × weapon damage.
5. **Ward** (optional): D6 per damage point. Target is defender's ward value.

### Critical Hit Effects

- **Auto-Wound**: Wound roll auto-succeeds, proceeds to Save
- **Extra Hit**: Counts as 2 hits (both resolve normally)
- **Mortal Wounds**: Skip Wound/Save phases, deal damage directly (still subject to Ward)

## License

MIT
