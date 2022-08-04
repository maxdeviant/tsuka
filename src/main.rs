use std::path::Path;

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
        let node = dbg!(node);
        self.types.push(TypeDoc {
            name: node.id.sym.to_string(),
        })
    }

    fn visit_function(&mut self, node: &Function) {
        let node = dbg!(node);
    }
}

fn main() {
    let cm: Lrc<SourceMap> = Default::default();
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

    let fm = cm
        .load_file(Path::new(
            "/Users/maxdeviant/projects/thaumaturge/src/types.ts",
        ))
        .expect("failed to load types.ts");

    let lexer = Lexer::new(
        Syntax::Typescript(Default::default()),
        Default::default(),
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);

    for e in parser.take_errors() {
        e.into_diagnostic(&handler).emit();
    }

    let module = parser
        .parse_module()
        .map_err(|mut e| {
            // Unrecoverable fatal error occurred
            e.into_diagnostic(&handler).emit()
        })
        .expect("failed to parser module");

    let mut doc_visitor = DocVisitor::new();

    doc_visitor.visit_module(&module);

    for ty in doc_visitor.types {
        println!("{}", ty.name);
    }
}
