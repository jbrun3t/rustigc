// SPDX-License-Identifier: GPL-2.0-or-later

use clap::{Parser, ValueEnum};

use rustigc::*;
use std::io::{self, Read};

use rustigc_tools::timezone::LocalTime;

const EXAMPLES: &str = "\
EXAMPLES:
    Score against the default ruleset:
        rustigc-xc-score < flight.igc

    Machine-readable output for another ruleset:
        rustigc-xc-score --ruleset cfd --format json < flight.igc

    Score against a ruleset of your own:
        rustigc-xc-score --ruleset-file my-league.json < flight.igc

    Draw the result on a map:
        rustigc-xc-score --format geojson < flight.igc > flight.geojson

    Score an explicit range of fixes:
        rustigc-xc-score --window 125,25425 < flight.igc";

/// How a result is printed.
#[derive(Clone, ValueEnum, Debug, PartialEq)]
enum Format {
    /// The flight date, one line per point of the task, then the score.
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
                  Every rule of the ruleset is searched at once and the highest scoring one is \
                  reported. Unless --window says otherwise, the flight is detected first and only \
                  its fixes are scored."
)]
#[command(after_long_help = EXAMPLES)]
struct Args {
    /// Scoring ruleset
    #[arg(long, default_value = "xcontest",
          value_parser = clap::builder::PossibleValuesParser::new(rustigc::preset_names()),
          long_help = "Ruleset to score against. Each one defines its own rules, \
                       multipliers and penalties, so the same track scores differently under \
                       each.")]
    ruleset: String,

    /// Scoring ruleset read from a JSON file
    #[arg(
        long,
        value_name = "PATH",
        conflicts_with = "ruleset",
        long_help = "Ruleset to score against, read from a JSON file in the format of the \
                       presets. Replaces --ruleset."
    )]
    ruleset_file: Option<std::path::PathBuf>,

    /// Output format
    #[arg(long, value_enum, default_value = "human")]
    format: Format,

    /// Fix window `start,stop` to score instead of the detected flight.
    #[arg(long, value_parser = parse_window,
          long_help = "Score this range of fix indices instead of the detected flight. Both \
                       bounds are indices into the track, the first fix being 0, and the report's \
                       takeoff and landing are these bounds.")]
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

    if let Some(l) = &local {
        let entry = l.at(log.track[result.entry].timestamp);
        println!("{}", entry.strftime("Flight on %Y-%m-%d %Z"));
    } else {
        println!("Flight has no date !");
    }

    println!("Takeoff: {}", disp(result.takeoff));
    println!(" Entry : {}", disp(result.entry));
    for (i, tp) in result.turnpoints.iter().enumerate() {
        println!("  TP{i}  : {}", disp(*tp));
    }
    println!(" Exit  : {}", disp(result.exit));
    println!("Landing: {}", disp(result.landing));

    print!(
        "{} {} points, {} km",
        result.description, result.score, result.distance_km
    );
    if result.multiplier != 1.0 {
        print!(" (×{})", result.multiplier);
    }
    if result.circuit {
        print!(
            " [ closing distance: {:.0} / {:.0} m ]",
            result.gap_m, result.threshold_m
        );
    }
    println!();
}

/// The ruleset in the JSON file at `path`; exits on a file that cannot be read or used.
fn read_ruleset(path: &std::path::Path) -> Ruleset {
    std::fs::read_to_string(path)
        .map_err(|e| e.to_string())
        .and_then(|text| Ruleset::from_json(&text).map_err(|e| e.to_string()))
        .unwrap_or_else(|e| {
            eprintln!("Could not read ruleset {}: {e}", path.display());
            std::process::exit(1);
        })
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

    let log = Log::new(&content).unwrap_or_else(|e| {
        eprintln!("Could not parse: {e}");
        std::process::exit(1);
    });

    let ruleset = match &args.ruleset_file {
        Some(path) => read_ruleset(path),
        // clap only accepts preset ids.
        None => Ruleset::preset(&args.ruleset).expect("ruleset accepted by clap"),
    };

    let window = args
        .window
        .or_else(|| log.track.flights().longest().map(|f| (f.start, f.stop)));
    let scored = window.and_then(|(start, stop)| {
        log.score(&ruleset, start, stop).unwrap_or_else(|e| {
            eprintln!("Could not score: {e}");
            std::process::exit(1);
        })
    });

    if scored.is_none() {
        eprintln!("Could not score");
    }

    match args.format {
        Format::Geojson => {
            let flight = window.map(|(start, stop)| Flight { start, stop });
            let collection = log
                .export(flight, scored.as_ref(), TrackLine::Draw)
                .unwrap_or_else(|e| {
                    eprintln!("Could not draw: {e}");
                    std::process::exit(1);
                });

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
