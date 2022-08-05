mod renderer;
mod scraper;

use std::path::PathBuf;

use crate::renderer::Renderer;
use crate::scraper::Scraper;

#[derive(Debug)]
pub enum DocItemKind {
    Class,
    TypeAlias,
    Interface,
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
        };

        PathBuf::from(format!("{}.{}.html", tag, self.name))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = PathBuf::from("output");

    if !output_dir.exists() {
        std::fs::create_dir_all(&output_dir)?;
    }

    let mut scraper = Scraper::new();
    scraper.scrape("/Users/maxdeviant/projects/thaumaturge/src/**/*.ts")?;

    let renderer = Renderer::new(output_dir);
    scraper.render(&renderer)?;

    Ok(())
}
