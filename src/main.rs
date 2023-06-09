use arcdps_log_tools::{
    extract_casts, extract_effects, extract_gear, extract_positions, extract_skills,
    hit_map::map_hits_to_set,
};
use arcdps_parse::{CombatEvent, EventKind, Log, Skill};
use clap::{error::ErrorKind, CommandFactory, Parser};
use serde::{Deserialize, Serialize};

mod cli;

use self::cli::*;

fn main() {
    let args = Args::parse();

    if args.input.is_empty() {
        Args::command()
            .error(
                ErrorKind::MissingRequiredArgument,
                "input file was not provided",
            )
            .exit();
    }

    let log = args.open_log();
    let events = args.filter_log(&log);

    match &args.command {
        Command::All => {
            #[derive(Debug, Clone, Serialize, Deserialize)]
            struct Event {
                kind: EventKind,
                #[serde(flatten)]
                event: CombatEvent,
            }

            impl From<CombatEvent> for Event {
                fn from(event: CombatEvent) -> Self {
                    Self {
                        kind: event.kind(),
                        event,
                    }
                }
            }

            let events: Vec<Event> = events.cloned().map(Into::into).collect();
            println!("Found {} events", events.len());
            args.write_output(&events);
        }

        Command::Cast { skill } => {
            let skill = skill.as_ref().map(|arg| find_skill(&log, arg));
            match skill {
                Some(skill) => println!("Finding casts of skill \"{}\" ({})", skill.name, skill.id),
                None => println!("Finding all skill casts"),
            }

            let data = extract_casts(&log, events, skill.map(|skill| skill.id));
            println!(
                "Found {} casts and {} hits without cast",
                data.casts.len(),
                data.hits_without_cast.len()
            );

            args.write_output(&data);
        }

        Command::Skill { skill } => {
            let skill = skill.as_ref().map(|arg| find_skill(&log, arg));
            match skill {
                Some(skill) => println!("Finding skill data for \"{}\" ({})", skill.name, skill.id),
                None => println!("Finding all skill data"),
            }

            let skills = extract_skills(&log, skill.map(|skill| skill.id));
            println!("Found {} skills/buffs", skills.len());

            args.write_output(&skills);
        }

        Command::Position => {
            println!("Finding positions");

            let positions = extract_positions(&log, events);
            println!("Found {} positions", positions.len());
            if let Some(pos) = positions.first() {
                println!("Initial position at {} {} {}", pos.x, pos.y, pos.z);
            }

            args.write_output(&positions);
        }

        Command::Effect => {
            println!("Finding effects");

            let effects = extract_effects(&log, events);
            println!("Found {} effects", effects.len());

            args.write_output(&effects);
        }

        Command::Hitmap => {
            let agent = args.agent_filter(&log).unwrap_or_else(|| {
                Args::command()
                    .error(
                        ErrorKind::MissingRequiredArgument,
                        "hit mapping requires agent",
                    )
                    .exit()
            });
            println!("Mapping direct damage hits to weapon sets");

            let hit_map: Vec<_> = map_hits_to_set(&log, agent.address).collect();
            println!("Found {} weapon sets", hit_map.len());

            args.write_output(&hit_map);
        }

        Command::Gear => {
            println!("Extracting POV gear");

            let gear = extract_gear(&log);
            println!(
                "Found {} gear buffs, {} runes, {} sigils",
                gear.buffs.len(),
                gear.runes.len(),
                gear.sigils
                    .values()
                    .map(|sigils| sigils.len())
                    .sum::<usize>(),
            );

            args.write_output(&gear);
        }
    }
}

fn find_skill<'a>(log: &'a Log, id_or_name: &str) -> &'a Skill {
    log.skills
        .iter()
        .find(|entry| match id_or_name.parse::<u32>() {
            Ok(id) => entry.id == id,
            Err(_) => entry.name == id_or_name,
        })
        .unwrap_or_else(|| panic!("Skill \"{}\" not found", id_or_name))
}
