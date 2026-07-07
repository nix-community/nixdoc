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
        category: Some(String::from("strings")),
        description: Some(String::from("string manipulation functions")),
        file: Some(PathBuf::from("test/strings.nix")),
        locs: Some(PathBuf::from("test/strings.json")),
        manifest: None,
        output: None,
        root: None,
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
        category: Some(String::from("")),
        description: Some(String::from("")),
        file: Some(PathBuf::from("test/strings.nix")),
        locs: Some(PathBuf::from("test/strings.json")),
        manifest: None,
        output: None,
        root: None,
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
        category: Some(String::from("strings")),
        description: Some(String::from("string manipulation functions")),
        file: Some(PathBuf::from("test/strings.nix")),
        locs: Some(PathBuf::from("test/strings.json")),
        manifest: None,
        output: None,
        root: None,
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
fn test_manifest_export() {
    let options = Options {
        prefix: String::from("lib"),
        anchor_prefix: String::from("function-library-"),
        json_output: false,
        category: None,
        description: None,
        file: None,
        locs: None,
        manifest: Some(PathBuf::from("test/manifest.json")),
        output: None,
        root: Some(PathBuf::from("test")),
    };

    let output = crate::export::run_export(&options);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    let entries = json["entries"].as_array().unwrap();
    let ids: Vec<&str> = entries.iter().map(|e| e["id"].as_str().unwrap()).collect();

    // `include` reaches `mkDerivation` at nesting level
    let mk = entries
        .iter()
        .find(|e| e["id"] == "pkgs.stdenv.mkDerivation")
        .expect("mkDerivation must be present");
    assert_eq!(mk["attrPath"], "pkgs.stdenv.mkDerivation");
    assert_eq!(mk["name"], "mkDerivation");
    assert_eq!(mk["source"]["file"], "make-derivation-lite.nix");
    assert_eq!(mk["source"]["line"], 35);
    assert_eq!(mk["source"]["column"], 3);
    assert!(mk["description"]
        .as_str()
        .unwrap()
        .contains("creates a derivation"));
    // A single source may place a function in several groups at once.
    let mk_groups: Vec<&str> = mk["groups"]
        .as_array()
        .unwrap()
        .iter()
        .map(|g| g.as_str().unwrap())
        .collect();
    assert_eq!(mk_groups, vec!["builders", "stdenv"]);

    // `include` scoping: overrideAttrs is documented but not included.
    // overrideAtts is documented, we include it only as example, in a real usage we would include it.
    // The flag exists to allow carving out edge cases.
    assert!(!ids.contains(&"pkgs.stdenv.overrideAttrs"));
    // legacy-comments are excluded.
    assert!(!ids.iter().any(|id| id.ends_with("legacyThing")));

    assert!(ids.contains(&"lib.strings.optionalString"));
    assert!(!ids.iter().any(|id| id.ends_with("concatStrings"))); // legacy-only
    assert!(!ids.iter().any(|id| id.ends_with("undocumented"))); // no doc-comment

    // The same function via two sources merges its group set.
    let opt = entries
        .iter()
        .find(|e| e["id"] == "lib.strings.optionalString")
        .expect("optionalString must be present");
    let opt_groups: Vec<&str> = opt["groups"]
        .as_array()
        .unwrap()
        .iter()
        .map(|g| g.as_str().unwrap())
        .collect();
    assert_eq!(opt_groups, vec!["default", "strings"]);

    // One description per group: manifest entry wins, else file header.
    let groups = json["groups"].as_array().unwrap();
    let desc_of = |id: &str| -> String {
        groups
            .iter()
            .find(|g| g["id"] == id)
            .unwrap_or_else(|| panic!("group `{id}` must be present"))["description"]
            .as_str()
            .unwrap()
            .to_string()
    };
    assert_eq!(desc_of("stdenv"), "the standard build environment"); // manifest override
    assert_eq!(desc_of("strings"), "String manipulation functions."); // file header
    assert_eq!(desc_of("builders"), "Standard environment build helpers."); // file header

    // `id` slugs `'` to `-prime`; `attrPath`/`name` keep the apostrophe.
    let mp = entries
        .iter()
        .find(|e| e["id"] == "lib.strings.mapAttrs-prime")
        .expect("primed name must be present with slugged id");
    assert_eq!(mp["attrPath"], "lib.strings.mapAttrs'");
    assert_eq!(mp["name"], "mapAttrs'");

    // Entries are sorted by `id` ascending.
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    assert_eq!(ids, sorted);

    insta::assert_snapshot!(output);
}

#[test]
fn test_manifest_export_position() {
    let output = crate::export::run_export(&export_options("test/manifest-position.json"));
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    let entry = &json["entries"][0];
    assert_eq!(entry["name"], "fun");
    assert_eq!(entry["source"]["file"], "position.nix");
    assert_eq!(entry["source"]["line"], 5);
    assert_eq!(entry["source"]["column"], 3);
}

// Helper function to create Options for manifest tests.
fn export_options(manifest: &str) -> Options {
    Options {
        prefix: String::from("lib"),
        anchor_prefix: String::from("function-library-"),
        json_output: false,
        category: None,
        description: None,
        file: None,
        locs: None,
        manifest: Some(PathBuf::from(manifest)),
        output: None,
        root: Some(PathBuf::from("test")),
    }
}

#[test]
#[should_panic(expected = "unsupported manifest version 2")]
fn test_manifest_rejects_unknown_version() {
    crate::export::run_export(&export_options("test/manifest-bad-version.json"));
}

#[test]
#[should_panic(expected = "unknown field")]
fn test_manifest_rejects_unknown_field() {
    crate::export::run_export(&export_options("test/manifest-unknown-field.json"));
}

#[test]
#[should_panic(expected = "empty `include` list")]
fn test_manifest_rejects_empty_include() {
    crate::export::run_export(&export_options("test/manifest-empty-include.json"));
}

#[test]
fn test_manifest_mode_file() {
    let output = crate::export::run_export(&export_options("test/manifest-file-function.json"));
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    let entries = json["entries"].as_array().unwrap();

    // The whole file is one function: exactly one entry, named by `path`.
    assert_eq!(entries.len(), 1);
    let e = &entries[0];
    assert_eq!(e["id"], "makeDiskImage");
    assert_eq!(e["attrPath"], "makeDiskImage");
    assert_eq!(e["name"], "makeDiskImage");

    // Lambda doc is the fileheader.
    assert!(e["description"]
        .as_str()
        .unwrap()
        .contains("Create a bootable disk image"));

    // Location points at the top-level lambda
    assert_eq!(e["source"]["file"], "make-disk-image-lite.nix");
    assert_eq!(e["source"]["line"], 12);
    assert_eq!(e["source"]["column"], 1);

    // The file header also feeds the group description.
    let groups = json["groups"].as_array().unwrap();
    let images = groups
        .iter()
        .find(|g| g["id"] == "images")
        .expect("group `images` must be present");
    assert!(images["description"]
        .as_str()
        .unwrap()
        .contains("Create a bootable disk image"));
}

#[test]
#[should_panic(expected = "cannot also use")]
fn test_manifest_mode_file_rejects_include() {
    crate::export::run_export(&export_options("test/manifest-file-function-bad.json"));
}

#[test]
#[should_panic(expected = "must be bounded")]
fn test_manifest_deep_requires_include() {
    crate::export::run_export(&export_options("test/manifest-deep-no-include.json"));
}
