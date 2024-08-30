use rnix::{
    ast::{AstToken, Comment, Expr, Lambda, Param},
    SyntaxKind, SyntaxNode,
};
use rowan::ast::AstNode;
use std::collections::HashMap;

use crate::{
    commonmark::{Argument, ManualEntry, SingleArg},
    format::handle_indentation,
    get_identifier, retrieve_doc_comment, DocComment,
};

#[derive(Debug)]
pub struct LegacyDocItem {
    pub name: String,
    pub comment: DocComment,
    pub args: Vec<Argument>,
}

impl LegacyDocItem {
    pub fn into_entry(
        self,
        prefix: &str,
        category: &str,
        locs: &HashMap<String, String>,
    ) -> ManualEntry {
        let ident = get_identifier(
            &prefix.to_string(),
            &category.to_string(),
            &self.name.to_string(),
        );

        ManualEntry {
            prefix: prefix.to_string(),
            category: category.to_string(),
            location: locs.get(&ident).cloned(),
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
pub fn retrieve_legacy_comment(node: &SyntaxNode, allow_line_comments: bool) -> Option<String> {
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

/// Traverse directly chained nix lambdas and collect the identifiers of all lambda arguments
/// until an unexpected AST node is encountered.
pub fn collect_lambda_args_legacy(mut lambda: Lambda) -> Vec<Argument> {
    let mut args = vec![];

    loop {
        match lambda.param().unwrap() {
            // a variable, e.g. `x:` in `id = x: x`
            // Single args are not supported by RFC145, due to ambiguous placement rules.
            Param::IdentParam(id) => {
                args.push(Argument::Flat(SingleArg {
                    name: id.to_string(),
                    doc: handle_indentation(
                        &retrieve_legacy_comment(id.syntax(), true).unwrap_or_default(),
                    ),
                }));
            }
            // an ident in a pattern, e.g. `a` in `foo = { a }: a`
            Param::Pattern(pat) => {
                // collect doc-comments for each lambda formal too
                // Lambda formals are supported by RFC145
                let pattern_vec: Vec<_> = pat
                    .pat_entries()
                    .map(|entry| SingleArg {
                        name: entry.ident().unwrap().to_string(),
                        doc: handle_indentation(
                            &retrieve_doc_comment(entry.syntax(), Some(1))
                                .or(retrieve_legacy_comment(entry.syntax(), true))
                                .unwrap_or_default(),
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
