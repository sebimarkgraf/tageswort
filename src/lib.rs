//! # tageswort
//! `tageswort` is a library for fetching the daily word of the day from aphorismen.de.
//! It provides a simple API to fetch the word of the day and parse it into a struct.
//! The struct contains the text of the word of the day and a link to the aphorismen.de website.
//!
//! The library is built on top of reqwest for fetching the word of the day and urlencoding for decoding the response.

use chrono::{Local, NaiveDate};
use std::env;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};
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
    ParseError,
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
        write!(f, "{}", format_tageswort_for_display(&self.text))
    }
}

const OFFLINE_FALLBACK_QUOTE: &str = "No network today. The quote couldn't make the net work.";

/// Decodes the raw response body returned by aphorismen.de into plain text.
pub fn decode_tageswort_response(body: &str) -> Result<String, TageswortError> {
    let text = decode(body)?.into_owned();
    Ok(text)
}

pub fn parse_tageswort_from_response(text: String) -> Result<Tageswort, TageswortError> {
    let lines: Vec<&str> = text
        .lines()
        .skip_while(|line| line.trim().is_empty())
        .collect();

    let trailing_blank_lines = lines
        .iter()
        .rev()
        .take_while(|line| line.trim().is_empty())
        .count();
    let lines = &lines[..lines.len().saturating_sub(trailing_blank_lines)];

    if lines.len() < 3 {
        return Err(TageswortError::ParseError);
    }

    let link_id = lines[lines.len() - 2];
    if link_id.trim().is_empty() || lines[..lines.len() - 2].is_empty() {
        return Err(TageswortError::ParseError);
    }

    let tageswort = Tageswort {
        text: lines[..lines.len() - 2].join("\n"),
        link: String::from("https://aphorismen.de/zitat/") + link_id,
    };
    Ok(tageswort)
}

fn format_tageswort_for_display(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() < 4 {
        return strip_html_break_tags(text);
    }

    let title = strip_html_break_tags(lines[0]);
    let quote_lines: Vec<String> = lines[1..lines.len() - 2]
        .iter()
        .map(|line| strip_html_break_tags(line))
        .collect();
    let attribution_name = strip_html_break_tags(lines[lines.len() - 2]);
    let attribution_detail = strip_html_break_tags(lines[lines.len() - 1]);

    if title.trim().is_empty() || quote_lines.is_empty() {
        return strip_html_break_tags(text);
    }

    let mut output = String::new();
    output.push_str(&title);
    output.push_str("\n\n");

    for line in quote_lines {
        output.push_str("> ");
        output.push_str(&line);
        output.push('\n');
    }

    output.push('\n');
    output.push_str("— ");
    output.push_str(&attribution_name);

    if !attribution_detail.trim().is_empty() {
        output.push('\n');
        output.push_str("  ");
        output.push_str(&attribution_detail);
    }

    output
}

