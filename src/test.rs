use rnix;
use std::fs;
use std::path::PathBuf;

use crate::{
    collect_entries, format::shift_headings, main_with_options, retrieve_description, ManualEntry,
    Options,
};

#[test]
fn test_main() {
    let options = Options {
        prefix: String::from("lib"),
        anchor_prefix: String::from("function-library-"),
        json_output: false,
        category: String::from("strings"),
        description: String::from("string manipulation functions"),
        file: PathBuf::from("test/strings.nix"),
        locs: Some(PathBuf::from("test/strings.json")),
    };

    let output = main_with_options(options);

    insta::assert_snapshot!(output);
}

#[test]
fn test_main_minimal() {
    let options = Options {
        prefix: String::from(""),
        anchor_prefix: String::from(""),
        json_output: false,
        category: String::from(""),
        description: String::from(""),
        file: PathBuf::from("test/strings.nix"),
        locs: Some(PathBuf::from("test/strings.json")),
    };

    let output = main_with_options(options);

    insta::assert_snapshot!(output);
}

#[test]
fn test_json_output() {
    let options = Options {
        prefix: String::from("lib"),
        anchor_prefix: String::from("function-library-"),
        json_output: true,
        category: String::from("strings"),
        description: String::from("string manipulation functions"),
        file: PathBuf::from("test/strings.nix"),
        locs: Some(PathBuf::from("test/strings.json")),
    };

    let output = main_with_options(options);

    insta::assert_snapshot!(output);
}

#[test]
fn test_description_of_lib_debug() {
    let src = fs::read_to_string("test/lib-debug.nix").unwrap();
    let nix = rnix::Root::parse(&src).ok().expect("failed to parse input");
    let prefix = "lib";
    let category = "debug";
    let desc = retrieve_description(&nix, &"Debug", category);
    let mut output = String::from(desc) + "\n";

    for entry in collect_entries(nix, prefix, category, &Default::default()) {
        entry.write_section("function-library-", &mut output);
    }

    insta::assert_snapshot!(output);
}

#[test]
fn test_arg_formatting() {
    let mut output = String::from("");
    let src = fs::read_to_string("test/arg-formatting.nix").unwrap();
    let nix = rnix::Root::parse(&src).ok().expect("failed to parse input");
    let prefix = "lib";
    let category = "options";

    for entry in collect_entries(nix, prefix, category, &Default::default()) {
        entry.write_section("function-library-", &mut output);
    }

    insta::assert_snapshot!(output);
}

#[test]
fn test_inherited_exports() {
    let mut output = String::from("");
    let src = fs::read_to_string("test/inherited-exports.nix").unwrap();
    let nix = rnix::Root::parse(&src).ok().expect("failed to parse input");
    let prefix = "lib";
    let category = "let";

    for entry in collect_entries(nix, prefix, category, &Default::default()) {
        entry.write_section("function-library-", &mut output);
    }

    insta::assert_snapshot!(output);
}

#[test]
fn test_line_comments() {
    let mut output = String::from("");
    let src = fs::read_to_string("test/line-comments.nix").unwrap();
    let nix = rnix::Root::parse(&src).ok().expect("failed to parse input");
    let prefix = "lib";
    let category = "let";

    for entry in collect_entries(nix, prefix, category, &Default::default()) {
        entry.write_section("function-library-", &mut output);
    }

    insta::assert_snapshot!(output);
}

#[test]
fn test_multi_line() {
    let mut output = String::from("");
    let src = fs::read_to_string("test/multi-line.nix").unwrap();
    let nix = rnix::Root::parse(&src).ok().expect("failed to parse input");
    let prefix = "lib";
    let category = "let";

    for entry in collect_entries(nix, prefix, category, &Default::default()) {
        entry.write_section("function-library-", &mut output);
    }

    insta::assert_snapshot!(output);
}

#[test]
fn test_doc_comment() {
    let mut output = String::from("");
    let src = fs::read_to_string("test/doc-comment.nix").unwrap();
    let nix = rnix::Root::parse(&src).ok().expect("failed to parse input");
    let prefix = "lib";
    let category = "debug";

    for entry in collect_entries(nix, prefix, category, &Default::default()) {
        entry.write_section("function-library-", &mut output);
    }

    insta::assert_snapshot!(output);
}

#[test]
fn test_commonmark() {
    let src = fs::read_to_string("test/commonmark.md").unwrap();

    let output = shift_headings(&src, 0);

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
    let src = fs::read_to_string("test/doc-comment-sec-heading.nix").unwrap();
    let nix = rnix::Root::parse(&src).ok().expect("failed to parse input");
    let prefix = "lib";
    let category = "debug";
    let desc = retrieve_description(&nix, &"Debug", category);
    let mut output = String::from(desc) + "\n";

    for entry in collect_entries(nix, prefix, category, &Default::default()) {
        entry.write_section("function-library-", &mut output);
    }

    insta::assert_snapshot!(output);
}

#[test]
fn test_doc_comment_no_duplicate_arguments() {
    let src = fs::read_to_string("test/doc-comment-arguments.nix").unwrap();
    let nix = rnix::Root::parse(&src).ok().expect("failed to parse input");
    let prefix = "lib";
    let category = "debug";
    let desc = retrieve_description(&nix, &"Debug", category);
    let mut output = String::from(desc) + "\n";

    for entry in collect_entries(nix, prefix, category, &Default::default()) {
        entry.write_section("function-library-", &mut output);
    }

    insta::assert_snapshot!(output);
}

#[test]
fn test_empty_prefix() {
    let test_entry = ManualEntry {
        args: vec![],
        category: "test".to_string(),
        location: None,
        description: vec![],
        example: None,
        fn_type: None,
        name: "mapSimple'".to_string(),
        prefix: "".to_string(),
    };

    let (ident, title) = test_entry.get_ident_title();

    assert_eq!(ident, "test.mapSimple-prime");
    assert_eq!(title, "test.mapSimple'");
}

#[test]
fn test_patterns() {
    let mut output = String::from("");
    let src = fs::read_to_string("test/patterns.nix").unwrap();
    let nix = rnix::Root::parse(&src).ok().expect("failed to parse input");
    let prefix = "lib";
    let category = "debug";

    for entry in collect_entries(nix, prefix, category, &Default::default()) {
        entry.write_section("function-library-", &mut output);
    }

    insta::assert_snapshot!(output);
}

#[test]
fn test_let_ident() {
    let mut output = String::from("");
    let src = fs::read_to_string("test/let-ident.nix").unwrap();
    let nix = rnix::Root::parse(&src).ok().expect("failed to parse input");
    let prefix = "lib";
    let category = "math";

    for entry in collect_entries(nix, prefix, category, &Default::default()) {
        entry.write_section("function-library-", &mut output);
    }

    insta::assert_snapshot!(output);
}

#[test]
fn test_let_ident_chained() {
    let mut output = String::from("");
    let src = fs::read_to_string("test/let-ident-chained.nix").unwrap();
    let nix = rnix::Root::parse(&src).ok().expect("failed to parse input");
    let prefix = "lib";
    let category = "math";

    for entry in collect_entries(nix, prefix, category, &Default::default()) {
        entry.write_section("function-library-", &mut output);
    }

    insta::assert_snapshot!(output);
}
