pub use clap::Parser;
use std::string::String;

/// Parse bcl2fastq Stats.json output file and restructure as
/// table with each row as the index count
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Command {
    /// Path to the Stats.json file
    #[clap(short, long)]
    pub json_file: String,

    #[clap(short, long, default_value_t = 4)]
    pub max_distance: u8,
}
