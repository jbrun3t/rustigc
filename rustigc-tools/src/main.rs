use rustigc::*;
use std::io::Write;
use std::io::{self, Read};

fn main() -> io::Result<()> {
    let mut content = String::new();
    let bytes_read = io::stdin().lock().read_to_string(&mut content)?;

    if bytes_read == 0 {
        eprintln!("No input on stdin");
        std::process::exit(0);
    }

    let log = Log::new(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    if let Err(e) = write_output(&log) {
        // Broken pipe is expected when piping to head, less, etc.
        // Exit silently in that case
        if e.kind() != io::ErrorKind::BrokenPipe {
            return Err(e);
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
