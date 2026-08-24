// SPDX-License-Identifier: GPL-2.0-or-later

use clap::{Parser, ValueEnum};

use rustigc::*;
use std::io::{self, Read};

#[derive(Clone, ValueEnum, Debug, PartialEq)]
enum Format {
    Human,
    Json,
    Geojson,
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

fn human_output(log: &Log, result: &ScoringResult) {
    // Get time origin for the flight and create display helper
    let origin = log.datetime();

    let coord = |i: usize| format!("{:.04},{:.04}", log.track[i].lat, log.track[i].lon);
    let disp = |i: usize| match &origin {
        Some(o) => {
            format!(
                "{} - [{}] - @{i}",
                log.track[i].datetime(o).strftime("%H:%M:%S"),
                coord(i)
            )
        }
        None => format!("[{}] - @{i}", coord(i)),
    };

    // Display the flight date, locally.
    if let Some(o) = &origin {
        let entry = log.track[result.entry].datetime(o);
        println!("{}", entry.strftime("Flight on %Y-%m-%d %:Q"));
    } else {
        println!("Flight has no date !");
    }

    // Points location
    println!("Takeoff: {}", disp(result.takeoff));
    println!(" Entry : {}", disp(result.entry));
    for (i, tp) in result.turnpoints.iter().enumerate() {
        println!("  TP{i}  : {}", disp(*tp));
    }
    println!(" Exit  : {}", disp(result.exit));
    println!("Landing: {}", disp(result.landing));

    // Final score report
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

    let window = args
        .window
        .or_else(|| log.track.flights().longest().map(|f| (f.start, f.stop)));
    let scored = window.and_then(|(start, stop)| log.score(&args.league, start, stop));

    if scored.is_none() {
        eprintln!("Could not score");
    }

    match args.format {
        Format::Geojson => {
            let flight = window.map(|(start, stop)| Flight { start, stop });
            let collection = log.export_flight(flight, scored.as_ref(), TrackLine::Draw);

            println!("{}", serde_json::to_string(&collection)?);
        }
        Format::Json => match &scored {
            Some(result) => println!("{}", serde_json::to_string_pretty(result)?),
            None => println!("null"),
        },
        Format::Human => {
            if let Some(result) = &scored {
                human_output(&log, result);
            }
        }
    }

    Ok(())
}
