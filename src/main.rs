// Copyright (C) 2018 Vincent Ambo <mail@tazj.in>
//
// nixdoc is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

//! This tool generates DocBook XML from a Nix file defining library
//! functions, such as the files in `lib/` in the nixpkgs repository.
//!
//! TODO:
//! * extract function argument names
//! * extract line number & add it to generated output
//! * figure out how to specify examples (& leading whitespace?!)

#[macro_use] extern crate structopt;
extern crate xml;
extern crate failure;
extern crate rnix;
extern crate rowan;

mod docbook;

use self::docbook::*;
use rnix::{
    parser::{Node, NodeType, Types},
    tokenizer::Token,
    types::{Ident, Lambda, Pattern, SetEntry, TypedNode}
};
use rowan::{SmolStr, RefRoot, WalkEvent};
use std::{
    fs,
    io,
    path::PathBuf
};
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
fn retrieve_doc_comment(allow_single_line: bool, mut node: Node<RefRoot<Types>>) -> Option<String> {
    loop {
        // Get the previous node, exploring parents if needed
        loop {
            let new = node.prev_sibling();
            if let Some(new) = new {
                node = new;
                break;
            } else {
                node = node.parent()?;
            }
        }

        // Check if it's a comment
        match node.kind() {
            NodeType::Token(Token::Comment) => {
                // Get the content and trim leading # or /*
                let mut content = node.leaf_text().map(SmolStr::as_str).unwrap_or_default();
                if content.starts_with('#') && allow_single_line {
                    content = &content[1..];
                } else if content.starts_with("/*") {
                    assert!(content[2..].ends_with("*/"));
                    let mut len = content.len();
                    content = &content[2..len-2];
                } else {
                    break None;
                }

                break Some(content.to_string());
            }
            NodeType::Token(Token::Whitespace) => (),
            _ => break None
        }
    }
}

/// Transforms an AST node into a `DocItem` if it has a leading
/// documentation comment.
fn retrieve_doc_item(node: &Ident<RefRoot<Types>>) -> Option<DocItem> {
    // We are only interested in identifiers.
    let comment = retrieve_doc_comment(false, *node.node())?;

    return Some(DocItem {
        name: node.as_str().to_string(),
        comment: parse_doc_comment(&comment),
        args: vec![],
    })
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

        if line.starts_with("Type:") {
            state = ParseState::Type;
            line = &line[5..]; // trim 'Type:'
        }

        if line.starts_with("Example:") {
            state = ParseState::Example;
            line = &line[8..]; // trim 'Example:'
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
fn collect_lambda_args<'a>(lambda_node: &Lambda<RefRoot<Types>>, args: &mut Vec<Argument>) -> Option<()> {
    let ident_node = lambda_node.arg();

    // "Flat" function arguments are represented as identifiers, ..
    if let Some(ident) = Ident::cast(ident_node) {
        args.push(Argument::Flat(SingleArg {
            name: ident.as_str().to_string(),
            doc: retrieve_doc_comment(true, *ident.node()),
        }));
    }

    // ... pattern style arguments are represented as, well, patterns.
    if let Some(pattern) = Pattern::cast(ident_node) {
        // The first child of a pattern is a token representing the
        // opening curly brace, followed by a sibling chain of
        // `PatEntry` nodes which each have the identifier as their
        // first child.
        let pattern_vec: Vec<_> = pattern.entries()
            .map(|entry| entry.name())
            .map(|name| SingleArg {
                name: name.as_str().to_string(),
                doc: retrieve_doc_comment(true, *name.node())
            })
            .collect();

        if !pattern_vec.is_empty() {
            args.push(Argument::Pattern(pattern_vec));
        }
    }

    // Curried or not?
    if let Some(lambda) = Lambda::cast(lambda_node.body()) {
        collect_lambda_args(&lambda, args);
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
fn collect_entry_information(node: &SetEntry<RefRoot<Types>>) -> Option<DocItem> {
    // The "root" of any attribute set entry is this `SetEntry` node.
    // It has an `Attribute` child, which in turn has the identifier
    // (on which the documentation comment is stored) as its child.
    let attr_node = node.key();
    let ident_node = attr_node.path().next().and_then(Ident::cast)?;

    // At this point we can retrieve the `DocItem` from the identifier
    // node - this already contains most of the information we are
    // interested in.
    let doc_item = retrieve_doc_item(&ident_node)?;

    // From our entry we check whether we are dealing with a lambda. If so, we
    // can start collecting the function arguments - otherwise we're done.
    let content_node = node.value();

    if let Some(lambda) = Lambda::cast(content_node) {
        let mut args: Vec<Argument> = vec![];
        collect_lambda_args(&lambda, &mut args);
        Some(DocItem { args, ..doc_item })
    } else {
        Some(doc_item)
    }
}

fn main() {
    let opts = Options::from_args();
    let src = fs::read_to_string(&opts.file).unwrap();
    let nix = rnix::parse(&src).as_result().unwrap();

    let entries: Vec<ManualEntry> = nix.node().borrowed().preorder()
        .filter_map(|event| match event {
            WalkEvent::Enter(node) => SetEntry::cast(node),
            WalkEvent::Leave(_) => None
        })
        .filter_map(|node| collect_entry_information(&node))
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
