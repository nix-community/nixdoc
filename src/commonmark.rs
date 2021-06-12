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

use std::io::Write;
use failure::Error;

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

impl Argument {
    /// Write CommonMark structure for a single function argument.
    fn write_argument(self) -> Result<(), Error> {
        match self {
            // Write a flat argument entry.
            Argument::Flat(arg) => {
                println!("`{}`\n", &arg.name);
                println!(": {}\n", arg.doc.unwrap_or("Function argument".into()).trim())
            },

            // Write a pattern argument entry and its individual
            // parameters as a nested structure.
            Argument::Pattern(pattern_args) => {
                println!("### pattern - structured function argument\n");
                for pattern_arg in pattern_args {
                    Argument::Flat(pattern_arg)
                        .write_argument()?;
                }
            },
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
    pub fn write_section(self) -> Result<(), Error> {
        let title = format!("lib.{}.{}", self.category, self.name);
        let ident = format!("lib.{}.{}", self.category, self.name.replace("'", "-prime"));

        println!("## `{}` {{#function-library-{}}}\n", title, ident);

        // <subtitle> (type signature)
        if let Some(t) = &self.fn_type {
            println!("`{}`\n", t);
        }

        // Primary doc string
        // TODO: Split paragraphs?
        for paragraph in &self.description {
            println!("{}\n", paragraph);
        }

        // Function argument names
        if !self.args.is_empty() {

            for arg in self.args {
                arg.write_argument()?;
            }
        }

        // Example program listing (if applicable)
        //
        // TODO: In grhmc's version there are multiple (named)
        // examples, how can this be achieved automatically?
        if let Some(example) = &self.example {
            println!("### {} usage example {{#function-library-example-{}}}\n", title, ident);
            println!("```nix{}```\n", example);
        }

        // TODO: add source links
        //println!("[Source](#function-location-{})", ident);

        Ok(())
    }
}
