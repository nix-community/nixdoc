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

    let mut curr_fence: Option<(usize, char)> = None;
    for raw_line in raw.split_inclusive('\n') {
        // Code blocks can only start with backticks or tildes
        // code fences can be indented by 0-3 spaces see commonmark spec.
        let fence_line = &trim_leading_whitespace(raw_line, 3);
        if fence_line.starts_with("```") | fence_line.starts_with("~~~") {
            let fence_info = get_fence(fence_line, true);
            if curr_fence.is_none() {
                // Start of code block
                curr_fence = fence_info;
            } else {
                // Possible end of code block. Ending fences cannot have info strings
                match (curr_fence, get_fence(fence_line, false)) {
                    // End of code block must have the same fence type as the start (~~~ or ```)
                    // Code blocks must be ended with at least the same number of backticks or tildes as the start fence
                    (Some((start_count, start_char)), Some((end_count, end_char))) => {
                        if start_count <= end_count && start_char == end_char {
                            // End of code block (same fence as start)
                            curr_fence = None;
                        }
                    }
                    _ => {}
                };
            }
        }

        // Remove up to 0-3 leading whitespaces.
        // If the line has 4 or more whitespaces it is not a heading according to commonmark spec.
        let heading_line = &trim_leading_whitespace(raw_line, 3);
        if curr_fence.is_none() && heading_line.starts_with('#') {
            let heading = handle_heading(heading_line, levels);
            result.push_str(&heading);
        } else {
            result.push_str(raw_line);
        }
    }
    result
}

/// Removes leading whitespaces from code fences if present
/// However maximum of [max] whitespaces are removed.
/// This is useful for code fences may have leading whitespaces (0-3).
fn trim_leading_whitespace(input: &str, max: usize) -> String {
    let mut count = 0;
    input
        .trim_start_matches(|c: char| {
            if c.is_whitespace() && count < max {
                count += 1;
                true
            } else {
                false
            }
        })
        .to_string()
}
/// A function that returns the count of a code fence line.
/// Param [allow_info] allows to keep info strings in code fences.
/// Ending fences cannot have info strings
pub fn get_fence(line: &str, allow_info: bool) -> Option<(usize, char)> {
    let mut chars = line.chars();
    if let Some(first_char) = chars.next() {
        if first_char == '`' || first_char == '~' {
            let mut count = 1;
            for ch in chars {
                if ch == first_char {
                    // count the number of repeated code fence characters
                    count += 1;
                } else {
                    if !allow_info && ch != '\n' {
                        // info string is not allowed this is not a code fence
                        return None;
                    }
                    return Some((count, first_char));
                }
            }
            return Some((count, first_char));
        }
    }
    None
}
// Dumb heading parser.
pub fn handle_heading(line: &str, levels: usize) -> String {
    let chars = line.chars();

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
