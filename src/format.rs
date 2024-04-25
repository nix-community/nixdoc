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
pub fn shift_headings(raw: &str, levels: usize) -> String {
    let mut result = String::new();

    let mut curr_fence: Option<String> = None;
    for line in raw.split_inclusive('\n') {
        // Code blocks can only start with backticks or tildes
        if line.starts_with("```") | line.starts_with("~~~") {
            if curr_fence.is_none() {
                // Start of code block
                curr_fence = get_fence(line);
            } else if curr_fence.as_deref() == Some(line.trim_end()) {
                // End of code block (same fence as start)
                curr_fence = None;
            }
        }

        if curr_fence.is_none() && line.starts_with('#') {
            let heading = handle_heading(line, levels);
            result.push_str(&heading);
        } else {
            result.push_str(line);
        }
    }
    result
}

pub fn get_fence(line: &str) -> Option<String> {
    let mut chars = line.chars();
    if let Some(first_char) = chars.next() {
        if first_char == '`' || first_char == '~' {
            let mut count = 1;
            for ch in chars {
                if ch == first_char {
                    // count the number of repeated code fence characters
                    count += 1;
                } else {
                    break;
                }
            }
            return Some(std::iter::repeat(first_char).take(count).collect());
        }
    }
    None
}
// Dumb heading parser.
pub fn handle_heading(line: &str, levels: usize) -> String {
    let chars = line.chars();

    // let mut leading_trivials: String = String::new();
    let mut hashes = String::new();
    let mut rest = String::new();
    for char in chars {
        match char {
            '#' if rest.is_empty() => {
                // only collect hashes if no other tokens
                hashes.push(char)
            }
            _ => rest.push(char),
        }
    }
    let new_hashes = match hashes.len() + levels {
        // We reached the maximum heading size.
        6.. => "#".repeat(6),
        _ => "#".repeat(hashes.len() + levels),
    };

    format!("{new_hashes}{rest}")
}