fn strip_html_break_tags(text: &str) -> String {
    text.replace("<br />", "")
        .replace("<br/>", "")
        .replace("<br>", "")
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
/// assert_eq!(tageswort.to_string(), "Dankbarkeit\n\n> Es ist schwer einzusehen, warum wir überschwänglich dankbar sein sollen für etwas, das wir nicht wollen, solange uns das, was wir wollen, vorenthalten wird.\n\n— Lisle de Vaux Matthewman\n  (1867 - 1903), Journalist und Schriftsteller");
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

pub fn get_tageswort(config: &Config) -> Result<Tageswort, TageswortError> {
    get_tageswort_for_date_with_fetch(
        config,
        default_cache_root(),
        Local::now().date_naive(),
        request_tageswort,
    )
}

fn get_tageswort_for_date_with_fetch<F>(
    config: &Config,
    cache_root: Option<PathBuf>,
    today: NaiveDate,
    fetch_text: F,
) -> Result<Tageswort, TageswortError>
where
    F: FnOnce(&Config) -> Result<String, TageswortError>,
{
    match fetch_text(config) {
        Ok(text) => {
            let tageswort = parse_tageswort_from_response(text.clone())?;
            if let Some(cache_root) = cache_root.as_deref() {
                let _ = write_cached_tageswort(cache_root, today, &text);
            }
            Ok(tageswort)
        }
        Err(TageswortError::Reqwest(_)) => {
            if let Some(cache_root) = cache_root.as_deref() {
                if let Some(tageswort) = read_cached_tageswort(cache_root, today) {
                    return Ok(tageswort);
                }
            }
            Ok(offline_fallback_tageswort())
        }
        Err(err) => Err(err),
    }
}

fn default_cache_root() -> Option<PathBuf> {
    dirs::cache_dir().map(|path| path.join("tageswort"))
}

fn cache_file_path(cache_root: &Path, today: NaiveDate) -> PathBuf {
    cache_root.join(format!("{}.txt", today.format("%Y-%m-%d")))
}

fn write_cached_tageswort(
    cache_root: &Path,
    today: NaiveDate,
    text: &str,
) -> Result<(), std::io::Error> {
    fs::create_dir_all(cache_root)?;
    fs::write(cache_file_path(cache_root, today), text)
}

fn read_cached_tageswort(cache_root: &Path, today: NaiveDate) -> Option<Tageswort> {
    let text = fs::read_to_string(cache_file_path(cache_root, today)).ok()?;
    parse_cached_tageswort(text)
}

fn parse_cached_tageswort(text: String) -> Option<Tageswort> {
    parse_tageswort_from_response(text).ok()
}

fn offline_fallback_tageswort() -> Tageswort {
    Tageswort {
        text: OFFLINE_FALLBACK_QUOTE.to_string(),
        link: String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

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

    fn expected_text() -> &'static str {
        "Dankbarkeit\nEs ist schwer einzusehen, warum wir überschwänglich dankbar sein sollen für etwas, das wir nicht wollen, solange uns das, was wir wollen, vorenthalten wird.\nLisle de Vaux Matthewman\n(1867 - 1903), Journalist und Schriftsteller"
    }

    fn expected_display_text() -> &'static str {
        "Dankbarkeit\n\n> Es ist schwer einzusehen, warum wir überschwänglich dankbar sein sollen für etwas, das wir nicht wollen, solange uns das, was wir wollen, vorenthalten wird.\n\n— Lisle de Vaux Matthewman\n  (1867 - 1903), Journalist und Schriftsteller"
    }

    fn assert_sample_parse(text: String) {
        let tageswort = parse_tageswort_from_response(text).unwrap();
        assert_eq!(tageswort.text, expected_text());
        assert_eq!(tageswort.link, "https://aphorismen.de/zitat/232285");
    }

    #[test]
    fn test_decode_tageswort_response() {
        let encoded = urlencoding::encode(sample_text()).into_owned();
        let decoded = decode_tageswort_response(&encoded).unwrap();
        assert_eq!(decoded, sample_text());
    }

    #[test]
    fn test_parse_tageswort_from_response() {
        assert_sample_parse(String::from(sample_text()));
    }

    #[test]
    fn test_parse_tageswort_without_trailing_newline() {
        assert_sample_parse(sample_text().trim_end_matches('\n').to_string());
    }

    #[test]
    fn test_parse_tageswort_with_crlf_line_endings() {
        assert_sample_parse(sample_text().replace('\n', "\r\n"));
    }

    #[test]
    fn test_parse_tageswort_ignores_outer_blank_lines() {
        let text = format!("\n\r\n{}\n\r\n", sample_text());
        assert_sample_parse(text);
    }

    #[test]
    fn test_parse_tageswort_rejects_too_short_payload() {
        assert!(matches!(
            parse_tageswort_from_response("Dankbarkeit\n232285".to_string()),
            Err(TageswortError::ParseError)
        ));
    }

    #[test]
    fn test_parse_tageswort_rejects_missing_link_footer() {
        let text = "\
Dankbarkeit
Es ist schwer einzusehen, warum wir überschwänglich dankbar sein sollen für etwas, das wir nicht wollen, solange uns das, was wir wollen, vorenthalten wird.
Lisle de Vaux Matthewman
(1867 - 1903), Journalist und Schriftsteller

11669
";
        assert!(matches!(
            parse_tageswort_from_response(text.to_string()),
            Err(TageswortError::ParseError)
        ));
    }

    #[test]
    fn test_display_formats_title_quote_and_attribution() {
        let tageswort = parse_tageswort_from_response(sample_text().to_string()).unwrap();

        assert_eq!(tageswort.to_string(), expected_display_text());
    }

    #[test]
    fn test_display_strips_html_break_variants() {
        let tageswort = Tageswort {
            text: "\
Leben
Hinweis:<br />
Zeile zwei<br/> und drei<br>
© KarlHeinz Karius
(*1935), Urheber, Mensch und Werbeberater"
                .to_string(),
            link: "https://aphorismen.de/zitat/213695".to_string(),
        };

        assert_eq!(
            tageswort.to_string(),
            "Leben\n\n> Hinweis:\n> Zeile zwei und drei\n\n— © KarlHeinz Karius\n  (*1935), Urheber, Mensch und Werbeberater"
        );
    }

    fn test_cache_root() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            env::temp_dir().join(format!("tageswort-tests-{}-{}", std::process::id(), unique));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn test_write_and_read_cached_tageswort_for_today() {
        let cache_root = test_cache_root();
        let today = NaiveDate::from_ymd_opt(2026, 4, 28).unwrap();

        write_cached_tageswort(&cache_root, today, sample_text()).unwrap();
        let tageswort = read_cached_tageswort(&cache_root, today).unwrap();
        let expected = parse_tageswort_from_response(sample_text().to_string()).unwrap();

        assert_eq!(tageswort.text, expected.text);
        assert_eq!(tageswort.link, expected.link);

        fs::remove_dir_all(cache_root).unwrap();
    }

    #[test]
    fn test_read_cached_tageswort_returns_none_when_missing() {
        let cache_root = test_cache_root();
        let today = NaiveDate::from_ymd_opt(2026, 4, 28).unwrap();

        assert!(read_cached_tageswort(&cache_root, today).is_none());

        fs::remove_dir_all(cache_root).unwrap();
    }

    #[test]
    fn test_read_cached_tageswort_treats_malformed_cache_as_miss() {
        let cache_root = test_cache_root();
        let today = NaiveDate::from_ymd_opt(2026, 4, 28).unwrap();

        write_cached_tageswort(&cache_root, today, "not enough lines").unwrap();

        assert!(read_cached_tageswort(&cache_root, today).is_none());

        fs::remove_dir_all(cache_root).unwrap();
    }

    #[test]
    fn test_read_cached_tageswort_ignores_yesterday() {
        let cache_root = test_cache_root();
        let today = NaiveDate::from_ymd_opt(2026, 4, 28).unwrap();
        let yesterday = NaiveDate::from_ymd_opt(2026, 4, 27).unwrap();

        write_cached_tageswort(&cache_root, yesterday, sample_text()).unwrap();

        assert!(read_cached_tageswort(&cache_root, today).is_none());

        fs::remove_dir_all(cache_root).unwrap();
    }

    #[test]
    fn test_get_tageswort_returns_fetched_quote_and_writes_cache() {
        let cache_root = test_cache_root();
        let today = NaiveDate::from_ymd_opt(2026, 4, 28).unwrap();
        let config = Config::new("https://example.invalid".to_string());

        let tageswort =
            get_tageswort_for_date_with_fetch(&config, Some(cache_root.clone()), today, |_| {
                Ok(sample_text().to_string())
            })
            .unwrap();
        let expected = parse_tageswort_from_response(sample_text().to_string()).unwrap();

        assert_eq!(tageswort.text, expected.text);
        assert_eq!(tageswort.link, expected.link);
        assert_eq!(
            fs::read_to_string(cache_file_path(&cache_root, today)).unwrap(),
            sample_text()
        );

        fs::remove_dir_all(cache_root).unwrap();
    }

    #[test]
    fn test_get_tageswort_returns_cached_quote_when_fetch_fails() {
        let cache_root = test_cache_root();
        let today = NaiveDate::from_ymd_opt(2026, 4, 28).unwrap();
        let config = Config::new("https://example.invalid".to_string());

        write_cached_tageswort(&cache_root, today, sample_text()).unwrap();

        let tageswort =
            get_tageswort_for_date_with_fetch(&config, Some(cache_root.clone()), today, |_| {
                Err(reqwest::blocking::get("http://127.0.0.1:9")
                    .unwrap_err()
                    .into())
            })
            .unwrap();
        let expected = parse_tageswort_from_response(sample_text().to_string()).unwrap();

        assert_eq!(tageswort.text, expected.text);
        assert_eq!(tageswort.link, expected.link);

        fs::remove_dir_all(cache_root).unwrap();
    }

    #[test]
    fn test_get_tageswort_returns_offline_fallback_when_fetch_fails_without_cache() {
        let cache_root = test_cache_root();
        let today = NaiveDate::from_ymd_opt(2026, 4, 28).unwrap();
        let config = Config::new("https://example.invalid".to_string());

        let tageswort =
            get_tageswort_for_date_with_fetch(&config, Some(cache_root.clone()), today, |_| {
                Err(reqwest::blocking::get("http://127.0.0.1:9")
                    .unwrap_err()
                    .into())
            })
            .unwrap();

        assert_eq!(tageswort.text, OFFLINE_FALLBACK_QUOTE);
        assert_eq!(tageswort.link, "");

        fs::remove_dir_all(cache_root).unwrap();
    }

    #[test]
    fn test_get_tageswort_returns_fetched_quote_when_cache_write_fails() {
        let cache_root = test_cache_root();
        let today = NaiveDate::from_ymd_opt(2026, 4, 28).unwrap();
        let config = Config::new("https://example.invalid".to_string());
        let blocking_path = cache_root.join("not-a-directory");

        fs::write(&blocking_path, "blocking").unwrap();

        let tageswort =
            get_tageswort_for_date_with_fetch(&config, Some(blocking_path), today, |_| {
                Ok(sample_text().to_string())
            })
            .unwrap();

        assert_eq!(tageswort.link, "https://aphorismen.de/zitat/232285");

        fs::remove_dir_all(cache_root).unwrap();
    }
}
