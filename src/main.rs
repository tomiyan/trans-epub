mod client;
mod epub;
mod translate;

use crate::epub::Epub;
use crate::translate::translator::{Context, Translator};
use clap::{Parser, Subcommand};
use env_logger::Env;
use log::debug;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[clap(subcommand)]
    subcommand: SubCommands,
}

#[derive(Subcommand)]
enum SubCommands {
    /// Use OpenAI API
    OpenAi {
        /// input file path
        #[arg(short, long)]
        input: PathBuf,

        /// output file path
        #[arg(short, long)]
        output: PathBuf,

        /// translate language
        #[arg(short, long)]
        language: String,

        /// OpenAI model ex(gpt-4o-mini, gpt-4o, gpt-4-turbo, gpt-3.5-turbo-1106)
        #[arg(short, long, default_value_t = String::from("gpt-4o-mini"))]
        model: String,

        /// OpenAI API Key
        #[arg(short, long, env, hide_env_values = true)]
        api_key: String,

        /// Number of lines of translation
        #[arg(long, default_value_t = 20)]
        lines: usize,

        /// Number of concurrent requests
        #[arg(long, default_value_t = 5)]
        requests: usize,
    },
    /// Use Gemini API
    Gemini {
        /// input file path
        #[arg(short, long)]
        input: PathBuf,

        /// output file path
        #[arg(short, long)]
        output: PathBuf,

        /// translate language
        #[arg(short, long)]
        language: String,

        /// Gemini model ex(gemini-2.0-flash-lite, gemini-1.5-flash)
        #[arg(short, long, default_value_t = String::from("gemini-2.0-flash-lite"))]
        model: String,

        /// Gemini API Key
        #[arg(short, long, env, hide_env_values = true)]
        api_key: String,

        /// Number of lines of translation
        #[arg(long, default_value_t = 100)]
        lines: usize,

        /// Number of concurrent requests
        #[arg(long, default_value_t = 1)]
        requests: usize,
    },
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    debug!("start");
    let args = Args::parse();
    match args.subcommand {
        SubCommands::OpenAi {
            api_key,
            model,
            language,
            lines,
            requests,
            input,
            output,
        } => {
            let translator = Translator::OpenAi(Context {
                model,
                api_key,
                language,
                lines,
                requests,
            });
            let epub = Epub::new(input, output);
            epub.translate(translator).await;
        }
        SubCommands::Gemini {
            api_key,
            model,
            language,
            lines,
            requests,
            input,
            output,
        } => {
            let translator = Translator::Gemini(Context {
                model,
                api_key,
                language,
                lines,
                requests,
            });
            let epub = Epub::new(input, output);
            epub.translate(translator).await;
        }
    }
    debug!("end");
}
