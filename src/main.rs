use glob::glob;
use swc_common::errors::{ColorConfig, Handler};
use swc_common::sync::Lrc;
use swc_common::SourceMap;
use swc_ecma_ast::*;
use swc_ecma_parser::lexer::Lexer;
use swc_ecma_parser::{Parser, StringInput, Syntax};
use swc_ecma_visit::Visit;

struct TypeDoc {
    pub name: String,
}

struct DocVisitor {
    pub types: Vec<TypeDoc>,
}

impl DocVisitor {
    pub fn new() -> Self {
        Self { types: Vec::new() }
    }
}

impl Visit for DocVisitor {
    fn visit_ts_type_alias_decl(&mut self, node: &TsTypeAliasDecl) {
        self.types.push(TypeDoc {
            name: node.id.sym.to_string(),
        })
    }

    fn visit_ts_interface_decl(&mut self, node: &TsInterfaceDecl) {
        self.types.push(TypeDoc {
            name: node.id.sym.to_string(),
        })
    }

    fn visit_class_decl(&mut self, node: &ClassDecl) {
        self.types.push(TypeDoc {
            name: node.ident.sym.to_string(),
        })
    }

    fn visit_function(&mut self, node: &Function) {}
}

fn main() {
    let mut doc_visitor = DocVisitor::new();

    for entry in glob("/Users/maxdeviant/projects/thaumaturge/src/**/*.ts")
        .expect("failed to read glob pattern")
    {
        match entry {
            Ok(path) => {
                let cm: Lrc<SourceMap> = Default::default();
                let handler =
                    Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

                let fm = cm.load_file(&path).expect("failed to load types.ts");

                let lexer = Lexer::new(
                    Syntax::Typescript(Default::default()),
                    Default::default(),
                    StringInput::from(&*fm),
                    None,
                );

                let mut parser = Parser::new_from(lexer);

                for err in parser.take_errors() {
                    err.into_diagnostic(&handler).emit();
                }

                let module = parser
                    .parse_module()
                    .map_err(|mut err| err.into_diagnostic(&handler).emit())
                    .expect("failed to parser module");

                doc_visitor.visit_module(&module);
            }
            Err(err) => println!("{:?}", err),
        }
    }

    for ty in doc_visitor.types {
        println!("{}", ty.name);
    }
}
