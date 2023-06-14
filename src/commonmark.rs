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

use failure::Error;
use std::iter::repeat;

use std::io::Write;

/// Represent a single function argument name and its (optional)
/// doc-string.
#[derive(Debug)]
pub struct SingleArg {
    pub name: String,
    pub doc: Option<String>,
}

/// Represent a function argument, which is either a flat identifier
/// or a pattern set.
#[derive(Debug)]
pub enum Argument {
    /// Flat function argument (e.g. `n: n * 2`).
    Flat(SingleArg),

    /// Pattern function argument (e.g. `{ name, age }: ...`)
    Pattern(Vec<SingleArg>),
}

fn print_indented<W: Write>(writer: &mut W, indent: usize, text: &str) -> Result<(), Error> {
    let prefix = repeat(' ').take(indent).collect::<String>();
    writeln!(
        writer,
        "{}",
        text.replace("\r\n", "\n")
            .replace("\n", &format!("\n{prefix}"))
    )?;

    Ok(())
}

impl Argument {
    /// Write CommonMark structure for a single function argument.
    fn write_argument<W: Write>(self, writer: &mut W, indent: usize) -> Result<(), Error> {
        match self {
            // Write a flat argument entry.
            Argument::Flat(arg) => {
                let arg_text = format!(
                    "`{}`\n\n: {}\n\n",
                    arg.name,
                    arg.doc.unwrap_or("Function argument".into()).trim()
                );
                print_indented(writer, indent, &arg_text)?;
            }

            // Write a pattern argument entry and its individual
            // parameters as a nested structure.
            Argument::Pattern(pattern_args) => {
                print_indented(writer, indent, "structured function argument\n\n: ")?;
                for pattern_arg in pattern_args {
                    Argument::Flat(pattern_arg).write_argument(writer, indent + 2)?;
                }
            }
        }

        Ok(())
    }
}

/// Represents a single manual section describing a library function.
#[derive(Debug)]
pub struct ManualEntry {
    /// Name of the function category (e.g. 'strings', 'trivial', 'attrsets')
    pub category: String,

    /// Name of the section (used as the title)
    pub name: String,

    /// Type signature (if provided). This is not actually a checked
    /// type signature in any way.
    pub fn_type: Option<String>,

    /// Primary description of the entry. Each entry is written as a
    /// separate paragraph.
    pub description: Vec<String>,

    /// Usage example for the entry.
    pub example: Option<String>,

    /// Arguments of the function
    pub args: Vec<Argument>,
}

impl ManualEntry {
    /// Write a single CommonMark entry for a documented Nix function.
    pub fn write_section<W: Write>(self, writer: &mut W) -> Result<(), Error> {
        let title = format!("lib.{}.{}", self.category, self.name);
        let ident = format!(
            "lib.{}.{}",
            self.category,
            self.name.replace('\'', "-prime")
        );

        writeln!(writer, "## `{}` {{#function-library-{}}}\n", title, ident)?;

        // <subtitle> (type signature)
        if let Some(t) = &self.fn_type {
            writeln!(writer, "`{}`\n", t)?;
        }

        // Primary doc string
        // TODO: Split paragraphs?
        for paragraph in &self.description {
            writeln!(writer, "{}\n", paragraph)?;
        }

        // Function argument names
        if !self.args.is_empty() {
            for arg in self.args {
                arg.write_argument(writer, 0)?;
            }
        }

        // Example program listing (if applicable)
        //
        // TODO: In grhmc's version there are multiple (named)
        // examples, how can this be achieved automatically?
        if let Some(example) = &self.example {
            writeln!(
                writer,
                "### {} usage example {{#function-library-example-{}}}\n",
                title, ident
            )?;
            writeln!(writer, "```nix{}```\n", example)?;
        }

        // TODO: add source links
        //println!("[Source](#function-location-{})", ident);

        Ok(())
    }
}
