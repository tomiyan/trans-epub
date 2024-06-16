# trans-epub

[![Crates.io](https://img.shields.io/crates/v/trans-epub.svg)](https://crates.io/crates/trans-epub)
[![Docs.rs](https://docs.rs/trans-epub/badge.svg)](https://docs.rs/trans-epub)
[![CI](https://github.com/tomiyan/trans-epub/workflows/CI/badge.svg)](https://github.com/tomiyan/trans-epub/actions)
[![Rust GitHub Template](https://img.shields.io/badge/Rust%20GitHub-Template-blue)](https://rust-github.github.io/)

This is a CLI tool to translate EPUB using OpenAI API.

## CAUTION

- If a translated book is available, we strongly recommend that you purchase it.
- This is only a tool to assist in reading books that have not been translated.
- Although the API is called in parallel, the translation takes a long time because of the ratelimit.
- Also, although the translation is done in units of 20 lines, if the number of lines does not match the original text and the translated text, the API is called again for each line, which is more expensive than simply translating the text.


![Translate sample](./docs/images/translate_sample.png)

## Installation

### Cargo

* Install the rust toolchain in order to have cargo installed by following
  [this](https://www.rust-lang.org/tools/install) guide.
* run `cargo install trans-epub`

## Execution

Install

```bash
curl -OL https://github.com/tomiyan/trans-epub/releases/download/0.0.9/trans-epub-0.0.9-macos-arm64.tar.gz
tar xvzf trans-epub-0.0.9-macos-arm64.tar.gz
```

Use Open AI help

```bash
./trans-epub open-ai --help
Use OpenAI API

Usage: trans-epub open-ai [OPTIONS] --input <INPUT> --output <OUTPUT> --language <LANGUAGE> --api-key <API_KEY>

Options:
  -i, --input <INPUT>        input file path
  -o, --output <OUTPUT>      output file path
  -l, --language <LANGUAGE>  translate language
  -m, --model <MODEL>        OpenAI model ex(gpt-4o, gpt-4-turbo, gpt-3.5-turbo-1106) [default: gpt-4o]
  -a, --api-key <API_KEY>    OpenAI API Key [env: API_KEY]
      --lines <LINES>        Number of lines of translation [default: 20]
      --requests <REQUESTS>  Number of concurrent requests [default: 5]
  -h, --help                 Print help
```

Use Open AI translate

```bash
export API_KEY=sk-....
./trans-epub open-ai -i ./origin.epub -o ./translated.epub -l Japanese
```

Wait a few minutes.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Thanks

Inspired by [epub-translator](https://github.com/sharplab/epub-translator)