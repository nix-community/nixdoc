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

//! This module implements a transformation of a subset of Markdown
//! into DocBook format.
//!
//! `nixdoc` does not officially want to commit to using a markup
//! format such as Markdown at the moment, but certain features such
//! as inline code blocks and reference links are nice to have.

use pulldown_cmark::{Parser, Options, Event, Tag};
use xml::writer::{EventWriter, XmlEvent};
use std::io::Write;
use docbook::{element, end, string};
use failure::Error;
use std::mem::replace;

/// When writing tag content, it can either be inserted as characters
/// or as verbatim CDATA. The latter is used for program listings and
/// such.
#[derive(PartialEq)]
enum TextMode { Characters, CDATA(String) }

/// Write the beginning of a tag. Not all Markdown tags are
/// implemented.
fn write_start_tag<'a, W: Write>(tag: Tag<'a>,
                                 w: &mut EventWriter<W>,
                                 text_mode: &mut TextMode) -> Result<(), Error> {
    match tag {
        Tag::Paragraph => element(w, "para"),
        Tag::Code      => element(w, "code"),
        Tag::BlockQuote        => element(w, "blockquote"),

        // All emphasis styles result in the same markup.
        Tag::Strong    => element(w, "emphasis"),
        Tag::Emphasis  => element(w, "emphasis"),
        Tag::Header(_) => element(w, "emphasis"),

        // Only HTML-style links are supported, not internal
        // references.
        Tag::Link(link, _title) => w.write(
            XmlEvent::start_element("link")
                .attr("xlink:href", &link)
        ).map_err(Into::into),

        // Code blocks need to switch text emission into CDATA mode.
        Tag::CodeBlock(_title) => {
            replace(text_mode, TextMode::CDATA(String::new()));
            element(w, "programlisting")
        },

        // Ordered & unordered lists are translated straight into
        // DocBook.
        Tag::List(None)    => element(w, "itemizedlist"),
        Tag::List(Some(_)) => element(w, "orderedlist"),

        // DocBook requires list item text to be wrapped in a
        // paragraph.
        Tag::Item => {
            element(w, "listitem")?;
            element(w, "para")
        },

        // Some tags are ignored, should people try to use them. A
        // warning is printed to stderr to not interrupt docs
        // generation.
        tag => {
            eprintln!("Warning: Markup for '{:?}' is unsupported in nixdoc", tag);
            Ok(())
        },
    }
}

/// Write the end of a tag. For almost all tags this simply ends the
/// current XML element, but some require a bit of special handling.
fn write_end_tag<'a, W: Write>(tag: Tag<'a>,
                               w: &mut EventWriter<W>,
                               text_mode: &mut TextMode) -> Result<(), Error> {
    match tag {
        // List items need to close twice, as there is a list item and
        // it contains a paragraph.
        Tag::Item => { end(w)?; end(w) },

        // The end of code blocks resets to character mode and flushes
        // the CDATA.
        Tag::CodeBlock(_) => {
            if let TextMode::CDATA(cdata) = replace(text_mode, TextMode::Characters) {
                w.write(XmlEvent::cdata(&cdata))?;
            }

            end(w)
        },

        // Everything else is just a tag close.
        _ => end(w),
    }
}

/// Write appropriate DocBook output for a single event to an XML
/// writer.
fn write_event<'a, W: Write>(event: Event<'a>,
                             w: &mut EventWriter<W>,
                             text_mode: &mut TextMode) -> Result<(), Error> {
    match event {
        Event::Start(tag) => write_start_tag(tag, w, text_mode),
        Event::End(tag) => write_end_tag(tag, w, text_mode),

        // Text is written either as a normal string (in characters
        // mode) or as verbatim CDATA if inside a code block.
        //
        // All lines of a program listing must go into the same CDATA
        // block, so they are assembled here and flushed when the code
        // block ends.
        Event::Text(text) => {
            match text_mode {
                TextMode::Characters => string(w, &text),
                TextMode::CDATA(ref mut cdata) => {
                    cdata.push_str(&text);
                    Ok(())
                },
            }
        },
        _ => unimplemented!(),
    }
}

/// Dangling links are interpreted as manual references. Note that
/// these are not currently cross-referenced and could result in
/// broken links if users write ... broken links.
fn reference_link_fn(title: &str, _: &str) -> Option<(String, String)> {
    // The `title` is presumed to be something like `lib.strings.foo`.
    // For now nixdoc only supports function references, so this is
    // interpreted to mean `#function-library-lib.strings.foo`.
    //
    // TODO: -prime!

    Some((format!("#function-library-{}", title), title.to_string()))
}

#[test]
fn test_md() {
    let markdown = r#"This is an **example** doc string.

With some `inline code`.

Dangling link (reference item): [lib.strings.foo]

[a link](https://nixos.org)

Type: foo :: a -> a

```How to frobulate
let a = 5;
in 5 + a
=> 10
```

* first list item
* second list tem

"#;

    // let parser = pulldown_cmark::Parser::new();
    let parser = Parser::new_with_broken_link_callback(
        markdown, Options::empty(), Some(&reference_link_fn)
    );

    use xml::EmitterConfig;
    use std::str;

    let mut bytes = vec![];

    {
        let mut writer = EmitterConfig::new()
            .perform_indent(true)
            .create_writer(&mut bytes);

        let mut text_mode = TextMode::Characters;

        for event in parser {
            println!("{:?}", event);
            write_event(event, &mut writer, &mut text_mode).expect("Write failed");
        }
    }

    println!("\n{}", str::from_utf8(&bytes).unwrap());
}
