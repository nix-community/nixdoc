use comrak::{
    nodes::{AstNode, NodeValue},
    parse_document, Arena, ComrakOptions, Options,
};
use textwrap::dedent;

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

    // This could only fail if we transform the AST in a way that is not supported by the markdown renderer.
    // Since the AST stems from comrak itself, this should never happen.
    comrak::format_commonmark(root, &options, &mut markdown_output)
        .expect("Failed to format markdown");
    // We can safely assume that the output is valid UTF-8, since comrak uses rust strings which are valid UTF-8.
    String::from_utf8(markdown_output).unwrap()
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
