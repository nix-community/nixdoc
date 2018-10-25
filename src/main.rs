#[macro_use] extern crate structopt;

use rnix::parser::{ASTNode, Data};
use rnix::tokenizer::Meta;
use rnix::tokenizer::Trivia;
use rnix;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

/// Command line arguments for nixdoc
#[derive(Debug, StructOpt)]
#[structopt(name = "nixdoc", about = "Generate Docbook from Nix library functions")]
struct Options {
    /// Nix file to process.
    #[structopt(short = "f", long = "file", parse(from_os_str))]
    file: PathBuf,
}

#[derive(Debug)]
struct DocComment {
    /// Primary documentation string.
    doc: String,

    /// Optional type annotation for the thing being documented.
    doc_type: Option<String>,

    /// Usage example(s) (interpreted as a single code block)
    example: Option<String>,
}

#[derive(Debug)]
struct DocItem {
    name: String,
    comment: DocComment,
}

/// Represents a single function parameter and (potentially) its
/// documentation.
#[derive(Debug)]
struct Parameter {
    name: String,
    description: Option<String>,
    arg_type: Option<String>,
}

/// Represents a single manual section describing a library function.
#[derive(Debug)]
struct ManualEntry {
    /// Name of the section (used as the title)
    name: String,

    /// Type signature (if provided). This is not actually a checked
    /// type signature in any way.
    fn_type: Option<String>,

    /// Primary description of the entry.
    description: String, // TODO

    /// Parameters of the function
    parameters: Vec<Parameter>,
}

/// Retrieve documentation comments. For now only multiline comments
/// starting with `@doc` are considered.
fn retrieve_doc_comment(meta: &Meta) -> Option<String> {
    for item in meta.leading.iter() {
        if let Trivia::Comment { multiline, content, .. } = item {
            if *multiline { //  && content.as_str().starts_with(" @doc") {
                return Some(content.to_string())
            }
        }
    }

    return None;
}

/// Transforms an AST node into a `DocItem` if it has a leading
/// documentation comment.
fn retrieve_doc_item(node: &ASTNode) -> Option<DocItem> {
    // We are only interested in identifiers.
    if let Data::Ident(meta, name) = &node.data {
        let comment = retrieve_doc_comment(meta)?;

        return Some(DocItem {
            name: name.to_string(),
            comment: parse_doc_comment(&comment),
        })
    }

    return None;
}

/// *Really* dumb, mutable, hacky doc comment "parser".
fn parse_doc_comment(raw: &str) -> DocComment {
    enum ParseState { Doc, Type, Example }

    let mut doc = String::new();
    let mut doc_type = String::new();
    let mut example = String::new();
    let mut state = ParseState::Doc;

    for line in raw.trim().lines() {
        let mut line = line.trim();

        if line.starts_with("@doc ") {
            state = ParseState::Doc;
            line = line.trim_start_matches("@doc ");
        }

        if line.starts_with("Type:") {
            state = ParseState::Type;
            line = line.trim_start_matches("Type:");
        }

        if line.starts_with("Example:") {
            state = ParseState::Example;
            line = line.trim_start_matches("Example:");
        }

        match state {
            ParseState::Type => doc_type.push_str(line.trim()),
            ParseState::Doc => {
                doc.push_str(line.trim());
                doc.push('\n');
            },
            ParseState::Example => {
                example.push_str(line.trim());
                example.push('\n');
            },
        }
    }


    let f = |s: String| if s.is_empty() { None } else { Some(s.into()) };

    DocComment {
        doc: doc.trim().into(),
        doc_type: f(doc_type),
        example: f(example),
    }
}

fn main() {
    let opts = Options::from_args();
    let src = fs::read_to_string(opts.file).unwrap();
    let nix = rnix::parse(&src).unwrap();

    let doc_items: Vec<DocItem> = nix.arena.into_iter()
        .filter_map(retrieve_doc_item)
        .collect();

    for doc_item in doc_items {
        println!("Item: {}\nDoc: {:#?}", doc_item.name, doc_item.comment)
    }
}
