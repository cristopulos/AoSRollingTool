use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::models::Unit;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitDatabase {
    pub units: Vec<Unit>,
}

pub fn load_units_from_path<P: AsRef<Path>>(path: P) -> Result<Vec<Unit>, io::Error> {
    let content = fs::read_to_string(path)?;
    let db: UnitDatabase = serde_json::from_str(&content)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(db.units)
}

#[allow(dead_code)]
pub fn load_units_from_str(content: &str) -> Result<Vec<Unit>, io::Error> {
    let db: UnitDatabase =
        serde_json::from_str(content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(db.units)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::models::CritEffect;

    const SAMPLE_UNITS_JSON: &str = r#"{
        "units": [
            {
                "id": "test_unit",
                "name": "Test Unit",
                "faction": "Test",
                "save": 4,
                "ward": null,
                "weapons": [
                    {
                        "name": "Sword",
                        "attack": "3",
                        "to_hit": 4,
                        "to_wound": 4,
                        "rend": 0,
                        "damage": "1",
                        "crit_hit": null
                    }
                ]
            },
            {
                "id": "crit_unit",
                "name": "Crit Unit",
                "faction": "Test",
                "save": 4,
                "ward": null,
                "weapons": [
                    {
                        "name": "Mortal Weapon",
                        "attack": "3",
                        "to_hit": 3,
                        "to_wound": 3,
                        "rend": -1,
                        "damage": "1",
                        "crit_hit": {
                            "type": "mortal_wounds",
                            "value": "D6"
                        }
                    }
                ]
            },
            {
                "id": "ward_unit",
                "name": "Ward Unit",
                "faction": "Test",
                "save": 4,
                "ward": 5,
                "weapons": []
            }
        ]
    }"#;

    #[test]
    fn load_valid_unit_json() {
        let json = r#"{
            "id": "single_unit",
            "name": "Single Unit",
            "faction": "Test",
            "save": 4,
            "ward": null,
            "weapons": [{"name": "Sword", "attack": "3", "to_hit": 4, "to_wound": 4, "rend": 0, "damage": "1"}]
        }"#;

        let unit = serde_json::from_str::<Unit>(json).unwrap();
        assert_eq!(unit.name, "Single Unit");
        assert_eq!(unit.weapons.len(), 1);
    }

    #[test]
    fn load_units_from_file() {
        let units = load_units_from_str(SAMPLE_UNITS_JSON).unwrap();
        assert_eq!(units.len(), 3);
        assert_eq!(units[0].name, "Test Unit");
    }

    #[test]
    fn unit_with_crit_weapon_parses() {
        let units = load_units_from_str(SAMPLE_UNITS_JSON).unwrap();
        let crit_unit = units.iter().find(|u| u.id == "crit_unit").unwrap();
        assert_eq!(
            crit_unit.weapons[0].crit_hit,
            Some(CritEffect::MortalWounds("D6".into()))
        );
    }

    #[test]
    fn unit_with_ward_parses() {
        let units = load_units_from_str(SAMPLE_UNITS_JSON).unwrap();
        let ward_unit = units.iter().find(|u| u.id == "ward_unit").unwrap();
        assert_eq!(ward_unit.ward, Some(5));
    }
}
