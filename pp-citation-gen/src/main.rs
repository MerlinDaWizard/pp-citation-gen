#![feature(lazy_cell)]
use clap::{arg, Parser, Subcommand};
use pp_citation_lib::{generate, generate_gif, CitationData, Colour};
use std::{ffi::OsStr, fs::write, path::PathBuf};
/// A program to generate Papers Please style citations
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Creates an animated gif instead of a static image
    #[arg(short, long, action)]
    gif: bool,
    /// Citation header
    #[arg(short, long, default_value_t = CitationData::default().header_text.to_string())]
    header: String,
    /// A list of violations, each one being a seperate line
    #[arg(short, long, value_parser, num_args = 0..=4, value_delimiter = ',', default_value = "\"Protocol Violated\",\"Entry Permit: Invalid Name\"")]
    violations: Vec<String>,
    #[arg(short, long, default_value_t = CitationData::default().punishment_text.to_string())]
    punishment: String,
    /// File to write too, will guess format
    #[arg(short, long)]
    output_file: PathBuf,
    /// A css colour string representing the background colour
    #[arg(short, long, default_value_t = CitationData::default().bg_colour)]
    bg_colour: Colour,
    /// A css colour string representing the foreground colour
    #[arg(short, long, default_value_t = CitationData::default().fg_colour)]
    fg_colour: Colour,
    /// A css colour string representing the decoration colour
    #[arg(short, long, default_value_t = CitationData::default().decoration_colour)]
    decoration_colour: Colour,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Creates an animated gif with an animation of the citation sliding up
    Gif {},
    /// Creates a static image of the citation
    Image {},
}

fn main() -> anyhow::Result<()> {
    let cli = Args::parse();
    let mut violations = cli.violations.iter();

    let config = CitationData {
        bg_colour: cli.bg_colour,
        fg_colour: cli.fg_colour,
        decoration_colour: cli.decoration_colour,
        header_text: &cli.header,
        violation_text: [
            violations.next().and_then(|x| Some(x.as_str())), // A bit weird but oh well.
            violations.next().and_then(|x| Some(x.as_str())),
            violations.next().and_then(|x| Some(x.as_str())),
            violations.next().and_then(|x| Some(x.as_str())),
        ],
        punishment_text: &cli.punishment,
        ..Default::default()
    };

    if cli.gif {
        if cli
            .output_file
            .extension()
            .and_then(|x| Some(x.to_ascii_lowercase() == OsStr::new("gif")))
            == Some(false)
        {
            eprintln!("Warning: exporting gif into non gif file extension")
        }
        write(cli.output_file, generate_gif(&config))?;
    } else {
        generate(&config).save(cli.output_file)?
    }
    Ok(())
}
