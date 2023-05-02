pub mod agent;
pub mod cast;
pub mod effect;
pub mod hit;
pub mod hit_map;
pub mod position;
pub mod skill;
mod util;

pub use self::agent::Agent;
pub use self::cast::{extract_casts, Cast, Casts};
pub use self::effect::{extract_effects, Effect};
pub use self::hit::{Hit, HitWithSkill};
pub use self::position::{extract_positions, Position};
pub use self::skill::{extract_skills, Skill, SkillKind, SkillWithInfo};

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
