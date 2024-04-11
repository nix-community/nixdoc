use comrak::{
    format_commonmark,
    nodes::{AstNode, NodeValue},
    parse_document, Arena, ComrakOptions, Options,
};
use std::io::Write;
use textwrap::dedent; // For using the write! macro

// Your custom renderer
struct CustomRenderer<'a> {
    options: &'a ComrakOptions,
}

impl<'a> CustomRenderer<'a> {
    fn new(options: &'a ComrakOptions) -> Self {
        CustomRenderer { options }
    }

    fn format_node(&self, root: &'a AstNode<'a>, buffer: &mut Vec<u8>) {
        for node in root.children() {
            match &node.data.borrow().value {
                NodeValue::Heading(heading) => {
                    // Handling headings specifically
                    write!(buffer, "{} ", "#".repeat(heading.level as usize)).expect(
                        "Failed to write UTF-8. Make sure files contains only valid UTF-8.",
                    );

                    node.first_child()
                        .map(|child| match child.data.borrow().value {
                            NodeValue::Text(ref text) => {
                                write!(buffer, "{}\n", text).expect("Failed to write UTF-8. Make sure files contains only valid UTF-8.");
                            }
                            _ => (),
                        });
                }
                // Handle other node types using comrak's default behavior
                _ => format_commonmark(node, self.options, buffer)
                    .expect("Failed to format markdown using the default comrak formatter."),
            }
            buffer.push(b'\n');
        }
    }
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
pub fn handle_indentation(raw: &str) -> Option<String> {
    let result: String = match raw.split_once('\n') {
        Some((first, rest)) => {
            format!("{}\n{}", first.trim_start(), dedent(rest))
        }
        None => raw.into(),
    };

    Some(result.trim().to_owned()).filter(|s| !s.is_empty())
}

/// Shift down markdown headings
///
/// Performs a line-wise matching to '# Heading '
///
/// Counts the current numbers of '#' and adds levels: [usize] to them
/// levels := 1; gives
/// '# Heading' -> '## Heading'
///
/// Commonmark markdown has 6 levels of headings. Everything beyond that (e.g., H7) is not supported and may produce unexpected renderings.
/// by default this function makes sure, headings don't exceed the H6 boundary.
/// levels := 2;
/// ...
/// H4 -> H6
/// H6 -> H6
///
pub fn shift_headings(raw: &str, levels: u8) -> String {
    let arena = Arena::new();

    // Change some of the default formatting options for better compatibility with nixos-render-docs (nrd).
    let mut options: Options = ComrakOptions::default();
    // Disable automatic generation of header IDs. nrd will generate them.
    options.extension.header_ids = None;

    // Parse the document into an AST
    let root = parse_document(&arena, raw, &options);
    increase_heading_levels(root, levels);

    let mut markdown_output = vec![];
    let renderer = CustomRenderer::new(&options);
    renderer.format_node(root, &mut markdown_output);

    // We can safely assume that the output is valid UTF-8, since comrak uses rust strings which are valid UTF-8.
    String::from_utf8(markdown_output).expect("Markdown contains invalid UTF-8")
}

// Internal function to operate on the markdown AST
fn increase_heading_levels<'a>(root: &'a AstNode<'a>, levels: u8) {
    for node in root.descendants() {
        match &mut node.data.borrow_mut().value {
            NodeValue::Heading(heading) => {
                // Increase heading level, but don't exceed the max level 6
                heading.level = (heading.level + levels).min(6);
            }
            _ => {}
        }
    }
}
