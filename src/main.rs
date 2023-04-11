use arcdps_hit_times::{agent_name, extract_casts, skill_name, until_null};
use arcdps_parse::{Log, Parse};
use clap::Parser;
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
};
use zip::ZipArchive;

/// CLI arguments.
#[derive(Debug, Clone, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to input file.
    pub input: PathBuf,

    /// Id or name of skill to extract data for.
    #[clap(short, long)]
    pub skill: String,

    /// Name of agent to filter data for.
    #[clap(short, long)]
    pub agent: Option<String>,

    /// Path to output file, defaults to input filename.
    pub output: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();
    let out = args
        .output
        .unwrap_or_else(|| {
            args.input
                .file_name()
                .expect("input path is no file")
                .into()
        })
        .with_extension("json");

    let mut archive = ZipArchive::new(BufReader::new(
        File::open(&args.input).expect("unable to open input file"),
    ))
    .expect("input log file not compressed");
    let mut file = archive.by_index(0).expect("input log file empty");

    let mut log = Log::parse(&mut file).expect("failed to parse EVTC log");
    log.events.sort_by_key(|event| event.time);

    let (skill_id, skill_name) = if let Ok(id) = args.skill.parse() {
        let name = skill_name(&log.skills, id).unwrap_or_default();
        (id, name)
    } else {
        let skill = log
            .skills
            .iter()
            .find(|skill| until_null(&skill.name) == args.skill)
            .unwrap_or_else(|| panic!("Skill \"{}\" not found", args.skill));
        (skill.id, args.skill.as_str())
    };

    let agent_filter = args.agent.as_deref().map(|name| {
        log.agents
            .iter()
            .find(|agent| until_null(&agent.name) == name)
            .map(|agent| agent.address)
            .unwrap_or_else(|| panic!("Did not find agent \"{}\"", name))
    });

    println!(
        "Finding casts of skill \"{}\" ({}) for {}",
        skill_name,
        args.skill,
        args.agent.as_deref().unwrap_or("all agents")
    );
    let (casts, hits_without_cast) = extract_casts(&log, skill_id, agent_filter);

    for info in &hits_without_cast {
        eprintln!(
            "Hit from \"{}\" ({}) at time {} without prior cast",
            agent_name(&log.agents, info.agent).unwrap_or_default(),
            info.agent,
            info.time
        );
    }
    println!(
        "Found {} casts and {} hits without cast",
        casts.len(),
        hits_without_cast.len()
    );

    let output = BufWriter::new(File::create(&out).expect("failed to create output file"));
    serde_json::to_writer_pretty(output, &casts).expect("failed to serialize cast data");
    println!("Saved cast data to \"{}\"", out.display());
}
