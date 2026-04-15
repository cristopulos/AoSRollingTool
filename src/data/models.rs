use serde::{de, Deserialize, Deserializer, Serialize};

/// Critical hit effects triggered on a roll of 6.
///
/// Variants:
/// - `AutoWound`: Wound roll auto-succeeds
/// - `ExtraHit`: Counts as 2 hits (base + extra)
/// - `MortalWounds`: Deals `weapon.damage` directly as mortal wounds, bypassing wound and save rolls
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type")]
pub enum CritEffect {
    #[serde(rename = "auto_wound")]
    AutoWound,
    #[serde(rename = "extra_hit")]
    ExtraHit,
    #[serde(rename = "mortal_wounds")]
    MortalWounds,
}

impl<'de> Deserialize<'de> for CritEffect {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct Helper {
            #[serde(rename = "type")]
            ty: String,
            #[serde(default)]
            value: Option<serde::de::IgnoredAny>,
        }

        let helper = Helper::deserialize(deserializer)?;
        match helper.ty.as_str() {
            "auto_wound" => Ok(CritEffect::AutoWound),
            "extra_hit" => Ok(CritEffect::ExtraHit),
            "mortal_wounds" => Ok(CritEffect::MortalWounds),
            other => Err(de::Error::unknown_variant(
                other,
                &["auto_wound", "extra_hit", "mortal_wounds"],
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Weapon {
    pub name: String,
    pub range: Option<String>,
    pub attack: String,
    pub to_hit: u8,
    pub to_wound: u8,
    pub rend: i8,
    pub damage: String,
    pub crit_hit: Option<CritEffect>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unit {
    pub id: String,
    pub name: String,
    pub faction: String,
    pub save: u8,
    pub ward: Option<u8>,
    pub weapons: Vec<Weapon>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CombatConfig {
    pub attacker_ids: Vec<String>,
    pub weapon_name: String,
    pub defender_id: String,
    pub include_ward: bool,
}
