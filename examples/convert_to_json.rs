use ame2020::Iter;
use clap::Parser;
use serde_json::to_writer_pretty;
use std::{
    error::Error,
    fs::File,
    io::{stdout, BufReader},
    path::PathBuf,
};

/// Example program for converting from the atomic mass evaluation format to json
#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Cli {
    /// File to read from.
    file: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let file = File::open(cli.file)?;
    let file = BufReader::new(file);

    let v = Iter::new(file).collect::<Result<Vec<_>, _>>()?;
    let writer = stdout().lock();
    to_writer_pretty(writer, &v)?;

    Ok(())
}
