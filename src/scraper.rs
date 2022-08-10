use glob::glob;
use swc::SwcComments;
use swc_common::comments::Comments;
use swc_common::errors::{ColorConfig, Handler};
use swc_common::sync::Lrc;
use swc_common::SourceMap;
use swc_ecma_ast::*;
use swc_ecma_parser::lexer::Lexer;
use swc_ecma_parser::{Parser, StringInput, Syntax, TsConfig};
use swc_ecma_visit::Visit;

use crate::renderer::Renderer;
use crate::{DocItem, DocItemKind};

pub struct Scraper {
    items: Vec<DocItem>,
}

impl Scraper {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn scrape(&mut self, pattern: &str) -> Result<(), Box<dyn std::error::Error>> {
        for entry in glob(pattern).expect("failed to read glob pattern") {
            match entry {
                Ok(path) => {
                    let cm: Lrc<SourceMap> = Default::default();
                    let handler =
                        Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

                    let fm = cm
                        .load_file(&path)
                        .expect(&format!("failed to load {}", path.display()));

                    let comments = SwcComments::default();

                    let lexer = Lexer::new(
                        Syntax::Typescript(TsConfig {
                            decorators: true,
                            ..Default::default()
                        }),
                        Default::default(),
                        StringInput::from(&*fm),
                        Some(&comments),
                    );

                    let mut parser = Parser::new_from(lexer);

                    for err in parser.take_errors() {
                        err.into_diagnostic(&handler).emit();
                    }

                    let module = parser
                        .parse_typescript_module()
                        .map_err(|err| err.into_diagnostic(&handler).emit())
                        .expect("failed to parse module");

                    let mut scraper = ModuleScraper::new(comments);

                    scraper.visit_module(&module);

                    self.items.extend(scraper.items);
                }
                Err(err) => println!("{:?}", err),
            }
        }

        Ok(())
    }

    pub fn render(self, renderer: &Renderer) -> Result<(), Box<dyn std::error::Error>> {
        renderer.render(self.items)
    }
}

struct ModuleScraper {
    comments: SwcComments,
    items: Vec<DocItem>,
}

impl ModuleScraper {
    fn new(comments: SwcComments) -> Self {
        Self {
            comments,
            items: Vec::new(),
        }
    }
}

impl Visit for ModuleScraper {
    fn visit_export_decl(&mut self, node: &ExportDecl) {
        let description = self
            .comments
            .get_leading(node.span.lo())
            .and_then(|comments| comments.first().cloned())
            .map(|comment| comment.text.to_string())
            .map(sanitize_doc_comment);

        match &node.decl {
            Decl::Class(class) => self.items.push(DocItem {
                name: class.ident.sym.to_string(),
                kind: DocItemKind::Class,
                description,
            }),
            Decl::TsTypeAlias(type_alias) => self.items.push(DocItem {
                name: type_alias.id.sym.to_string(),
                kind: DocItemKind::TypeAlias,
                description,
            }),
            Decl::TsInterface(interface) => self.items.push(DocItem {
                name: interface.id.sym.to_string(),
                kind: DocItemKind::Interface,
                description,
            }),
            Decl::Fn(fn_decl) => self.items.push(DocItem {
                name: fn_decl.ident.sym.to_string(),
                kind: DocItemKind::Function,
                description,
            }),
            Decl::Var(var_decl) => {
                for var in &var_decl.decls {
                    if let Some(ident) = var.name.as_ident() {
                        let is_function = var
                            .init
                            .as_ref()
                            .map(|init| init.is_arrow())
                            .unwrap_or(false);

                        self.items.push(DocItem {
                            name: ident.id.sym.to_string(),
                            kind: if is_function {
                                DocItemKind::Function
                            } else {
                                DocItemKind::Var
                            },
                            description: description.clone(),
                        })
                    }
                }
            }
            _ => {}
        }
    }
}

fn sanitize_doc_comment(comment: String) -> String {
    comment
        .lines()
        .map(|line| line.trim_start_matches(&[' ', '*']))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}
