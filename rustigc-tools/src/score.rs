// SPDX-License-Identifier: GPL-2.0-or-later

use clap::{Parser, ValueEnum};

use rustigc::*;
use std::io::{self, Read};

use rustigc_tools::timezone::LocalTime;

const EXAMPLES: &str = "\
EXAMPLES:
    Score against the default league:
        rustigc-xc-score < flight.igc

    Machine-readable output for another league:
        rustigc-xc-score --league cfd --format json < flight.igc

    Draw the result on a map:
        rustigc-xc-score --format geojson < flight.igc > flight.geojson

    Score an explicit range of fixes:
        rustigc-xc-score --window 125,25425 < flight.igc";

/// How a result is printed.
#[derive(Clone, ValueEnum, Debug, PartialEq)]
enum Format {
    /// One line per fix of the task, then the score.
    Human,
    /// The scoring report as JSON, `null` when nothing scored.
    Json,
    /// The track, the flight and the task, drawn as GeoJSON.
    Geojson,
}

#[derive(Parser, Debug)]
#[command(name = "rustigc-xc-score", version)]
#[command(about = "Score IGC files")]
#[command(
    long_about = "Score an IGC file read on stdin and report the best task found.\n\n\
                  Every rule of the league is searched at once and the highest scoring one is \
                  reported. Unless --window says otherwise, the flight is detected first and only \
                  its fixes are scored."
)]
#[command(after_long_help = EXAMPLES)]
struct Args {
    /// Scoring league
    #[arg(long, default_value = "xcontest",
          value_parser = clap::builder::PossibleValuesParser::new(rustigc::league_names()),
          long_help = "Ruleset to score against. Each league defines its own rules, \
                       multipliers and penalties, so the same track scores differently under \
                       each.")]
    league: String,

    /// Output format
    #[arg(long, value_enum, default_value = "human")]
    format: Format,

    /// Explicit `start,stop` fix window to score, bypassing flight auto-detection.
    #[arg(long, value_parser = parse_window,
          long_help = "Score this range of fix indices instead of the auto-detected flight. \
                       Both bounds are indices into the track, the first fix being 0. The \
                       detected flight is still what the human report calls takeoff and landing.")]
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
    let local = LocalTime::new(log);

    let coord = |i: usize| format!("{:.04},{:.04}", log.track[i].lat, log.track[i].lon);
    let disp = |i: usize| match &local {
        Some(l) => {
            format!(
                "{} - [{}] - @{i}",
                l.at(log.track[i].timestamp).strftime("%H:%M:%S"),
                coord(i)
            )
        }
        None => format!("[{}] - @{i}", coord(i)),
    };

    // Display the flight date, locally.
    if let Some(l) = &local {
        let entry = l.at(log.track[result.entry].timestamp);
        println!("{}", entry.strftime("Flight on %Y-%m-%d %Z"));
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
        result.description, result.score, result.distance_km
    );
    if result.multiplier != 1.0 {
        print!(" (×{})", result.multiplier);
    }
    if result.circuit {
        print!(
            " [ closing distance: {} km, max {:.0} m ]",
            result.gap_km, result.threshold_m
        );
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
