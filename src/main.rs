mod renderer;
mod scraper;

use std::path::PathBuf;

use clap::Parser;

use crate::renderer::Renderer;
use crate::scraper::Scraper;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DocItemKind {
    Class,
    TypeAlias,
    Interface,
    Function,
    Var,
}

#[derive(Debug)]
pub struct DocItem {
    pub name: String,
    pub kind: DocItemKind,
    pub description: Option<String>,
}

impl DocItem {
    pub fn filepath(&self) -> PathBuf {
        let tag = match self.kind {
            DocItemKind::Class => "class",
            DocItemKind::TypeAlias => "type",
            DocItemKind::Interface => "interface",
            DocItemKind::Function => "function",
            DocItemKind::Var => "var",
        };

        PathBuf::from(format!("{}.{}.html", tag, self.name))
    }

    pub fn short_description(&self) -> Option<String> {
        self.description.as_ref().map(|description| {
            description
                .lines()
                .take_while(|line| !line.is_empty())
                .collect::<Vec<_>>()
                .join(" ")
        })
    }
}

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    input: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let output_dir = PathBuf::from("output");

    if !output_dir.exists() {
        std::fs::create_dir_all(&output_dir)?;
    }

    let mut scraper = Scraper::new();
    scraper.scrape(&shellexpand::tilde(&args.input))?;

    let renderer = Renderer::new(output_dir);
    scraper.render(&renderer)?;

    Ok(())
}
