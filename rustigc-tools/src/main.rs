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

    let raw =
        Log::new(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    let mut stdout = io::stdout();
    serde_json::to_writer_pretty(&mut stdout, &raw)?;
    writeln!(stdout)?;
    stdout.flush()?;

    Ok(())
}
