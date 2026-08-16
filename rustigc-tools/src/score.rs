use clap::{Parser, ValueEnum};

use rustigc::*;
use std::io::{self, Read};

#[derive(Clone, ValueEnum, Debug, PartialEq)]
enum Format {
    Human,
    Json,
}

#[derive(Parser, Debug)]
#[command(name = "rustigc-xc-score")]
#[command(about = "Score IGC Files", long_about = None)]
struct Args {
    /// Scoring league
    #[arg(long, default_value = "xcontest",
          value_parser = clap::builder::PossibleValuesParser::new(rustigc::league_names()))]
    league: String,

    /// Output format
    #[arg(long, value_enum, default_value = "human")]
    format: Format,

    /// Explicit `start,stop` fix window to score, bypassing flight auto-detection.
    #[arg(long, value_parser = parse_window)]
    window: Option<(usize, usize)>,
}

fn parse_window(s: &str) -> Result<(usize, usize), String> {
    let (start, stop) = s
        .split_once(',')
        .ok_or_else(|| "expected \"start,stop\"".to_string())?;
    let start: usize = start
        .trim()
        .parse()
        .map_err(|_| format!("invalid start: {start}"))?;
    let stop: usize = stop
        .trim()
        .parse()
        .map_err(|_| format!("invalid stop: {stop}"))?;
    Ok((start, stop))
}

fn main() -> io::Result<()> {
    env_logger::init();
    let args = Args::parse();

    let mut content = Vec::new();
    let bytes_read = io::stdin().lock().read_to_end(&mut content)?;

    if bytes_read == 0 {
        eprintln!("No input on stdin");
        std::process::exit(0);
    }

    let log =
        Log::new(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    // The detected bracket is reported as such whatever `--window` scores.
    let flight = Analysis::new(&log.track).flight();
    let window = args.window.or(flight);
    let scored = window.and_then(|(start, stop)| log.score(&args.league, start, stop));

    match scored {
        Some(result) => match args.format {
            Format::Json => {
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
            Format::Human => {
                println!("Entry @{}", result.entry);
                println!("Exit  @{}", result.exit);
                for (i, tp) in result.turnpoints.iter().enumerate() {
                    println!("  - TP{} @{}", i, tp);
                }
                print!(
                    "{} {} points, {} km",
                    result.description, result.score, result.distance
                );
                if result.multiplier != 1.0 {
                    print!(" (×{})", result.multiplier);
                }
                if result.circuit {
                    print!(" [ closing distance: {} km ]", result.gap);
                }
                println!();
            }
        },
        None => {
            eprintln!("No TP found");
            match args.format {
                Format::Json => println!("null"),
                Format::Human => {}
            }
        }
    }

    Ok(())
}
