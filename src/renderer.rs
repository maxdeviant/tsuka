use std::collections::HashMap;
use std::fmt::Write;
use std::fs::File;
use std::path::PathBuf;

use itertools::Itertools;
use pulldown_cmark as markdown;

use crate::{DocItem, DocItemKind};

pub struct Renderer {
    output_dir: PathBuf,
}

impl Renderer {
    pub fn new<P: Into<PathBuf>>(output_dir: P) -> Self {
        Self {
            output_dir: output_dir.into(),
        }
    }

    pub fn render(&self, items: Vec<DocItem>) -> Result<(), Box<dyn std::error::Error>> {
        let mut options = markdown::Options::empty();
        options.insert(markdown::Options::ENABLE_STRIKETHROUGH);
        options.insert(markdown::Options::ENABLE_TABLES);

        let mut output = String::new();
        writeln!(&mut output, "<!doctype html>")?;
        writeln!(&mut output, r#"<html lang="en">"#)?;
        writeln!(&mut output, "<head>")?;
        writeln!(&mut output, r#"<meta charset="utf-8">"#)?;
        writeln!(
            &mut output,
            r#"<link rel="stylesheet" href="https://unpkg.com/tachyons@4.12.0/css/tachyons.min.css" />"#
        )?;
        writeln!(&mut output, "</head>")?;
        writeln!(&mut output, r#"<body class="light-gray bg-dark-blue">"#)?;

        let items_by_kind = items.into_iter().into_group_map_by(|item| item.kind);

        for (kind, items) in items_by_kind {
            writeln!(&mut output, "<h2>{:?}</h2>", kind)?;

            writeln!(&mut output, r#"<div class="dt">"#)?;

            for item in items {
                writeln!(&mut output, r#"<div class="dt-row">"#)?;

                writeln!(&mut output, r#"<div class="dtc pr3">"#)?;
                writeln!(
                    &mut output,
                    r#"<a class="link light-gray" href="{href}">{}</a>"#,
                    item.name,
                    href = item.filepath().display(),
                )?;
                writeln!(&mut output, "</div>")?;

                writeln!(&mut output, r#"<div class="dtc">"#)?;

                let short_description = item
                    .description
                    .clone()
                    .unwrap_or(String::new())
                    .lines()
                    .next()
                    .map(|x| x.to_owned())
                    .unwrap_or(String::new());

                let parser = markdown::Parser::new_ext(&short_description, options);

                let mut short_description_html = String::new();
                markdown::html::push_html(&mut short_description_html, parser);

                writeln!(&mut output, "{}", short_description_html)?;
                writeln!(&mut output, "</div>")?;
                writeln!(&mut output, "</div>")?;

                let mut item_output = String::new();
                writeln!(&mut item_output, "<!doctype html>")?;
                writeln!(&mut item_output, r#"<html lang="en">"#)?;
                writeln!(&mut item_output, "<head>")?;
                writeln!(&mut item_output, r#"<meta charset="utf-8">"#)?;
                writeln!(
                    &mut item_output,
                    r#"<link rel="stylesheet" href="https://unpkg.com/tachyons@4.12.0/css/tachyons.min.css" />"#
                )?;
                writeln!(&mut item_output, "</head>")?;
                writeln!(&mut item_output, "<body>")?;
                writeln!(&mut item_output, "<h1>{}</h1>", item.name)?;

                let description = item.description.clone().unwrap_or(String::new());

                let parser = markdown::Parser::new_ext(&description, options);

                let mut description_html = String::new();
                markdown::html::push_html(&mut description_html, parser);

                writeln!(&mut item_output, "{}", description_html)?;
                writeln!(&mut item_output, "</body>")?;
                writeln!(&mut item_output, "</html>")?;

                let output_filepath = self.output_dir.join(&item.filepath());
                let mut output_file = File::create(&output_filepath)?;
                output_file.write_all(item_output.as_bytes())?;
            }

            writeln!(&mut output, "</div>")?;
        }

        writeln!(&mut output, "</body>")?;
        writeln!(&mut output, "</html>")?;

        use std::io::Write;

        let output_filepath = self.output_dir.join("index.html");
        let mut output_file = File::create(&output_filepath)?;
        output_file.write_all(output.as_bytes())?;

        Ok(())
    }
}
