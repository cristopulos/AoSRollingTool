# AoS4 Combat Roller

A Rust-based desktop application with a GUI for visualizing combat sequence rolls in **Age of Sigmar 4th Edition**.

## Features

- **Full Combat Sequence Visualization**: Hit → Wound → Save → Damage → Ward
- **Unit & Weapon Profiles**: Pre-loaded with sample units from major factions
- **Combat Breakdown Display**: A prominent final damage card at the top of the display shows total damage dealt, color-coded by percentile ranking (green = top 10%, red = bottom 25%). Each phase shows dice rolls with color-coded results (green = success, red = fail, gold = crit). Crit effect contributions are displayed inline within their respective phases:
  - Hit phase: extra hits shown inline (e.g., "11 base + 4 extra = 15")
  - Wound phase: auto-wounds shown inline (e.g., "5 normal + 2 extra = 7")
  - Damage phase: mortal wounds shown inline (e.g., "3 normal + 2 MW = 5")
  - **Large roll sets**: When displaying more than 20 dice, results are grouped (e.g., "4×6 3×5") to maintain readability. Groups are sorted by die value descending with color applied to the group based on the highest-priority result within.
- **Combat Log**: History of all previous rolls
- **Critical Hit Support**: Auto-wound, Extra Hit, and Mortal Wounds (with optional override dropdown)
- **Ward Saves**: Optional ward phase per unit profile
- **Stop After Wound**: Checkbox to stop the combat sequence after Hit and Wound phases, allowing the defender to roll saves externally (useful for in-person games where each player rolls their own dice).
- **Manual Defender**: When enabled, hides the unit list and lets you input save (1-7, default 4) and optional ward (1-6, default none) directly. A save value of 7 means "no save" (auto-fail). Works with both single rolls and 10,000-run simulations. The "Include Ward Saves" checkbox controls whether the ward phase is actually tested.
- **Monte Carlo Simulation**: Runs up to 10,000 simulations to display percentile ranking, mean, and quartile statistics. The damage distribution is visualized with a bar chart where each unique damage value gets its own bar (no grouping), with count labels above each bar. The chart includes vertical markers for your actual roll, and 25th/75th percentile lines for quartile reference.

## Supported Factions

Data is scraped from [Wahapedia](https://wahapedia.ru) AoS4 warscroll pages using the extraction pipeline in `aos4-extractor/`. The full faction list is defined in `aos4-extractor/factions.toml`.

**All 28 factions scraped (1126 units):**

| Grand Alliance | Factions |
|----------------|----------|
| **Order** (9) | Cities of Sigmar, Daughters of Khaine, Fyreslayers, Idoneth Deepkin, Kharadron Overlords, Lumineth Realm-lords, Seraphon, Stormcast Eternals, Sylvaneth |
| **Chaos** (7) | Beasts of Chaos, Blades of Khorne, Disciples of Tzeentch, Hedonites of Slaanesh, Helsmiths of Hashut, Maggotkin of Nurgle, Skaven, Slaves to Darkness |
| **Death** (4) | Flesh-eater Courts, Nighthaunt, Ossiarch Bonereapers, Soulblight Gravelords |
| **Destruction** (5) | Bonesplitterz, Gloomspite Gitz, Ironjawz, Kruleboyz, Ogor Mawtribes, Sons of Behemat |
| **Special** (1) | Endless Spells |

To re-scrape or add new factions, run the pipeline from `aos4-extractor/`:
```bash
cd aos4-extractor
python3 src/run_pipeline.py
```

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

## Combat Rules Implemented

1. **Hit**: Roll D6 per attack. 6 = critical hit (weapon-specific effect). Target can be modified by weapon modifiers.
2. **Wound**: Roll D6 per hit. Target is the weapon's `to_wound` value, adjustable via modifiers.
3. **Save**: Roll D6 per wound. Target is `defender.save - weapon.rend`. Modifiers can adjust effective rend. If target > 6, auto-fail. **Ethereal**: When the defender is designated as ethereal, rend (both weapon rend and the rend modifier) is ignored, so the target is just `defender.save`.
4. **Damage**: Unsaved wounds × weapon damage. Damage can be modified (flat values and dice expressions both supported).
5. **Ward** (optional): D6 per damage point. Target is defender's ward value.

> **2+ minimum target (all phases):** A natural roll of 1 always fails. No modifier can ever push a Hit, Wound, Save, or Ward target below 2+. For example, a 2+ hit test given a +1 modifier still requires a roll of 2 or higher — the +1 has no further effect once the target is already at 2+. This floor is enforced both at the modifier layer (so displayed targets are honest) and inside each dice-resolution function (so it cannot be bypassed by any caller).

### Weapon Stat Modifiers

The UI provides six modifier controls (range: -3 to +3 for numeric modifiers):
- **Hit modifier**: Adjusts the to-hit target (positive = easier to hit)
- **Wound modifier**: Adjusts the to-wound target (positive = easier to wound)
- **Rend modifier**: Adjusts effective rend (positive = better armor penetration)
- **Damage modifier**: Adds to damage output (e.g., `"D3" → "D3+2"` or `"2" → "4"`)
- **Attacks modifier**: Adds to the per-model attack count (e.g., `"D6" → "D6+1"` or `"2" → "4"`). Applied per-model before the total is computed, so with 5 models and modifier +2: `"2"` → 5 × 4 = 20 attacks. Ignored when using attack override.
- **Crit Effect**: Dropdown override to replace the weapon's built-in critical hit effect. Options are *Default (use weapon)*, *Auto Wound*, *Extra Hit*, and *Mortal Wounds*. When an override is active, the weapon stat display shows an `[Override]` prefix.

### Critical Hit Effects

- **Auto-Wound**: Wound roll auto-succeeds, proceeds to Save
- **Extra Hit**: Counts as 2 hits (both resolve through wound/save normally)
- **Mortal Wounds**: On a 6, bypass wound/save and deal the weapon's `damage` value directly as mortal wounds. The `damage_modifier` is applied to the damage value. Still subject to Ward saves.

## License

MIT
