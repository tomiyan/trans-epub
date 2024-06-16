use crate::translate::open_ai::OpenAi;
use log::{debug, info};
use quick_xml::events::{BytesText, Event};
use quick_xml::{Reader, Writer};
use regex::Regex;
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::path::PathBuf;
use zip::write::SimpleFileOptions;
use zip::ZipArchive;

pub struct Epub {
    input_path: PathBuf,
    output_path: PathBuf,
}

impl Epub {
    pub fn new(input_path: PathBuf, output_path: PathBuf) -> Self {
        Self {
            input_path,
            output_path,
        }
    }

    pub async fn translate(self, mut open_ai: OpenAi) {
        debug!("translate start");
        let input_file = File::open(self.input_path).expect("input file open fail");
        let mut archive = ZipArchive::new(input_file).expect("input file unzip fail");

        let mut file_contents = Vec::new();
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).unwrap();
            file_contents.push((file.name().to_string(), buffer));
        }

        let mut translated_contents = Vec::new();
        let mut count = 1;
        let size = file_contents.len();
        for (name, content) in file_contents {
            info!("{}/{} {}", count, size, name);
            count += 1;
            if name.ends_with(".xhtml")
                || name.ends_with(".xml")
                || name.ends_with(".html")
                || name.ends_with(".htm")
            {
                let content = strip_xml_content(&content);
                let lines = translate_lines(&content).await;
                let lines = open_ai.translate(lines).await;
                let translated_content = translate_xml_content(lines, &content).await;
                translated_contents.push((name, translated_content));
            } else {
                translated_contents.push((name, content));
            }
        }
        debug!("translate end");

        debug!("output file start");
        let file = File::create(self.output_path).expect("output file open fail");
        let mut zip = zip::ZipWriter::new(file);

        for (name, content) in translated_contents {
            zip.start_file(name, SimpleFileOptions::default())
                .expect("output file zip fail");
            zip.write_all(&content).expect("output file zip fail");
        }

        zip.finish().expect("output file zip fail");
        debug!("output file end");
    }
}

fn strip_xml_content(content: &[u8]) -> Vec<u8> {
    let mut reader = Reader::from_reader(content);
    reader.config_mut().trim_text(true);

    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut is_rt = false;
    loop {
        match reader.read_event() {
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => match e.name().0 {
                b"rt" => is_rt = true,
                b"ruby" => continue,
                _ => writer.write_event(Event::Start(e)).unwrap(),
            },
            Ok(Event::End(e)) => match e.name().0 {
                b"rt" => is_rt = false,
                b"ruby" => continue,
                _ => writer.write_event(Event::End(e)).unwrap(),
            },
            Ok(Event::Text(e)) if !is_rt => writer.write_event(Event::Text(e)).unwrap(),
            Ok(Event::Text(_)) if is_rt => continue,
            event => writer.write_event(event.unwrap()).unwrap(),
        }
    }
    writer.into_inner().into_inner()
}

async fn translate_lines(content: &[u8]) -> Vec<String> {
    let ignore_text = Regex::new(r"^[\s\p{Cc}\p{So}0-9[:punct:]–]*$").unwrap();
    let mut reader = Reader::from_reader(content);
    reader.config_mut().trim_text(true);

    let mut is_translate = false;
    let mut translate_tag: String = String::new();
    let mut depth = 0;
    let mut translate: String = String::new();
    let mut result = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                let tag = std::str::from_utf8(e.name().0).unwrap();
                match tag {
                    "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "li" => {
                        if !is_translate {
                            translate_tag = tag.to_string();
                            is_translate = true;
                            translate = String::new();
                        }
                        if *tag == translate_tag {
                            depth += 1;
                        }
                    }
                    _ => (),
                }
            }
            Ok(Event::End(e)) => {
                let tag = std::str::from_utf8(e.name().0).unwrap();
                match tag {
                    "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "li" => {
                        if *tag == translate_tag {
                            depth -= 1;
                            if depth == 0 {
                                is_translate = false;
                                if !ignore_text.is_match(&translate) {
                                    result.push(translate.clone());
                                }
                            }
                        }
                    }
                    _ => (),
                }
            }
            Ok(Event::Text(e)) => {
                let original_text = e.unescape().unwrap().into_owned();
                if is_translate {
                    translate.push_str(&original_text);
                }
            }
            _ => (),
        }
    }
    result
}

async fn translate_xml_content(lines: Vec<String>, content: &[u8]) -> Vec<u8> {
    let ignore_text = Regex::new(r"^[\s\p{Cc}\p{So}0-9[:punct:]–]*$").unwrap();
    let mut reader = Reader::from_reader(content);
    reader.config_mut().trim_text(true);

    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut is_translate = false;
    let mut translate_tag: String = String::new();
    let mut depth = 0;
    let mut translate: String = String::new();
    let mut index = 0;

    loop {
        match reader.read_event() {
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                let tag = std::str::from_utf8(e.name().0).unwrap();
                match tag {
                    "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "li" => {
                        if !is_translate {
                            translate_tag = tag.to_string();
                            is_translate = true;
                            translate = String::new();
                        }
                        if *tag == translate_tag {
                            depth += 1;
                        }
                        writer.write_event(Event::Start(e)).unwrap();
                    }
                    _ => writer.write_event(Event::Start(e)).unwrap(),
                }
            }
            Ok(Event::End(e)) => {
                let tag = std::str::from_utf8(e.name().0).unwrap();
                match tag {
                    "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "li" => {
                        if *tag == translate_tag {
                            depth -= 1;
                            if depth == 0 {
                                is_translate = false;
                                if !ignore_text.is_match(&translate) {
                                    writer
                                        .write_event(Event::Text(BytesText::new("<<")))
                                        .unwrap();
                                    writer
                                        .write_event(Event::Text(BytesText::new(
                                            lines.get(index).unwrap(),
                                        )))
                                        .unwrap();
                                    writer
                                        .write_event(Event::Text(BytesText::new(">>")))
                                        .unwrap();
                                    index += 1;
                                }
                            }
                        }
                        writer.write_event(Event::End(e)).unwrap();
                    }
                    _ => writer.write_event(Event::End(e)).unwrap(),
                }
            }
            Ok(Event::Text(e)) => {
                let original_text = e.unescape().unwrap().into_owned();
                if is_translate {
                    translate.push_str(&original_text);
                }
                writer
                    .write_event(Event::Text(BytesText::new(&original_text)))
                    .unwrap();
            }
            event => writer.write_event(event.unwrap()).unwrap(),
        }
    }
    writer.into_inner().into_inner()
}
