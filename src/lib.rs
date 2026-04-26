//! # tageswort
//! `tageswort` is a library for fetching the daily word of the day from aphorismen.de.
//! It provides a simple API to fetch the word of the day and parse it into a struct.
//! The struct contains the text of the word of the day and a link to the aphorismen.de website.
//!
//! The library is built on top of reqwest for fetching the word of the day and urlencoding for decoding the response.

use std::env;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::string::FromUtf8Error;
use urlencoding::decode;

pub struct Config {
    url: String,
}

impl Config {
    pub fn new(url: String) -> Config {
        Config { url }
    }
}

impl Default for Config {
    fn default() -> Self {
        let default_url = env::var("TAGESWORT_URL").unwrap_or(String::from(
            "https://assets.aphorismen.de/tagesspruch/tageswort.txt",
        ));
        Config::new(default_url)
    }
}

#[derive(Debug)]
pub enum TageswortError {
    Reqwest(reqwest::Error),
    UrlEncoding(FromUtf8Error),
}

impl From<reqwest::Error> for TageswortError {
    fn from(error: reqwest::Error) -> Self {
        TageswortError::Reqwest(error)
    }
}

impl From<FromUtf8Error> for TageswortError {
    fn from(error: FromUtf8Error) -> Self {
        TageswortError::UrlEncoding(error)
    }
}

pub struct Tageswort {
    pub text: String,
    pub link: String,
}

impl Display for Tageswort {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}

/// Decodes the raw response body returned by aphorismen.de into plain text.
pub fn decode_tageswort_response(body: &str) -> Result<String, TageswortError> {
    let text = decode(body)?.into_owned();
    Ok(text)
}

pub fn parse_tageswort_from_response(text: String) -> Result<Tageswort, TageswortError> {
    let lines: Vec<&str> = text.split("\n").collect();
    let tageswort = Tageswort {
        text: lines[0..lines.len() - 3].join("\n"),
        link: String::from("https://aphorismen.de/zitat/") + lines[lines.len() - 3],
    };
    Ok(tageswort)
}

/// Fetches the word of the day from aphorismen.de and returns it as a string.
/// The word of the day is fetched from the url specified in the config.
///
/// # Arguments
/// * `config` - The configuration for fetching the word of the day.
/// # Returns
/// * The word of the day as a string.
/// # Errors
/// * If the request to fetch the word of the day fails.
/// * If the response from the request cannot be decoded.
/// # Example
/// ```
/// use tageswort::{decode_tageswort_response, parse_tageswort_from_response};
///
/// let raw_body = "\
/// Dankbarkeit%0A\
/// Es%20ist%20schwer%20einzusehen%2C%20warum%20wir%20%C3%BCberschw%C3%A4nglich%20dankbar%20sein%20sollen%20f%C3%BCr%20etwas%2C%20das%20wir%20nicht%20wollen%2C%20solange%20uns%20das%2C%20was%20wir%20wollen%2C%20vorenthalten%20wird.%0A\
/// Lisle%20de%20Vaux%20Matthewman%0A\
/// %281867%20-%201903%29%2C%20Journalist%20und%20Schriftsteller%0A\
/// 232285%0A\
/// 11669%0A";
///
/// let text = decode_tageswort_response(raw_body).unwrap();
/// let tageswort = parse_tageswort_from_response(text).unwrap();
///
/// assert_eq!(tageswort.to_string(), "Dankbarkeit\nEs ist schwer einzusehen, warum wir überschwänglich dankbar sein sollen für etwas, das wir nicht wollen, solange uns das, was wir wollen, vorenthalten wird.\nLisle de Vaux Matthewman\n(1867 - 1903), Journalist und Schriftsteller");
/// assert_eq!(tageswort.link, "https://aphorismen.de/zitat/232285");
/// ```
///
/// Live usage:
/// ```no_run
/// use tageswort::{Config, request_tageswort};
///
/// let config = Config::default();
/// let tageswort = request_tageswort(&config).unwrap();
/// assert!(!tageswort.is_empty());
/// ```
pub fn request_tageswort(config: &Config) -> Result<String, TageswortError> {
    let body = reqwest::blocking::get(config.url.clone())?.text()?;
    decode_tageswort_response(&body)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_text() -> &'static str {
        "\
Dankbarkeit
Es ist schwer einzusehen, warum wir überschwänglich dankbar sein sollen für etwas, das wir nicht wollen, solange uns das, was wir wollen, vorenthalten wird.
Lisle de Vaux Matthewman
(1867 - 1903), Journalist und Schriftsteller
232285
11669
"
    }

    #[test]
    fn test_decode_tageswort_response() {
        let encoded = urlencoding::encode(sample_text()).into_owned();
        let decoded = decode_tageswort_response(&encoded).unwrap();
        assert_eq!(decoded, sample_text());
    }

    #[test]
    fn test_parse_tageswort_from_response() {
        let text = String::from(sample_text());
        let tageswort = parse_tageswort_from_response(text).unwrap();
        assert_eq!(tageswort.text, "Dankbarkeit\nEs ist schwer einzusehen, warum wir überschwänglich dankbar sein sollen für etwas, das wir nicht wollen, solange uns das, was wir wollen, vorenthalten wird.\nLisle de Vaux Matthewman\n(1867 - 1903), Journalist und Schriftsteller");
        assert_eq!(tageswort.link, "https://aphorismen.de/zitat/232285");
    }
}
