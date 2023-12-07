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

mod commonmark;

use self::commonmark::*;
use rnix::{
    ast::{AstToken, Attr, AttrpathValue, Comment, Expr, Inherit, Lambda, LetIn, Param},
    SyntaxKind, SyntaxNode,
};
use rowan::{ast::AstNode, WalkEvent};
use std::fs;
use textwrap::dedent;

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

impl DocItem {
    fn into_entry(self, prefix: &str, category: &str) -> ManualEntry {
        ManualEntry {
            prefix: prefix.to_string(),
            category: category.to_string(),
            name: self.name,
            description: self
                .comment
                .doc
                .split("\n\n")
                .map(|s| s.to_string())
                .collect(),
            fn_type: self.comment.doc_type,
            example: self.comment.example,
            args: self.args,
        }
    }
}

/// Retrieve documentation comments.
fn retrieve_doc_comment(node: &SyntaxNode, allow_line_comments: bool) -> Option<String> {
    // if the current node has a doc comment it'll be immediately preceded by that comment,
    // or there will be a whitespace token and *then* the comment tokens before it. We merge
    // multiple line comments into one large comment if they are on adjacent lines for
    // documentation simplicity.
    let mut token = node.first_token()?.prev_token()?;
    if token.kind() == SyntaxKind::TOKEN_WHITESPACE {
        token = token.prev_token()?;
    }
    if token.kind() != SyntaxKind::TOKEN_COMMENT {
        return None;
    }

    // if we want to ignore line comments (eg because they may contain deprecation
    // comments on attributes) we'll backtrack to the first preceding multiline comment.
    while !allow_line_comments && token.text().starts_with('#') {
        token = token.prev_token()?;
        if token.kind() == SyntaxKind::TOKEN_WHITESPACE {
            token = token.prev_token()?;
        }
        if token.kind() != SyntaxKind::TOKEN_COMMENT {
            return None;
        }
    }

    if token.text().starts_with("/*") {
        return Some(Comment::cast(token)?.text().to_string());
    }

    // backtrack to the start of the doc comment, allowing only adjacent line comments.
    // we don't care much about optimization here, doc comments aren't long enough for that.
    if token.text().starts_with('#') {
        let mut result = String::new();
        while let Some(comment) = Comment::cast(token) {
            if !comment.syntax().text().starts_with('#') {
                break;
            }
            result.insert_str(0, comment.text().trim());
            let ws = match comment.syntax().prev_token() {
                Some(t) if t.kind() == SyntaxKind::TOKEN_WHITESPACE => t,
                _ => break,
            };
            // only adjacent lines continue a doc comment, empty lines do not.
            match ws.text().strip_prefix('\n') {
                Some(trail) if !trail.contains('\n') => result.insert(0, ' '),
                _ => break,
            }
            token = match ws.prev_token() {
                Some(c) => c,
                _ => break,
            };
        }
        return Some(result);
    }

    None
}

/// Transforms an AST node into a `DocItem` if it has a leading
/// documentation comment.
fn retrieve_doc_item(node: &AttrpathValue) -> Option<DocItem> {
    let comment = retrieve_doc_comment(node.syntax(), false)?;
    let ident = node.attrpath().unwrap();
    // TODO this should join attrs() with '.' to handle whitespace, dynamic attrs and string
    // attrs. none of these happen in nixpkgs lib, and the latter two should probably be
    // rejected entirely.
    let item_name = ident.to_string();

    Some(DocItem {
        name: item_name,
        comment: parse_doc_comment(&comment),
        args: vec![],
    })
}

