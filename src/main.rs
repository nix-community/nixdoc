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

//! This tool generates CommonMark from a Nix file defining library
//! functions, such as the files in `lib/` in the nixpkgs repository.
//!
//! TODO:
//! * extract function argument names
//! * extract line number & add it to generated output
//! * figure out how to specify examples (& leading whitespace?!)

mod comment;
mod commonmark;
mod format;
mod frontmatter;
mod legacy;
#[cfg(test)]
mod test;

use crate::{format::handle_indentation, legacy::retrieve_legacy_comment};

use self::comment::get_expr_docs;
use self::commonmark::*;
use format::shift_headings;
use frontmatter::get_imported_content;
use legacy::{collect_lambda_args_legacy, LegacyDocItem};
use rnix::{
    ast::{Attr, AttrpathValue, Expr, Inherit, LetIn},
    SyntaxKind, SyntaxNode,
};
use rowan::{ast::AstNode, WalkEvent};
use std::{fs, path::Path, process::exit};

use std::collections::HashMap;
use std::io;
use std::io::Write;

use clap::Parser;
use std::path::PathBuf;

/// Command line arguments for nixdoc
#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Options {
    /// Prefix for the category (e.g. 'lib' or 'utils').
    #[arg(short, long, default_value_t = String::from("lib"))]
    prefix: String,

    /// Name of the function category (e.g. 'strings', 'attrsets').
    #[arg(short, long)]
    category: String,

    /// Description of the function category.
    #[arg(short, long)]
    description: String,

    /// Nix file to process.
    #[arg(short, long)]
    file: PathBuf,

    /// Path to a file containing location data as JSON.
    #[arg(short, long)]
    locs: Option<PathBuf>,
}

#[derive(Debug)]
struct DocComment {
    /// Primary documentation string.
    doc: String,

    /// Optional type annotation for the thing being documented.
    /// This is only available as legacy feature
    doc_type: Option<String>,

    /// Usage example(s) (interpreted as a single code block)
    /// This is only available as legacy feature
    example: Option<String>,
}

#[derive(Debug)]
struct DocItem {
    name: String,
    comment: DocComment,
}

enum DocItemOrLegacy {
    LegacyDocItem(LegacyDocItem),
    DocItem(DocItem),
}

/// Returns a rfc145 doc-comment if one is present
pub fn retrieve_doc_comment(
    node: &SyntaxNode,
    shift_headings_by: Option<usize>,
    file: &Path,
) -> Option<String> {
    let doc_comment = get_expr_docs(node);

    doc_comment.map(|inner| {
        // Must handle indentation before processing yaml frontmatter
        let content = handle_indentation(&inner).unwrap_or_default();

        let final_content = match get_imported_content(file, &content) {
            // Use the imported content instead of the original content
            Ok(Some(imported_content)) => imported_content,

            // Use the original content
            Ok(None) => content,

            // Abort if we failed to read the frontmatter
            Err(e) => {
                eprintln!("{}", e);
                exit(1);
            }
        };

        shift_headings(
            &handle_indentation(&final_content).unwrap_or(String::new()),
            // H1 to H4 can be used in the doc-comment with the current rendering.
            // They will be shifted to H3, H6
            // H1 and H2 are currently used by the outer rendering. (category and function name)
            shift_headings_by.unwrap_or(2),
        )
    })
}

/// Transforms an AST node into a `DocItem` if it has a leading
/// documentation comment.
fn retrieve_doc_item(node: &AttrpathValue, file_path: &Path) -> Option<DocItemOrLegacy> {
    let ident = node.attrpath().unwrap();
    // TODO this should join attrs() with '.' to handle whitespace, dynamic attrs and string
    // attrs. none of these happen in nixpkgs lib, and the latter two should probably be
    // rejected entirely.
    let item_name = ident.to_string();

    let doc_comment = retrieve_doc_comment(node.syntax(), Some(2), file_path);
    match doc_comment {
        Some(comment) => Some(DocItemOrLegacy::DocItem(DocItem {
            name: item_name,
            comment: DocComment {
                doc: comment,
                doc_type: None,
                example: None,
            },
        })),
        // Fallback to legacy comment is there is no doc_comment
        None => {
            let comment = retrieve_legacy_comment(node.syntax(), false)?;
            Some(DocItemOrLegacy::LegacyDocItem(LegacyDocItem {
                name: item_name,
                comment: parse_doc_comment(&comment),
                args: vec![],
            }))
        }
    }
}

/// Dumb, mutable, hacky doc comment "parser".
fn parse_doc_comment(raw: &str) -> DocComment {
    enum ParseState {
        Doc,
        Type,
        Example,
    }

    let mut state = ParseState::Doc;

    // Split the string into three parts, docs, type and example
    let mut doc_str = String::new();
    let mut type_str = String::new();
    let mut example_str = String::new();

    for line in raw.split_inclusive('\n') {
        let trimmed_line = line.trim();
        if let Some(suffix) = trimmed_line.strip_prefix("Type:") {
            state = ParseState::Type;
            type_str.push_str(suffix);
            type_str.push('\n');
        } else if let Some(suffix) = trimmed_line.strip_prefix("Example:") {
            state = ParseState::Example;
            example_str.push_str(suffix);
            example_str.push('\n');
        } else {
            match state {
                ParseState::Doc => doc_str.push_str(line),
                ParseState::Type => type_str.push_str(line),
                ParseState::Example => example_str.push_str(line),
            }
        }
    }

    DocComment {
        doc: handle_indentation(&doc_str).unwrap_or(String::new()),
        doc_type: handle_indentation(&type_str),
        example: handle_indentation(&example_str),
    }
}

