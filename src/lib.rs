pub mod agent;
pub mod cast;
pub mod effect;
pub mod gear;
pub mod hit;
pub mod hit_map;
pub mod position;
pub mod skill;
pub mod util;
pub mod weapons;

pub use self::agent::Agent;
pub use self::cast::{extract_casts, Cast, Casts};
pub use self::effect::{extract_effects, Effect};
pub use self::gear::{extract_gear, GearInfo, GearItem, Rune, Sigil};
pub use self::hit::{Hit, HitWithSkill};
pub use self::position::{extract_positions, Position};
pub use self::skill::{extract_skills, Skill, SkillKind, SkillWithInfo};
pub use self::weapons::{WeaponMap, WeaponSet};

use arcdps_parse::{Log, StateChange};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BreakbarState {
    Active = 0,
    Recover = 1,
    Immune = 2,
    None = 3,
}

pub fn log_start(log: &Log) -> u64 {
    log.events
        .iter()
        .find(|event| event.is_statechange == StateChange::LogStart)
        .map(|event| event.time)
        .expect("no log start event")
}
