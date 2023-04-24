use arcdps_parse::{CombatEvent, Log, Parse};
use clap::Parser;
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};
use zip::ZipArchive;

/// CLI interface.
#[derive(Debug, Clone, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Command.
    #[command(subcommand)]
    pub command: Command,

    #[clap(flatten)]
    pub args: Args,
}

/// Arguments.
#[derive(Debug, Clone, clap::Args)]
pub struct Args {
    /// Path to input file.
    #[clap(global = true, default_value_t)]
    pub input: String,

    /// Path to output file, defaults to input filename.
    #[clap(global = true)]
    pub output: Option<PathBuf>,

    /// Id or name of agent to filter data for.
    #[clap(short, long, global = true)]
    pub agent: Option<String>,
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
    /// Extract all events.
    All,

    /// Extract cast & hit data.
    Cast {
        /// Id or name of skill to extract data for.
        #[clap(short, long)]
        skill: String,
    },

    /// Extract position data.
    Position,

    /// Extract buff information.
    BuffInfo,
}

impl Args {
    pub fn open_log(&self) -> Log {
        let mut archive = ZipArchive::new(BufReader::new(
            File::open(&self.input).expect("unable to open input file"),
        ))
        .expect("input log file not compressed");
        let mut file = archive.by_index(0).expect("input log file empty");

        let mut log = Log::parse(&mut file).expect("failed to parse EVTC log");
        log.events.sort_by_key(|event| event.time);
        log
    }

    pub fn filter_log<'a>(&self, log: &'a Log) -> impl Iterator<Item = &'a CombatEvent> {
        let agent = self.agent.as_deref().map(|arg| {
            log.agents
                .iter()
                .find(|agent| match arg.parse::<u64>() {
                    Ok(id) => agent.address == id,
                    Err(_) => agent.name[0] == arg,
                })
                .unwrap_or_else(|| panic!("Agent \"{}\" not found", arg))
        });

        println!(
            "Agent filter: {}",
            agent
                .map(|agent| format!("\"{}\" ({})", agent.name[0], agent.address))
                .unwrap_or_else(|| "all agents".into())
        );

        let agent_filter = agent.map(|agent| agent.address);
        log.events
            .iter()
            .filter(move |event| agent_filter.map(|id| event.src_agent == id).unwrap_or(true))
    }

    pub fn write_output<T>(&self, value: &T)
    where
        T: ?Sized + serde::Serialize,
    {
        let out = self
            .output
            .clone()
            .unwrap_or_else(|| {
                Path::new(&self.input)
                    .file_name()
                    .expect("input path is no file")
                    .into()
            })
            .with_extension("json");

        let output = BufWriter::new(File::create(&out).expect("failed to create output file"));
        serde_json::to_writer_pretty(output, value).expect("failed to serialize data");
        println!("Saved data to \"{}\"", out.display());
    }
}