/// Traverse the arena from a top-level SetEntry and collect, where
/// possible:
///
/// 1. The identifier of the set entry itself.
/// 2. The attached doc comment on the entry.
/// 3. The argument names of any curried functions (pattern functions
///    not yet supported).
fn collect_entry_information(entry: AttrpathValue, file_path: &Path) -> Option<LegacyDocItem> {
    let doc_item = retrieve_doc_item(&entry, file_path)?;

    match doc_item {
        DocItemOrLegacy::LegacyDocItem(v) => {
            if let Some(Expr::Lambda(l)) = entry.value() {
                Some(LegacyDocItem {
                    args: collect_lambda_args_legacy(l, file_path),
                    ..v
                })
            } else {
                Some(v)
            }
        }
        // Convert DocItems into legacyItem for markdown rendering
        DocItemOrLegacy::DocItem(v) => Some(LegacyDocItem {
            args: vec![],
            name: v.name,
            comment: v.comment,
        }),
    }
}

// a binding is an assignment, which can take place in an attrset
// - as attributes
// - as inherits
fn collect_bindings(
    node: &SyntaxNode,
    prefix: &str,
    category: &str,
    scope: HashMap<String, ManualEntry>,
    file_path: &Path,
) -> Vec<ManualEntry> {
    for ev in node.preorder() {
        match ev {
            WalkEvent::Enter(n) if n.kind() == SyntaxKind::NODE_ATTR_SET => {
                let mut entries = vec![];
                for child in n.children() {
                    if let Some(apv) = AttrpathValue::cast(child.clone()) {
                        entries.extend(
                            collect_entry_information(apv, file_path)
                                .map(|di| di.into_entry(prefix, category)),
                        );
                    } else if let Some(inh) = Inherit::cast(child) {
                        // `inherit (x) ...` needs much more handling than we can
                        // reasonably do here
                        if inh.from().is_some() {
                            continue;
                        }
                        entries.extend(inh.attrs().filter_map(|a| match a {
                            Attr::Ident(i) => scope.get(&i.syntax().text().to_string()).cloned(),
                            // ignore non-ident keys. these aren't useful as lib
                            // functions in general anyway.
                            _ => None,
                        }));
                    }
                }
                return entries;
            }
            _ => (),
        }
    }

    vec![]
}

// Main entrypoint for collection
// TODO: document
fn collect_entries(
    root: rnix::Root,
    prefix: &str,
    category: &str,
    file_path: &Path,
) -> Vec<ManualEntry> {
    // we will look into the top-level let and its body for function docs.
    // we only need a single level of scope for this.
    // since only the body can export a function we don't need to implement
    // mutually recursive resolution.
    for ev in root.syntax().preorder() {
        match ev {
            WalkEvent::Enter(n) if n.kind() == SyntaxKind::NODE_LET_IN => {
                return collect_bindings(
                    LetIn::cast(n.clone()).unwrap().body().unwrap().syntax(),
                    prefix,
                    category,
                    n.children()
                        .filter_map(AttrpathValue::cast)
                        .filter_map(|v| collect_entry_information(v, file_path))
                        .map(|di| (di.name.to_string(), di.into_entry(prefix, category)))
                        .collect(),
                    file_path,
                );
            }
            WalkEvent::Enter(n) if n.kind() == SyntaxKind::NODE_ATTR_SET => {
                return collect_bindings(&n, prefix, category, Default::default(), file_path);
            }
            _ => (),
        }
    }

    vec![]
}

fn retrieve_description(
    nix: &rnix::Root,
    description: &str,
    category: &str,
    file: &Path,
) -> String {
    format!(
        "# {} {{#sec-functions-library-{}}}\n{}\n",
        description,
        category,
        &nix.syntax()
            .first_child()
            .and_then(|node| retrieve_doc_comment(&node, Some(1), file)
                .or(retrieve_legacy_comment(&node, false)))
            .and_then(|doc_item| handle_indentation(&doc_item))
            .unwrap_or_default()
    )
}

fn main() {
    let mut output = io::stdout();
    let opts = Options::parse();
    let src = fs::read_to_string(&opts.file).unwrap();
    let locs = match opts.locs {
        None => Default::default(),
        Some(p) => fs::read_to_string(p)
            .map_err(|e| e.to_string())
            .and_then(|json| serde_json::from_str(&json).map_err(|e| e.to_string()))
            .expect("could not read location information"),
    };
    let nix = rnix::Root::parse(&src).ok().expect("failed to parse input");
    let description = retrieve_description(&nix, &opts.description, &opts.category, &opts.file);

    // TODO: move this to commonmark.rs
    writeln!(output, "{}", description).expect("Failed to write header");

    for entry in collect_entries(nix, &opts.prefix, &opts.category, &opts.file) {
        entry
            .write_section(&locs, &mut output)
            .expect("Failed to write section")
    }
}
