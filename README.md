# Tageswort

`tageswort` is a small Rust CLI and library that fetches the daily word entry
from aphorismen.de, parses the response, and prints a readable version to the
terminal.

The command-line interface is intentionally simple: running `tageswort` fetches
today's entry and writes the formatted text to stdout. The library API exposes
the parsed quote text and source link for integrations that need more than the
plain terminal output.

## Features

- Fetches the daily word from aphorismen.de.
- Decodes the URL-encoded response payload.
- Parses the title, quote, attribution, and aphorismen.de quote link.
- Formats the result for terminal output.
- Caches the fetched response for the current day.
- Falls back to today's cache if the network request fails.
- Prints a built-in offline fallback when neither the network nor today's cache
  is available.

## CLI overview

The CLI has no flags or subcommands at the moment. It reads configuration from
the environment, fetches the current daily text, and prints the formatted result:

```sh
tageswort
```

Example output:

```text
Dankbarkeit

> Es ist schwer einzusehen, warum wir ueberschwaenglich dankbar sein sollen ...

— Lisle de Vaux Matthewman
  (1867 - 1903), Journalist und Schriftsteller
```

If fetching or parsing fails in a way that cannot be recovered from, the command
prints an error to stderr and exits with status code `1`.

## Configuration

By default, `tageswort` fetches from:

```text
https://assets.aphorismen.de/tagesspruch/tageswort.txt
```

Set `TAGESWORT_URL` to use another compatible endpoint:

```sh
TAGESWORT_URL="https://example.com/tageswort.txt" tageswort
```

The endpoint must return the same URL-encoded, line-based format used by
aphorismen.de. After decoding, the payload is expected to contain:

1. A text block with title, quote text, author, and attribution details
2. A quote id used for the aphorismen.de link
3. An additional footer id

## Cache and offline behavior

Successful fetches are cached below the platform cache directory in a
`tageswort` folder. Cache file names use the current date:

```text
<cache-dir>/tageswort/YYYY-MM-DD.txt
```

On a network failure, `tageswort` first tries to read today's cached response. If
there is no valid cache entry for today, it prints the built-in offline fallback:

```text
No network today. The quote couldn't make the net work.
```

Cache write failures do not fail the command as long as the freshly fetched
quote can still be displayed.

## Install and run

From a local checkout:

```sh
cargo run
```

Build the release binary:

```sh
cargo build --release
```

Install it from the checkout into Cargo's binary directory:

```sh
cargo install --path .
```

If you use Nix flakes, the default app can be run with:

```sh
nix run
```

## Development

Common development commands:

```sh
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
```

The flake also defines checks for formatting, clippy, and tests:

```sh
nix flake check
```

## Library usage

The crate exposes the same core behavior used by the CLI:

```rust
use tageswort::{get_tageswort, Config, TageswortError};

fn main() -> Result<(), TageswortError> {
    let config = Config::default();
    let tageswort = get_tageswort(&config)?;

    println!("{}", tageswort);
    println!("Source: {}", tageswort.link);

    Ok(())
}
```

For tests and integrations that already have a response body, use
`decode_tageswort_response` and `parse_tageswort_from_response` directly.
