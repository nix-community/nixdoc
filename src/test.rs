use rnix;
use std::fs;
use std::path::PathBuf;

use std::io::Write;

use crate::{
    collect_entries,
    format::shift_headings,
    frontmatter::{get_imported_content, FrontmatterError, FrontmatterErrorKind},
    retrieve_description,
};

#[test]
fn test_main() {
    let mut output = Vec::new();
    let src_path = PathBuf::from("test/strings.nix");
    let src = fs::read_to_string(&src_path).unwrap();
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

    for entry in collect_entries(nix, prefix, category, &src_path) {
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
    let src_path = PathBuf::from("test/lib-debug.nix");
    let src = fs::read_to_string(&src_path).unwrap();
    let nix = rnix::Root::parse(&src).ok().expect("failed to parse input");
    let prefix = "lib";
    let category = "debug";
    let desc = retrieve_description(&nix, &"Debug", category, &src_path);
    writeln!(output, "{}", desc).expect("Failed to write header");

    for entry in collect_entries(nix, prefix, category, &src_path) {
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
    let src_path = PathBuf::from("test/arg-formatting.nix");
    let src = fs::read_to_string(&src_path).unwrap();
    let nix = rnix::Root::parse(&src).ok().expect("failed to parse input");
    let prefix = "lib";
    let category = "options";

    for entry in collect_entries(nix, prefix, category, &src_path) {
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
    let src_path = PathBuf::from("test/inherited-exports.nix");
    let src = fs::read_to_string(&src_path).unwrap();
    let nix = rnix::Root::parse(&src).ok().expect("failed to parse input");
    let prefix = "lib";
    let category = "let";

    for entry in collect_entries(nix, prefix, category, &src_path) {
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
    let src_path = PathBuf::from("test/line-comments.nix");
    let src = fs::read_to_string(&src_path).unwrap();
    let nix = rnix::Root::parse(&src).ok().expect("failed to parse input");
    let prefix = "lib";
    let category = "let";

    for entry in collect_entries(nix, prefix, category, &src_path) {
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
    let src_path = PathBuf::from("test/multi-line.nix");
    let src = fs::read_to_string(&src_path).unwrap();
    let nix = rnix::Root::parse(&src).ok().expect("failed to parse input");
    let prefix = "lib";
    let category = "let";

    for entry in collect_entries(nix, prefix, category, &src_path) {
        entry
            .write_section(&Default::default(), &mut output)
            .expect("Failed to write section")
    }

    let output = String::from_utf8(output).expect("not utf8");

    insta::assert_snapshot!(output);
}

#[test]
fn test_doc_comment() {
    let mut output = Vec::new();
    let src_path = PathBuf::from("test/doc-comment.nix");
    let src = fs::read_to_string(&src_path).unwrap();
    let nix = rnix::Root::parse(&src).ok().expect("failed to parse input");
    let prefix = "lib";
    let category = "debug";

    for entry in collect_entries(nix, prefix, category, &src_path) {
        entry
            .write_section(&Default::default(), &mut output)
            .expect("Failed to write section")
    }

    let output = String::from_utf8(output).expect("not utf8");

    insta::assert_snapshot!(output);
}

#[test]
fn test_headings() {
    let src = fs::read_to_string("test/headings.md").unwrap();

    let output = shift_headings(&src, 2);

    insta::assert_snapshot!(output);
}

#[test]
fn test_doc_comment_section_description() {
    let mut output = Vec::new();
    let src_path = PathBuf::from("test/doc-comment-sec-heading.nix");
    let src = fs::read_to_string(&src_path).unwrap();
    let nix = rnix::Root::parse(&src).ok().expect("failed to parse input");
    let prefix = "lib";
    let category = "debug";
    let desc = retrieve_description(&nix, &"Debug", category, &src_path);
    writeln!(output, "{}", desc).expect("Failed to write header");

    for entry in collect_entries(nix, prefix, category, &src_path) {
        entry
            .write_section(&Default::default(), &mut output)
            .expect("Failed to write section")
    }

    let output = String::from_utf8(output).expect("not utf8");

    insta::assert_snapshot!(output);
}

#[test]
fn test_doc_comment_no_duplicate_arguments() {
    let mut output = Vec::new();
    let src_path = PathBuf::from("test/doc-comment-arguments.nix");
    let src = fs::read_to_string(&src_path).unwrap();
    let nix = rnix::Root::parse(&src).ok().expect("failed to parse input");
    let prefix = "lib";
    let category = "debug";
    let desc = retrieve_description(&nix, &"Debug", category, &src_path);
    writeln!(output, "{}", desc).expect("Failed to write header");

    for entry in collect_entries(nix, prefix, category, &src_path) {
        entry
            .write_section(&Default::default(), &mut output)
            .expect("Failed to write section")
    }

    let output = String::from_utf8(output).expect("not utf8");

    insta::assert_snapshot!(output);
}

#[test]
fn test_frontmatter_doc_location_e2e() {
    let mut output = Vec::new();
    let src_path = PathBuf::from("test/frontmatter-doc-location.nix");
    let src = fs::read_to_string(&src_path).unwrap();
    let nix = rnix::Root::parse(&src).ok().expect("failed to parse input");
    let prefix = "lib";
    let category = "debug";
    let desc = retrieve_description(&nix, &"Debug", category, &src_path);
    writeln!(output, "{}", desc).expect("Failed to write header");

    for entry in collect_entries(nix, prefix, category, &src_path) {
        entry
            .write_section(&Default::default(), &mut output)
            .expect("Failed to write section")
    }

    let output = String::from_utf8(output).expect("not utf8");

    insta::assert_snapshot!(output);
}

const NOT_RELATIVE: &str = r#"---
doc_location: /tmp/not-relative.md
---
Other stuff
"#;

#[test]
fn test_frontmatter_doc_location_relative() {
    let base_file = PathBuf::from("test/frontmatter-doc-location.nix");

    let result = get_imported_content(&base_file, NOT_RELATIVE);

    assert_eq!(
        result.unwrap_err().kind,
        FrontmatterErrorKind::DocLocationNotRelativePath
    );
}

const INVALID_TYPE: &str = r#"---
doc_location: 1
---
Other stuff
"#;

#[test]
fn test_frontmatter_doc_location_type() {
    let base_file = PathBuf::from("test/frontmatter-doc-location.nix");

    let result = get_imported_content(&base_file, INVALID_TYPE);

    assert_eq!(result.unwrap_err().kind, FrontmatterErrorKind::InvalidYaml);
}

const FILE_NOT_FOUND: &str = r#"---
doc_location: ./does-not-exist.md
---
Other stuff
"#;

#[test]
fn test_frontmatter_doc_location_file_not_found() {
    let base_file = PathBuf::from("test/frontmatter-doc-location.nix");

    let result = get_imported_content(&base_file, FILE_NOT_FOUND);

    assert_eq!(
        result.unwrap_err().kind,
        FrontmatterErrorKind::DocLocationFileNotFound
    );
}
