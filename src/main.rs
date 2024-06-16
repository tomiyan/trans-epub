mod client;
mod epub;
mod translate;

use crate::epub::Epub;
use crate::translate::open_ai::OpenAi;
use clap::{Parser, Subcommand};
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

        /// OpenAI model ex(gpt-4o, gpt-4-turbo, gpt-3.5-turbo-1106)
        #[arg(short, long, default_value_t = String::from("gpt-4o"))]
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
}

#[tokio::main]
async fn main() {
    env_logger::init();
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
            let open_ai = OpenAi::new(api_key, model, language, lines, requests);
            let epub = Epub::new(input, output);
            epub.translate(open_ai).await;
        }
    }
    debug!("end");
}
