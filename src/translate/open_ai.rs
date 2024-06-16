use crate::client::open_ai::{request, Ratelimit, Stats};
use futures::{stream, StreamExt};
use log::{debug, error, trace};
use serde::Deserialize;

#[derive(Deserialize)]
struct ChoiceContent {
    results: Vec<ChoiceContentResult>,
}

#[derive(Deserialize)]
struct ChoiceContentResult {
    translated: Vec<String>,
}

pub struct OpenAi {
    model: String,
    api_key: String,
    language: String,
    lines: usize,
    requests: usize,
}

pub struct BulkTranslated {
    pub number: i32,
    pub original_lines: Vec<String>,
    pub translated_lines: Vec<String>,
    pub stats: Stats,
    pub ratelimit: Ratelimit,
}

impl OpenAi {
    pub fn new(
        api_key: String,
        model: String,
        language: String,
        lines: usize,
        requests: usize,
    ) -> Self {
        Self {
            api_key,
            model,
            language,
            lines,
            requests,
        }
    }

    pub async fn translate(&mut self, lines: Vec<String>) -> Vec<String> {
        debug!("line_length:{}", lines.len());
        if lines.is_empty() {
            return lines;
        }
        translate_parallel(
            &self.language,
            &self.model,
            &self.api_key,
            lines,
            self.lines,
            self.requests,
            0,
        )
        .await
    }
}

async fn translate_parallel(
    language: &String,
    model: &String,
    api_key: &String,
    lines: Vec<String>,
    chunk_lines: usize,
    requests: usize,
    retry_count: i32,
) -> Vec<String> {
    let mut number = 0;
    let bodies = stream::iter(lines.chunks(chunk_lines))
        .map(|chunked| {
            let language = language.clone();
            let model = model.clone();
            let api_key = api_key.clone();
            number += 1;
            let order_number = number;
            async move {
                translate_bulk(order_number, &language, &model, &api_key, chunked.to_vec()).await
            }
        })
        .buffer_unordered(requests);

    let mut responses = vec![];
    let mut bodies_stream = bodies;
    while let Some(response) = bodies_stream.next().await {
        responses.push(response);
    }

    responses.sort_by(|a, b| a.number.cmp(&b.number));
    let mut translated = vec![];
    for response in responses {
        let mut translated_lines = response.translated_lines;
        let original_lines = response.original_lines;
        response.stats.log();
        response.ratelimit.log();
        if translated_lines.len() != original_lines.len() {
            if retry_count > 4 {
                panic!("retry max error");
            }
            for l in &original_lines {
                trace!("{}", l);
            }
            for l in &translated_lines {
                trace!("{}", l);
            }
            error!("retry count: {}", retry_count);
            error!(
                "translated line length error {}/{}",
                translated_lines.len(),
                original_lines.len()
            );
            translated_lines = Box::pin(translate_parallel(
                language,
                model,
                api_key,
                original_lines,
                1,
                requests,
                retry_count + 1,
            ))
            .await;
        }
        translated.append(&mut translated_lines);
    }
    translated
}

async fn translate_bulk(
    number: i32,
    language: &String,
    model: &str,
    api_key: &String,
    original_lines: Vec<String>,
) -> BulkTranslated {
    let mut user_contents: Vec<String> = vec![];
    for line in &original_lines {
        user_contents.push(format!("<paragraph>{}</paragraph>", line));
    }

    let prompt = format!("You are an excellent translator.\
        Translate it into {}. Please output the following JSON.\
        A string in `<paragraph>` tag to `</paragraph>` tag is one paragraph.\
        The value of the `results` Key is an array type.\
        Please output one line for each paragraph entered.\
        There are {} paragraphs of input, please output {} lines.\
        The value of `line` Key is a number type.\
        Please output the number of the input paragraph.\
        The value of `translated` Key is an array of String type.\
        If a paragraph of input is translated and a paragraph consists of multiple sentences, output an array consisting of multiple String.\
        Please remove `<paragraph>` and `</paragraph>` tags from the translation result.", language, &original_lines.len(), &original_lines.len());

    let response = request(model, api_key, &prompt, &user_contents)
        .await
        .expect("OpenAI API Request Error");
    let choice_content = serde_json::from_str::<ChoiceContent>(response.choice.trim());
    if choice_content.is_err() {
        error!("JSON Parse error choice:{}", &response.choice.trim());
        return BulkTranslated {
            number,
            original_lines,
            translated_lines: vec![],
            stats: response.stats,
            ratelimit: response.ratelimit,
        };
    }
    let mut translated_lines = vec![];
    for result in choice_content.unwrap().results {
        translated_lines.push(result.translated.join("\n"));
    }

    BulkTranslated {
        number,
        original_lines,
        translated_lines,
        stats: response.stats,
        ratelimit: response.ratelimit,
    }
}
