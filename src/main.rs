use std::fmt::Write;
use std::fs::File;
use std::path::PathBuf;

use glob::glob;
use pulldown_cmark as markdown;
use swc::SwcComments;
use swc_common::comments::Comments;
use swc_common::errors::{ColorConfig, Handler};
use swc_common::sync::Lrc;
use swc_common::SourceMap;
use swc_ecma_ast::*;
use swc_ecma_parser::lexer::Lexer;
use swc_ecma_parser::{Parser, StringInput, Syntax};
use swc_ecma_visit::Visit;

#[derive(Debug)]
enum DocItemKind {
    Class,
    TypeAlias,
    Interface,
}

#[derive(Debug)]
struct DocItem {
    pub name: String,
    pub kind: DocItemKind,
    pub description: Option<String>,
}

struct DocVisitor {
    comments: SwcComments,
    pub types: Vec<DocItem>,
}

impl DocVisitor {
    pub fn new(comments: SwcComments) -> Self {
        Self {
            comments,
            types: Vec::new(),
        }
    }
}

impl Visit for DocVisitor {
    fn visit_export_decl(&mut self, node: &ExportDecl) {
        match &node.decl {
            Decl::Class(class) => self.types.push(DocItem {
                name: class.ident.sym.to_string(),
                kind: DocItemKind::Class,
                description: self
                    .comments
                    .get_leading(node.span.lo())
                    .and_then(|comments| comments.first().cloned())
                    .map(|comment| comment.text.to_string())
                    .map(sanitize_doc_comment),
            }),
            Decl::TsTypeAlias(type_alias) => self.types.push(DocItem {
                name: type_alias.id.sym.to_string(),
                kind: DocItemKind::TypeAlias,
                description: self
                    .comments
                    .get_leading(node.span.lo())
                    .and_then(|comments| comments.first().cloned())
                    .map(|comment| comment.text.to_string())
                    .map(sanitize_doc_comment),
            }),
            Decl::TsInterface(interface) => self.types.push(DocItem {
                name: interface.id.sym.to_string(),
                kind: DocItemKind::Interface,
                description: self
                    .comments
                    .get_leading(node.span.lo())
                    .and_then(|comments| comments.first().cloned())
                    .map(|comment| comment.text.to_string())
                    .map(sanitize_doc_comment),
            }),
            _ => {}
        }
    }

    fn visit_function(&mut self, node: &Function) {}
}

fn sanitize_doc_comment(comment: String) -> String {
    comment
        .lines()
        .map(|line| line.trim_start_matches(&[' ', '*']))
        .collect::<Vec<_>>()
        .join("\n")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = PathBuf::from("output");

    if !output_dir.exists() {
        std::fs::create_dir_all(&output_dir)?;
    }

    let mut options = markdown::Options::empty();
    options.insert(markdown::Options::ENABLE_STRIKETHROUGH);
    options.insert(markdown::Options::ENABLE_TABLES);

    let mut output = String::new();
    writeln!(&mut output, "<!doctype html>")?;
    writeln!(&mut output, r#"<html lang="en">"#)?;
    writeln!(&mut output, "<head>")?;
    writeln!(&mut output, r#"<meta charset="utf-8">"#)?;
    writeln!(&mut output, "</head>")?;
    writeln!(&mut output, "<body>")?;

    for entry in glob("/Users/maxdeviant/projects/thaumaturge/src/**/*.ts")
        .expect("failed to read glob pattern")
    {
        match entry {
            Ok(path) => {
                let cm: Lrc<SourceMap> = Default::default();
                let handler =
                    Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

                let fm = cm.load_file(&path).expect("failed to load types.ts");

                let comments = SwcComments::default();

                let lexer = Lexer::new(
                    Syntax::Typescript(Default::default()),
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
                    .map_err(|mut err| err.into_diagnostic(&handler).emit())
                    .expect("failed to parser module");

                let mut doc_visitor = DocVisitor::new(comments);

                doc_visitor.visit_module(&module);

                for ty in doc_visitor.types {
                    writeln!(&mut output, "<div>")?;
                    writeln!(&mut output, "<h1>{}</h1>", ty.name)?;

                    let description = ty.description.unwrap_or(String::new());

                    let parser = markdown::Parser::new_ext(&description, options);

                    let mut description_html = String::new();
                    markdown::html::push_html(&mut description_html, parser);

                    writeln!(&mut output, "{}", description_html)?;
                    writeln!(&mut output, "</div>")?;

                    let mut item_output = String::new();
                    writeln!(&mut item_output, "<!doctype html>")?;
                    writeln!(&mut item_output, r#"<html lang="en">"#)?;
                    writeln!(&mut item_output, "<head>")?;
                    writeln!(&mut item_output, r#"<meta charset="utf-8">"#)?;
                    writeln!(&mut item_output, "</head>")?;
                    writeln!(&mut item_output, "<body>")?;
                    writeln!(&mut item_output, "<h1>{}</h1>", ty.name)?;
                    writeln!(&mut item_output, "{}", description_html)?;
                    writeln!(&mut item_output, "</body>")?;
                    writeln!(&mut item_output, "</html>")?;

                    let tag = match ty.kind {
                        DocItemKind::Class => "class",
                        DocItemKind::TypeAlias => "type",
                        DocItemKind::Interface => "interface",
                    };

                    let output_filepath = output_dir.join(format!("{}.{}.html", tag, ty.name));
                    let mut output_file = File::create(&output_filepath)?;
                    output_file.write_all(item_output.as_bytes())?;
                }
            }
            Err(err) => println!("{:?}", err),
        }
    }

    writeln!(&mut output, "</body>")?;
    writeln!(&mut output, "</html>")?;

    use std::io::Write;

    let output_filepath = output_dir.join("index.html");
    let mut output_file = File::create(&output_filepath)?;
    output_file.write_all(output.as_bytes())?;

    Ok(())
}
