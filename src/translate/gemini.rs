use crate::client::gemini::{request, Stats};
use crate::translate::translator::Context;
use futures::{stream, StreamExt};
use log::{debug, error, trace};
use serde::Deserialize;

#[derive(Deserialize)]
struct Translated {
    text: Vec<String>,
}

pub struct BulkTranslated {
    pub number: i32,
    pub original_lines: Vec<String>,
    pub translated_lines: Vec<String>,
    pub stats: Stats,
}

pub async fn translate(context: &Context, lines: Vec<String>) -> Vec<String> {
    debug!("line_length:{}", lines.len());
    if lines.is_empty() {
        return lines;
    }
    translate_parallel(
        &context.language,
        &context.model,
        &context.api_key,
        lines,
        context.lines,
        context.requests,
        0,
    )
    .await
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
        If a paragraph of input is translated and a paragraph consists of multiple sentences, output an array consisting of multiple String.\
        There are {} paragraphs of input, please output {} lines.\
        Using this JSON schema:\
        Paragraph = {{\"line\": number, \"text\": list[string]}}\
        Return a `list[Paragraph]`\
        Please remove `<paragraph>` and `</paragraph>` tags from the translation result.", language, &original_lines.len(), &original_lines.len());

    let response = request(model, api_key, &prompt, &user_contents)
        .await
        .expect("Gemini API Request Error");
    let translated_vec = serde_json::from_str::<Vec<Translated>>(response.text.trim());
    if translated_vec.is_err() {
        error!("JSON Parse error choice:{}", &response.text.trim());
        return BulkTranslated {
            number,
            original_lines,
            translated_lines: vec![],
            stats: response.stats,
        };
    }
    let mut translated_lines = vec![];
    for result in translated_vec.unwrap() {
        translated_lines.push(result.text.join("\n"));
    }

    BulkTranslated {
        number,
        original_lines,
        translated_lines,
        stats: response.stats,
    }
}
