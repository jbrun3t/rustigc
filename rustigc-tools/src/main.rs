use clap::Parser;
use log::error;
use rustigc::*;
use std::io::Write;
use std::io::{self, Read};

#[derive(Parser, Debug)]
#[command(name = "rustigc")]
#[command(about = "Parse IGC files", long_about = None)]
struct Args {
    /// Suppress all output (useful for profiling)
    #[arg(short, long)]
    quiet: bool,
}

fn main() -> io::Result<()> {
    env_logger::init();
    let args = Args::parse();

    let mut content = Vec::new();
    let bytes_read = io::stdin().lock().read_to_end(&mut content)?;

    if bytes_read == 0 {
        error!("No input on stdin");
        std::process::exit(0);
    }

    let log = Log::new(&content).unwrap_or_else(|e| {
        error!("{}", e);
        std::process::exit(1);
    });

    if !args.quiet {
        if let Err(e) = write_output(&log) {
            // Broken pipe is expected when piping to head, less, etc.
            // Exit silently in that case
            if e.kind() != io::ErrorKind::BrokenPipe {
                return Err(e);
            }
        }
    }

    Ok(())
}

fn write_output(log: &Log) -> io::Result<()> {
    let mut stdout = io::stdout();
    serde_json::to_writer_pretty(&mut stdout, log)?;
    writeln!(stdout)?;
    stdout.flush()?;
    Ok(())
}
