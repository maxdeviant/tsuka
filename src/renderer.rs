use std::fs::File;
use std::path::PathBuf;

use indexmap::IndexMap;
use maud::{html, Markup, PreEscaped, Render, DOCTYPE};
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

        let mut items_by_kind = IndexMap::new();

        let kind_order = &[
            DocItemKind::Class,
            DocItemKind::Interface,
            DocItemKind::TypeAlias,
        ];

        for kind in *kind_order {
            items_by_kind.entry(kind).or_insert(Vec::new());
        }

        for item in items {
            let output_filepath = self.output_dir.join(&item.filepath());
            let mut output_file = File::create(&output_filepath)?;
            output_file.write_all(
                DocItemPage { item: &item }
                    .render()
                    .into_string()
                    .as_bytes(),
            )?;

            items_by_kind
                .entry(item.kind)
                .or_insert(Vec::new())
                .push(item);
        }

        let index_page = IndexPage { items_by_kind };

        use std::io::Write;

        let output_filepath = self.output_dir.join("index.html");
        let mut output_file = File::create(&output_filepath)?;
        output_file.write_all(index_page.render().into_string().as_bytes())?;

        Ok(())
    }
}

struct Markdown<T: AsRef<str>>(T);

impl<T: AsRef<str>> Render for Markdown<T> {
    fn render(&self) -> Markup {
        let mut options = markdown::Options::empty();
        options.insert(markdown::Options::ENABLE_STRIKETHROUGH);
        options.insert(markdown::Options::ENABLE_TABLES);

        let mut unsafe_html = String::new();
        let parser = markdown::Parser::new_ext(self.0.as_ref(), options);
        markdown::html::push_html(&mut unsafe_html, parser);

        let safe_html = ammonia::clean(&unsafe_html);
        PreEscaped(safe_html)
    }
}

struct Page<'a> {
    title: &'a str,
    content: Markup,
}

impl<'a> Render for Page<'a> {
    fn render(&self) -> Markup {
        html! {
            (DOCTYPE)
            head {
                meta charset="utf-8";
                title { (self.title) }
                link rel="stylesheet" href="https://unpkg.com/tachyons@4.12.0/css/tachyons.min.css";
            }
            body.light-gray.bg-dark-blue.avenir {
                (self.content)
            }
        }
    }
}

struct IndexPage {
    items_by_kind: IndexMap<DocItemKind, Vec<DocItem>>,
}

impl Render for IndexPage {
    fn render(&self) -> Markup {
        Page {
            title: "Index",
            content: html! {
                @for (kind, items) in &self.items_by_kind {
                    h2 {
                        @match kind {
                            DocItemKind::Class => "Classes",
                            DocItemKind::TypeAlias => "Type Aliases",
                            DocItemKind::Interface => "Interfaces"
                        }
                    }
                    div.dt {
                        @for item in items {
                            div.dt-row {
                                div.dtc.pr3 {
                                    a.link.light-gray href=(item.filepath().display()) {
                                        (item.name)
                                    }
                                }
                                div.dtc {
                                    (Markdown(item.short_description().unwrap_or(String::new())))
                                }
                            }
                        }
                    }
                }
            },
        }
        .render()
    }
}

struct DocItemPage<'a> {
    item: &'a DocItem,
}

impl<'a> Render for DocItemPage<'a> {
    fn render(&self) -> Markup {
        Page {
            title: &self.item.name,
            content: html! {
                h1 { (self.item.name) }
                (Markdown(self.item.description.clone().unwrap_or(String::new())))
            },
        }
        .render()
    }
}