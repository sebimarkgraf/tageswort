use std::process;
use tageswort::{get_tageswort, Config, TageswortError};

fn main() {
    let config = Config::default();

    if let Err(err) = run(config) {
        eprintln!("Problem running the tageswort: {:#?}", err);
        process::exit(1);
    }
}

fn run(config: Config) -> Result<(), TageswortError> {
    let tageswort = get_tageswort(&config)?;
    println!("{}", tageswort);
    Ok(())
}
