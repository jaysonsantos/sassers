#[macro_use]
extern crate log;

extern crate regex;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use regex::Regex;
use std::io::Write;

mod ast;
mod context;
mod error;
mod expression_evaluator;
mod sass;
mod operator;
mod operator_offset;
mod operator_or_token;
mod optimizer;
mod parser;
mod token;
mod token_offset;
mod tokenizer;

use context::Context;
use error::Result;
use tokenizer::Tokenizer;
use parser::Parser;
use sass::output_style::{SassOutputStyle, Nested, Compressed, Expanded,
                         Compact, Debug};

fn resolve_imports(inputpath: &PathBuf) -> Result<String> {
    let mut file = try!(File::open(&inputpath));
    let mut sass = String::new();

    try!(file.read_to_string(&mut sass));

    let mut imports_resolved = String::new();
    for line in sass.split("\n") {
        let re = Regex::new("@import \"([^\"]*)\";").unwrap();

        match re.captures(line) {
            Some(caps) => {
                let imported = try!(resolve_imports(&inputpath.with_file_name(caps.at(1).unwrap())));
                imports_resolved.push_str(&imported);
            },
            None => {
                imports_resolved.push_str(line);
            },
        }
        imports_resolved.push_str("\n");
    }
    Ok(imports_resolved)
}

pub fn compile(input_filename: &str, output: &mut Write, style: &str) -> Result<()> {
    let input_path = PathBuf::from(input_filename);
    let imports_resolved = try!(resolve_imports(&input_path));

    match style {
        "tokens" => {
            let mut tokenizer = Tokenizer::new(&imports_resolved);
            while let Some(token) = tokenizer.next() {
                try!(writeln!(output, "{:?}", token));
            }
        },
        "ast" => {
            let mut parser = Parser::new(&imports_resolved);
            while let Some(root) = parser.next() {
                try!(writeln!(output, "{:#?}", root));
            }
        },
        other => {
            let style: Box<SassOutputStyle> = get_style(other);
            let mut parser  = Parser::new(&imports_resolved);
            let mut context = Context::new();
            while let Some(Ok(ast_root)) = parser.next() {
                let evaluated = ast_root.evaluate(&mut context);
                if let Some(root) = evaluated {
                    let optimized = optimizer::optimize(root);
                    for r in optimized.into_iter() {
                        try!(r.stream(output, &*style));
                    }
                }
            }
        },
    }
    Ok(())
}

fn get_style(style: &str) -> Box<SassOutputStyle> {
    match style {
        "nested"     => Box::new(Nested {}),
        "compressed" => Box::new(Compressed {}),
        "expanded"   => Box::new(Expanded {}),
        "compact"    => Box::new(Compact {}),
        "debug"      => Box::new(Debug {}),
        style        => panic!("Unknown output style {:?}. Please specify one of nested, compressed, expanded, or compact.", style),
    }
}
