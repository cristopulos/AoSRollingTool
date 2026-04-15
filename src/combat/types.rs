use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Hit,
    Wound,
    Save,
    Damage,
    Ward,
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Phase::Hit => write!(f, "Hit"),
            Phase::Wound => write!(f, "Wound"),
            Phase::Save => write!(f, "Save"),
            Phase::Damage => write!(f, "Damage"),
            Phase::Ward => write!(f, "Ward"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiceRoll {
    pub value: u8,
    pub success: bool,
    pub is_crit: bool,
}

#[derive(Debug, Clone)]
pub enum VarianceStep {
    AttackRoll {
        per_model: String,
        results: Vec<u8>,
        total: usize,
    },
    DamageRoll {
        per_wound: String,
        results: Vec<u8>,
        total: usize,
    },
}

/// The result of a single combat phase (Hit, Wound, Save, Damage, or Ward).
/// Contains dice rolls, success/failure counts, and optional annotations
/// that explain how crit effects contributed to the phase outcome.
#[derive(Debug, Clone)]
pub struct PhaseResult {
    #[allow(dead_code)]
    pub phase: Phase,
    pub rolls: Vec<DiceRoll>,
    pub successes: usize,
    pub failures: usize,
    pub total_output: usize,
    pub auto_fails: bool,
    pub skipped: bool,
    pub description: String,
    pub variance_step: Option<VarianceStep>,
    /// Human-readable annotation explaining crit effect contributions to this phase.
    /// Displayed below the phase in the UI breakdown:
    /// - Wound phase: "(+N extra hit[s])" for ExtraHit crits
    /// - Save phase: "(+N auto-wound[s])" for AutoWound crits
    /// - Damage phase: "(+N MW)" for MortalWounds crits
    pub annotation: Option<String>,
    /// Number of extra dice this phase received from crit effects.
    /// - Wound phase: count of ExtraHit crits (each generates an additional wound roll)
    /// - Save phase: count of AutoWound crits (each bypassed the wound roll)
    /// - Damage phase: count of MortalWound crits (each dealt damage directly)
    #[allow(dead_code)]
    pub crit_extra_count: usize,
}

#[derive(Debug, Clone)]
pub struct CombatResult {
    pub attacker_name: String,
    pub weapon_name: String,
    pub defender_name: String,
    pub phases: Vec<PhaseResult>,
    pub final_damage: usize,
    /// Mortal wounds from critical hits that bypass the Save phase.
    /// Still subject to Ward saves.
    pub mortal_wounds: usize,
    /// True when combat was stopped after the Wound phase via the "Stop after wound" option.
    pub stopped_after_wound: bool,
    /// Total successful hits (including auto-wounds and extra hits from crits).
    /// Only meaningful when `stopped_after_wound` is true.
    pub total_hits: usize,
    /// Total successful wounds (normal wounds + auto-wounds from crits).
    /// Only meaningful when `stopped_after_wound` is true.
    pub total_wounds: usize,
}

#[derive(Debug, Clone, Default)]
pub struct WardResult {
    pub final_damage: usize,
    pub wounds_saved: usize,
    pub rolls: Vec<DiceRoll>,
}
