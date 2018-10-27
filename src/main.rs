//! This tool generates DocBook XML from a Nix file defining library
//! functions, such as the files in `lib/` in the nixpkgs repository.
//!
//! TODO:
//! * extract function argument names
//! * extract line number & add it to generated output
//! * figure out how to specify examples (& leading whitespace?!)

mod docbook;

use self::docbook::*;
use rnix::parser::{Arena, ASTNode, ASTKind, Data};
use rnix::tokenizer::Meta;
use rnix::tokenizer::Trivia;
use rnix;
use std::fs;
use std::io;
use std::path::PathBuf;
use structopt::StructOpt;
use xml::writer::{EmitterConfig, XmlEvent};

/// Command line arguments for nixdoc
#[derive(Debug, StructOpt)]
#[structopt(name = "nixdoc", about = "Generate Docbook from Nix library functions")]
struct Options {
    /// Nix file to process.
    #[structopt(short = "f", long = "file", parse(from_os_str))]
    file: PathBuf,

    /// Name of the function category (e.g. 'strings', 'attrsets').
    #[structopt(short = "c", long = "category")]
    category: String,

    /// Description of the function category.
    #[structopt(short = "d", long = "description")]
    description: String,
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
    args: Vec<Argument>,
}

/// Retrieve documentation comments. For now only multiline comments
/// starting with `@doc` are considered.
fn retrieve_doc_comment(allow_single_line: bool, meta: &Meta) -> Option<String> {
    for item in meta.leading.iter() {
        if let Trivia::Comment { multiline, content, .. } = item {
            if *multiline || allow_single_line {
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
        let comment = retrieve_doc_comment(false, meta)?;

        return Some(DocItem {
            name: name.to_string(),
            comment: parse_doc_comment(&comment),
            args: vec![],
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
            line = &line[5..]; //.trim_start_matches("Type:");
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

/// Traverse a pattern argument, collecting its argument names.
fn collect_pattern_args<'a>(arena: &Arena<'a>,
                            entry: &ASTNode,
                            args: &mut Vec<SingleArg>) -> Option<()> {
    if let Data::Ident(meta, name) = &arena[entry.node.child?].data {
        args.push(SingleArg {
            name: name.to_string(),
            doc: retrieve_doc_comment(true, meta),
        });
    }

    // Recurse, but only if the entry's sibling is also an entry.
    let next_entry = &arena[entry.node.sibling?];
    if next_entry.kind == ASTKind::PatEntry {
        collect_pattern_args(arena, next_entry, args);
    }

    Some(())
}

/// Traverse a Nix lambda and collect the identifiers of arguments
/// until an unexpected AST node is encountered.
///
/// This will collect the argument names for curried functions in the
/// `a: b: c: ...`-style, but does not currently work with pattern
/// functions (`{ a, b, c }: ...`).
///
/// In the AST representation used by rnix, any lambda node has an
/// immediate child that is the identifier of its argument. The "body"
/// of the lambda is two steps to the right from that identifier, if
/// it is a lambda the function is curried and we can recurse.
fn collect_lambda_args<'a>(arena: &Arena<'a>,
                           lambda_node: &ASTNode,
                           args: &mut Vec<Argument>) -> Option<()> {
    let ident_node = &arena[lambda_node.node.child?];

    // "Flat" function arguments are represented as identifiers, ..
    if let Data::Ident(meta, name) = &ident_node.data {
        args.push(Argument::Flat(SingleArg {
            name: name.to_string(),
            doc: retrieve_doc_comment(true, meta),
        }));
    }

    // ... pattern style arguments are represented as, well, patterns.
    if ident_node.kind == ASTKind::Pattern {
        let mut pattern_vec = vec![];

        // The first child of a pattern is a token representing the
        // opening curly brace, followed by a sibling chain of
        // `PatEntry` nodes which each have the identifier as their
        // first child.
        let token_node = &arena[ident_node.node.child?];
        let first_entry = &arena[token_node.node.sibling?];
        collect_pattern_args(arena, first_entry, &mut pattern_vec);

        if !pattern_vec.is_empty() {
            args.push(Argument::Pattern(pattern_vec));
        }
    }

    // Two to the right ...
    let token_node = &arena[ident_node.node.sibling?];
    let body_node = &arena[token_node.node.sibling?];

    // Curried or not?
    if body_node.kind == ASTKind::Lambda {
        collect_lambda_args(arena, body_node, args);
    }

    Some(())
}

/// Traverse the arena from a top-level SetEntry and collect, where
/// possible:
///
/// 1. The identifier of the set entry itself.
/// 2. The attached doc comment on the entry.
/// 3. The argument names of any curried functions (pattern functions
///    not yet supported).
fn collect_entry_information<'a>(arena: &Arena<'a>, entry_node: &ASTNode) -> Option<DocItem> {
    // The "root" of any attribute set entry is this `SetEntry` node.
    // It has an `Attribute` child, which in turn has the identifier
    // (on which the documentation comment is stored) as its child.
    let attr_node = &arena[entry_node.node.child?];
    let ident_node = &arena[attr_node.node.child?];

    // At this point we can retrieve the `DocItem` from the identifier
    // node - this already contains most of the information we are
    // interested in.
    let doc_item = retrieve_doc_item(ident_node)?;

    // From our entry we can walk two nodes to the right and check
    // whether we are dealing with a lambda. If so, we can start
    // collecting the function arguments - otherwise we're done.
    let assign_node = &arena[attr_node.node.sibling?];
    let content_node = &arena[assign_node.node.sibling?];

    if content_node.kind == ASTKind::Lambda {
        let mut args: Vec<Argument> = vec![];
        collect_lambda_args(arena, content_node, &mut args);
        Some(DocItem { args, ..doc_item })
    } else {
        Some(doc_item)
    }
}

fn main() {
    let opts = Options::from_args();
    let src = fs::read_to_string(&opts.file).unwrap();
    let nix = rnix::parse(&src).unwrap();

    let entries: Vec<ManualEntry> = nix.arena.into_iter()
        .filter(|node| node.kind == ASTKind::SetEntry)
        .filter_map(|node| collect_entry_information(&nix.arena, node))
        .map(|d| ManualEntry {
            category: opts.category.clone(),
            name: d.name,
            description: d.comment.doc
                .split("\n\n")
                .map(|s| s.to_string())
                .collect(),
            fn_type: d.comment.doc_type,
            example: d.comment.example,
            args: d.args,
        })
        .collect();

    let mut writer = EmitterConfig::new()
        .perform_indent(true)
        .create_writer(io::stdout());

    writer.write(
        XmlEvent::start_element("section")
            .attr("xmlns", "http://docbook.org/ns/docbook")
            .attr("xmlns:xlink", "http://www.w3.org/1999/xlink")
            .attr("xmlns:xi", "http://www.w3.org/2001/XInclude")
            .attr("xml:id", format!("sec-functions-library-{}", opts.category).as_str()))
        .unwrap();

    writer.write(XmlEvent::comment(r#"Do not edit this file manually!

This file was generated using nixdoc[1]. Please edit the source Nix
file from which this XML was generated instead.

If you need to manually override the documentation of a single
function in this file, create a new override file at
`nixpkgs/docs/functions/library/overrides/<function-identifier>.xml`.

[1]: https://github.com/tazjin/nixdoc
"#)).unwrap();

    writer.write(XmlEvent::start_element("title")).unwrap();
    writer.write(XmlEvent::characters(&opts.description)).unwrap();
    writer.write(XmlEvent::end_element()).unwrap();

    for entry in entries {
        entry.write_section_xml(&mut writer).expect("Failed to write section")
    }

    writer.write(XmlEvent::end_element()).unwrap();
}
