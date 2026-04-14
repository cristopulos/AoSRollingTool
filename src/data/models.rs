use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum CritEffect {
    #[serde(rename = "auto_wound")]
    AutoWound,
    #[serde(rename = "extra_hit")]
    ExtraHit,
    #[serde(rename = "mortal_wounds")]
    MortalWounds(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Weapon {
    pub name: String,
    pub range: Option<String>,
    pub attacks: String,
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
