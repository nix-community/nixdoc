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

//! This module implements CommonMark output for a struct
//! representing a single entry in the manual.

use serde::Serialize;

/// Represent a single function argument name and its (optional)
/// doc-string.
#[derive(Clone, Debug, Serialize)]
pub struct SingleArg {
    pub name: String,
    pub doc: Option<String>,
}

/// Represent a function argument, which is either a flat identifier
/// or a pattern set.
#[derive(Clone, Debug, Serialize)]
pub enum Argument {
    /// Flat function argument (e.g. `n: n * 2`).
    Flat(SingleArg),

    /// Pattern function argument (e.g. `{ name, age }: ...`)
    Pattern(Vec<SingleArg>),
}

impl Argument {
    /// Write CommonMark structure for a single function argument.
    /// We use the definition list extension, which prepends each argument with `: `.
    /// For pattern arguments, we create a nested definition list.
    fn format_argument(self) -> String {
        match self {
            // Write a flat argument entry, e.g. `id = x: x`
            //
            // `x`
            // : Function argument
            Argument::Flat(arg) => {
                format!(
                    "`{}`\n\n: {}\n\n",
                    arg.name,
                    handle_indentation(arg.doc.unwrap_or("Function argument".into()).trim())
                )
            }

            // Write a pattern argument entry and its individual
            // parameters as a nested structure, e.g.:
            //
            // `foo = { a }: a`
            //
            // structured function argument
            // : `a`
            //   : Function argument
            Argument::Pattern(pattern_args) => {
                let mut inner = String::new();
                for pattern_arg in pattern_args {
                    inner += &Argument::Flat(pattern_arg).format_argument();
                }

                let indented = textwrap::indent(&inner, "  ");

                format!(
                    // The `:` creates another definition list of which `indented` is the term.
                    "structured function argument\n\n: {}",
                    // drop leading indentation on the first line, the `: ` serves this function
                    // already.
                    indented.trim_start()
                )
            }
        }
    }
}

/// Since the first line starts with `: `, indent every other line by 2 spaces, so
/// that the text aligns, to result in:
///
/// : first line
///   every other line
fn handle_indentation(raw: &str) -> String {
    match raw.split_once('\n') {
        Some((first, rest)) => {
            format!("{}\n{}", first, textwrap::indent(rest, "  "))
        }
        None => raw.into(),
    }
}

/// Generate the identifier for CommonMark.
/// ident is used as URL Encoded link to the function and has thus stricter rules (i.e. "' " in "lib.map' "  is not allowed).
pub(crate) fn get_identifier(prefix: &String, category: &String, name: &String) -> String {
    let name_prime = name.replace('\'', "-prime");
    vec![prefix, category, &name_prime]
        .into_iter()
        .filter(|x| !x.is_empty())
        .cloned()
        .collect::<Vec<String>>()
        .join(".")
}

/// Generate the title for CommonMark.
/// the title is the human-readable name of the function.
pub(crate) fn get_title(prefix: &String, category: &String, name: &String) -> String {
    vec![prefix, category, name]
        .into_iter()
        .filter(|x| !x.is_empty())
        .cloned()
        .collect::<Vec<String>>()
        .join(".")
}

/// Represents a single manual section describing a library function.
#[derive(Clone, Debug, Serialize)]
pub struct ManualEntry {
    /// Prefix for the category (e.g. 'lib' or 'utils').
    pub prefix: String,

    /// Name of the function category (e.g. 'strings', 'trivial', 'attrsets').
    pub category: String,

    /// Location of the function.
    pub location: Option<String>,

    /// Name of the section (used as the title).
    pub name: String,

    /// Type signature (if provided). This is not actually a checked
    /// type signature in any way.
    pub fn_type: Option<String>,

    /// Primary description of the entry. Each entry is written as a
    /// separate paragraph.
    pub description: Vec<String>,

    /// Usage example for the entry.
    pub example: Option<String>,

    /// Arguments of the function.
    pub args: Vec<Argument>,
}

impl ManualEntry {
    pub(crate) fn get_ident_title(&self) -> (String, String) {
        let ident = get_identifier(&self.prefix, &self.category, &self.name);
        let title = get_title(&self.prefix, &self.category, &self.name);
        (ident, title)
    }

    /// Write a single CommonMark entry for a documented Nix function.
    ///
    /// # Arguments
    ///
    /// - `anchor_prefix`: The prefix to use for the anchor links.
    ///   In Nixpkgs this would be "function-library-".
    /// - `output`: The output string to append the CommonMark onto.
    pub fn write_section(self, anchor_prefix: &str, output: &mut String) -> String {
        let (ident, title) = self.get_ident_title();
        output.push_str(&format!(
            "## `{}` {{#{}{}}}\n\n",
            title, anchor_prefix, ident
        ));

        // <subtitle> (type signature)
        if let Some(t) = &self.fn_type {
            if t.lines().count() > 1 {
                output.push_str(&format!("**Type**:\n```\n{}\n```\n\n", t));
            } else {
                output.push_str(&format!("**Type**: `{}`\n\n", t));
            }
        }

        // Primary doc string
        // TODO: Split paragraphs?
        for paragraph in &self.description {
            output.push_str(&format!("{}\n\n", paragraph));
        }

        // Function argument names
        if !self.args.is_empty() {
            for arg in self.args {
                output.push_str(&format!("{}\n", arg.format_argument()));
            }
        }

        // Example program listing (if applicable)
        //
        // TODO: In grhmc's version there are multiple (named)
        // examples, how can this be achieved automatically?
        if let Some(example) = &self.example {
            output.push_str(&format!(
                "::: {{.example #{}example-{}}}\n",
                anchor_prefix, ident
            ));
            output.push_str(&format!("# `{}` usage example\n\n", title));
            output.push_str(&format!("```nix\n{}\n```\n:::\n\n", example.trim()));
        }

        if let Some(loc) = self.location {
            output.push_str(&String::from(format!("Located at {loc}.\n\n")));
        }

        output.to_string()
    }
}