/// Ensure all lines in a multi-line doc-comments have the same indentation.
///
/// Consider such a doc comment:
///
/// ```nix
/// {
///   /* foo is
///   the value:
///     10
///   */
///   foo = 10;
/// }
/// ```
///
/// The parser turns this into:
///
/// ```
/// foo is
///   the value:
///     10
/// ```
///
///
/// where the first line has no leading indentation, and all other lines have preserved their
/// original indentation.
///
/// What we want instead is:
///
/// ```
/// foo is
/// the value:
///   10
/// ```
///
/// i.e. we want the whole thing to be dedented. To achieve this, we remove all leading whitespace
/// from the first line, and remove all common whitespace from the rest of the string.
fn handle_indentation(raw: &str) -> Option<String> {
    let result: String = match raw.split_once('\n') {
        Some((first, rest)) => {
            format!("{}\n{}", first.trim_start(), dedent(rest))
        }
        None => raw.into(),
    };

    Some(result.trim().to_owned()).filter(|s| !s.is_empty())
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

/// Traverse a Nix lambda and collect the identifiers of arguments
/// until an unexpected AST node is encountered.
fn collect_lambda_args(mut lambda: Lambda) -> Vec<Argument> {
    let mut args = vec![];

    loop {
        match lambda.param().unwrap() {
            // a variable, e.g. `id = x: x`
            Param::IdentParam(id) => {
                args.push(Argument::Flat(SingleArg {
                    name: id.to_string(),
                    doc: handle_indentation(
                        &retrieve_doc_comment(id.syntax(), true).unwrap_or_default(),
                    ),
                }));
            }
            // an attribute set, e.g. `foo = { a }: a`
            Param::Pattern(pat) => {
                // collect doc-comments for each attribute in the set
                let pattern_vec: Vec<_> = pat
                    .pat_entries()
                    .map(|entry| SingleArg {
                        name: entry.ident().unwrap().to_string(),
                        doc: handle_indentation(
                            &retrieve_doc_comment(entry.syntax(), true).unwrap_or_default(),
                        ),
                    })
                    .collect();

                args.push(Argument::Pattern(pattern_vec));
            }
        }

        // Curried or not?
        match lambda.body() {
            Some(Expr::Lambda(inner)) => lambda = inner,
            _ => break,
        }
    }

    args
}

/// Traverse the arena from a top-level SetEntry and collect, where
/// possible:
///
/// 1. The identifier of the set entry itself.
/// 2. The attached doc comment on the entry.
/// 3. The argument names of any curried functions (pattern functions
///    not yet supported).
fn collect_entry_information(entry: AttrpathValue) -> Option<DocItem> {
    let doc_item = retrieve_doc_item(&entry)?;

    if let Some(Expr::Lambda(l)) = entry.value() {
        Some(DocItem {
            args: collect_lambda_args(l),
            ..doc_item
        })
    } else {
        Some(doc_item)
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
) -> Vec<ManualEntry> {
    for ev in node.preorder() {
        match ev {
            WalkEvent::Enter(n) if n.kind() == SyntaxKind::NODE_ATTR_SET => {
                let mut entries = vec![];
                for child in n.children() {
                    if let Some(apv) = AttrpathValue::cast(child.clone()) {
                        entries.extend(
                            collect_entry_information(apv)
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
fn collect_entries(root: rnix::Root, prefix: &str, category: &str) -> Vec<ManualEntry> {
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
                        .filter_map(collect_entry_information)
                        .map(|di| (di.name.to_string(), di.into_entry(prefix, category)))
                        .collect(),
                );
            }
            WalkEvent::Enter(n) if n.kind() == SyntaxKind::NODE_ATTR_SET => {
                return collect_bindings(&n, prefix, category, Default::default());
            }
            _ => (),
        }
    }

    vec![]
}

fn retrieve_description(nix: &rnix::Root, description: &str, category: &str) -> String {
    format!(
        "# {} {{#sec-functions-library-{}}}\n{}\n",
        description,
        category,
        &nix.syntax()
            .first_child()
            .and_then(|node| retrieve_doc_comment(&node, false))
            .and_then(|comment| handle_indentation(&comment))
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
    let description = retrieve_description(&nix, &opts.description, &opts.category);

    // TODO: move this to commonmark.rs
    writeln!(output, "{}", description).expect("Failed to write header");

    for entry in collect_entries(nix, &opts.prefix, &opts.category) {
        entry
            .write_section(&locs, &mut output)
            .expect("Failed to write section")
    }
}

#[test]
fn test_main() {
    let mut output = Vec::new();
    let src = fs::read_to_string("test/strings.nix").unwrap();
    let locs = serde_json::from_str(&fs::read_to_string("test/strings.json").unwrap()).unwrap();
    let nix = rnix::Root::parse(&src).ok().expect("failed to parse input");
    let desc = "string manipulation functions";
    let prefix = "lib";
    let category = "strings";

    // TODO: move this to commonmark.rs
    writeln!(
        output,
        "# {} {{#sec-functions-library-{}}}\n",
        desc, category
    )
    .expect("Failed to write header");

    for entry in collect_entries(nix, prefix, category) {
        entry
            .write_section(&locs, &mut output)
            .expect("Failed to write section")
    }

    let output = String::from_utf8(output).expect("not utf8");

    insta::assert_snapshot!(output);
}

#[test]
fn test_description_of_lib_debug() {
    let mut output = Vec::new();
    let src = fs::read_to_string("test/lib-debug.nix").unwrap();
    let nix = rnix::Root::parse(&src).ok().expect("failed to parse input");
    let prefix = "lib";
    let category = "debug";
    let desc = retrieve_description(&nix, &"Debug", category);
    writeln!(output, "{}", desc).expect("Failed to write header");

    for entry in collect_entries(nix, prefix, category) {
        entry
            .write_section(&Default::default(), &mut output)
            .expect("Failed to write section")
    }

    let output = String::from_utf8(output).expect("not utf8");

    insta::assert_snapshot!(output);
}

#[test]
fn test_arg_formatting() {
    let mut output = Vec::new();
    let src = fs::read_to_string("test/arg-formatting.nix").unwrap();
    let nix = rnix::Root::parse(&src).ok().expect("failed to parse input");
    let prefix = "lib";
    let category = "options";

    for entry in collect_entries(nix, prefix, category) {
        entry
            .write_section(&Default::default(), &mut output)
            .expect("Failed to write section")
    }

    let output = String::from_utf8(output).expect("not utf8");

    insta::assert_snapshot!(output);
}

#[test]
fn test_inherited_exports() {
    let mut output = Vec::new();
    let src = fs::read_to_string("test/inherited-exports.nix").unwrap();
    let nix = rnix::Root::parse(&src).ok().expect("failed to parse input");
    let prefix = "lib";
    let category = "let";

    for entry in collect_entries(nix, prefix, category) {
        entry
            .write_section(&Default::default(), &mut output)
            .expect("Failed to write section")
    }

    let output = String::from_utf8(output).expect("not utf8");

    insta::assert_snapshot!(output);
}

#[test]
fn test_line_comments() {
    let mut output = Vec::new();
    let src = fs::read_to_string("test/line-comments.nix").unwrap();
    let nix = rnix::Root::parse(&src).ok().expect("failed to parse input");
    let prefix = "lib";
    let category = "let";

    for entry in collect_entries(nix, prefix, category) {
        entry
            .write_section(&Default::default(), &mut output)
            .expect("Failed to write section")
    }

    let output = String::from_utf8(output).expect("not utf8");

    insta::assert_snapshot!(output);
}

#[test]
fn test_multi_line() {
    let mut output = Vec::new();
    let src = fs::read_to_string("test/multi-line.nix").unwrap();
    let nix = rnix::Root::parse(&src).ok().expect("failed to parse input");
    let prefix = "lib";
    let category = "let";

    for entry in collect_entries(nix, prefix, category) {
        entry
            .write_section(&Default::default(), &mut output)
            .expect("Failed to write section")
    }

    let output = String::from_utf8(output).expect("not utf8");

    insta::assert_snapshot!(output);
}
