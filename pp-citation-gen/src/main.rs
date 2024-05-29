#![feature(lazy_cell)]
use anyhow::anyhow;
use clap::{arg, Parser, Subcommand};
use image::Rgba;
use pp_citation_lib::{generate, generate_gif, CitationData};
use std::{ffi::OsStr, fmt::Display, fs::write, path::PathBuf, str::FromStr};
/// A program to generate Papers Please style citations
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Creates an animated gif instead of a static image
    #[arg(short, long, action)]
    gif: bool,
    /// Citation header
    #[arg(short, long, default_value_t = CitationData::default().header_text)]
    header: String,
    /// A list of violations, each one being a seperate line
    #[arg(short, long, value_parser, num_args = 0..=4, value_delimiter = ',', default_value = "\"Protocol Violated\",\"Entry Permit: Invalid Name\"")]
    violations: Vec<String>,
    #[arg(short, long, default_value_t = CitationData::default().punishment_text)]
    punishment: String,
    /// File to write too, will guess format
    #[arg(short, long)]
    output_file: PathBuf,
    #[arg(short, long, default_value_t = RgbaWrapper(CitationData::default().bg_colour))]
    bg_colour: RgbaWrapper,
    #[arg(short, long, default_value_t = RgbaWrapper(CitationData::default().fg_colour))]
    fg_colour: RgbaWrapper,
    #[arg(short, long, default_value_t = RgbaWrapper(CitationData::default().decoration_colour))]
    decoration_colour: RgbaWrapper,
}

#[derive(Debug, Clone)]
struct RgbaWrapper(Rgba<u8>);

impl RgbaWrapper {
    fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        RgbaWrapper(Rgba([r, g, b, a]))
    }
}
impl Display for RgbaWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "[{}, {}, {}, {}]",
            self.0 .0[0], self.0 .0[1], self.0 .0[2], self.0 .0[3],
        ))
    }
}

impl Into<Rgba<u8>> for RgbaWrapper {
    fn into(self) -> Rgba<u8> {
        self.0
    }
}

impl FromStr for RgbaWrapper {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim().trim_matches(|c| ['[', ']', '(', ')'].contains(&c));
        let colours = trimmed.split(',').map(|x| x.trim());
        let mut rgb = colours.map(|x| x.parse::<u8>());
        Ok(RgbaWrapper::new(
            rgb.next()
                .ok_or(anyhow!("Could not parse red colour channel"))??,
            rgb.next()
                .ok_or(anyhow!("Could not parse green colour channel"))??,
            rgb.next()
                .ok_or(anyhow!("Could not parse blue colour channel"))??,
            rgb.next().unwrap_or(Ok(255))?,
        ))
    }
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
    let mut violations = cli.violations.into_iter();

    let config = CitationData {
        bg_colour: cli.bg_colour.into(),
        fg_colour: cli.fg_colour.into(),
        decoration_colour: cli.decoration_colour.into(),
        header_text: cli.header,
        violation_text: [
            violations.next(),
            violations.next(),
            violations.next(),
            violations.next(),
        ],
        punishment_text: cli.punishment,
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
    // let img = generate(config);
    // img.save("output.png").unwrap();

    let data = generate_gif(&config);
    write("output.gif", data).unwrap();
    Ok(())
}
