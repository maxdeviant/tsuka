use glob::glob;
use swc::SwcComments;
use swc_common::comments::Comments;
use swc_common::errors::{ColorConfig, Handler};
use swc_common::sync::Lrc;
use swc_common::SourceMap;
use swc_ecma_ast::*;
use swc_ecma_parser::lexer::Lexer;
use swc_ecma_parser::{Parser, StringInput, Syntax};
use swc_ecma_visit::Visit;

struct TypeDoc {
    pub name: String,
    pub description: Option<String>,
}

struct DocVisitor {
    comments: SwcComments,
    pub types: Vec<TypeDoc>,
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
    fn visit_ts_type_alias_decl(&mut self, node: &TsTypeAliasDecl) {
        self.types.push(TypeDoc {
            name: node.id.sym.to_string(),
            description: None,
        })
    }

    fn visit_ts_interface_decl(&mut self, node: &TsInterfaceDecl) {
        self.types.push(TypeDoc {
            name: node.id.sym.to_string(),
            description: None,
        })
    }

    fn visit_class_decl(&mut self, node: &ClassDecl) {
        self.types.push(TypeDoc {
            name: node.ident.sym.to_string(),
            description: None,
        })
    }

    fn visit_export_decl(&mut self, node: &ExportDecl) {
        match &node.decl {
            Decl::Class(class) => self.types.push(TypeDoc {
                name: class.ident.sym.to_string(),
                description: self
                    .comments
                    .get_leading(node.span.lo())
                    .and_then(|comments| comments.first().cloned())
                    .map(|comment| comment.text.to_string()),
            }),
            _ => {}
        }
    }

    fn visit_function(&mut self, node: &Function) {}
}

fn main() {
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
                    println!("{} - {}", ty.name, ty.description.unwrap_or(String::new()));
                }
            }
            Err(err) => println!("{:?}", err),
        }
    }
}
