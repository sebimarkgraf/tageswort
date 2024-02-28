use reqwest;
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

    pub fn default() -> Config {
        let default_url = env::var("TAGESWORT_URL").unwrap_or(String::from(
            "https://assets.aphorismen.de/tagesspruch/tageswort.txt",
        ));
        return Config::new(default_url);
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

pub fn parse_tageswort_from_response(text: String) -> Result<Tageswort, TageswortError> {
    let lines: Vec<&str> = text.split("\n").collect();
    let tageswort = Tageswort {
        text: lines[0..lines.len() - 3].join("\n"),
        link: String::from("https://aphorismen.de/zitat/") + lines[lines.len() - 3],
    };
    return Ok(tageswort);
}

pub fn request_tageswort(config: &Config) -> Result<String, TageswortError> {
    let body = reqwest::blocking::get(config.url.clone())?.text()?;
    let text = decode(&body)?.into_owned();
    println!("{}", text);
    return Ok(text);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tageswort_from_response() {
        let text = String::from("\
Dankbarkeit
Es ist schwer einzusehen, warum wir überschwänglich dankbar sein sollen für etwas, das wir nicht wollen, solange uns das, was wir wollen, vorenthalten wird.
Lisle de Vaux Matthewman
(1867 - 1903), Journalist und Schriftsteller
232285
11669
");
        let tageswort = parse_tageswort_from_response(text).unwrap();
        assert_eq!(tageswort.text, "Dankbarkeit\nEs ist schwer einzusehen, warum wir überschwänglich dankbar sein sollen für etwas, das wir nicht wollen, solange uns das, was wir wollen, vorenthalten wird.\nLisle de Vaux Matthewman\n(1867 - 1903), Journalist und Schriftsteller");
        assert_eq!(tageswort.link, "https://aphorismen.de/zitat/232285");
    }
}
