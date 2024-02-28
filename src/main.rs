use std::process;
use tageswort::{parse_tageswort_from_response, request_tageswort, Config, TageswortError};

fn main() {
    let config = Config::default();

    if let Err(err) = run(config) {
        eprintln!("Problem running the tageswort: {:#?}", err);
        process::exit(1);
    }
}

fn run(config: Config) -> Result<(), TageswortError> {
    let text = request_tageswort(&config)?;
    let tageswort = parse_tageswort_from_response(text)?;
    println!("{}", tageswort);
    Ok(())
}
