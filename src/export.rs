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

//! Export mode: configured by `--manifest`. Emits a single export JSON.
//!
//! Consumes a `.json` that describes how to process a source tree and emits a single JSON artifact.
//!
//! one entry per documented function. Processing all sources in one run

use crate::commonmark::{get_identifier, get_title};
use crate::{retrieve_doc_comment, Options};

use rnix::ast::{AttrSet, HasEntry};
use rnix::{
    ast::{AttrpathValue, LetIn},
    SyntaxKind, SyntaxNode,
};
use rowan::{ast::AstNode, WalkEvent};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

const EXPORT_SCHEMA_VERSION: u32 = 1;
const MANIFEST_SCHEMA_VERSION: u32 = 1;

/// --manifest input JSON
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub version: u32,

    #[serde(default)]
    pub groups: Vec<GroupDef>,

    pub sources: Vec<ManifestItem>,
}

/// A group definition
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroupDef {
    pub id: String,

    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// Top-level attrset and inherits. Such as the format used in "lib/default.nix"
    #[default]
    Flat,
    /// All binding at any ast-depth
    Deep,
    /// The file is itself the documented function
    File,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ManifestItem {
    pub path: String,

    /// Path to the `.nix` file, resolved against `--root`.
    pub file: String,

    #[serde(default)]
    pub groups: Vec<String>,

    /// Discovery strategy. See `Mode`
    #[serde(default)]
    pub mode: Mode,

    #[serde(default)]
    pub include: Option<Vec<String>>,

    /// Name denylist applied after discovery: drop bindings whose local name is
    /// listed. Invalid with `mode: "file"`.
    #[serde(default)]
    pub exclude: Option<Vec<String>>,
}

// Output
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportFile {
    pub schema_version: u32,
    pub groups: Vec<GroupOut>,
    pub entries: Vec<ExportEntry>,
}

// Resolved group
#[derive(Debug, Serialize)]
pub struct GroupOut {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// One output item
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportEntry {
    /// URL-safe id
    pub id: String,
    /// Human readable path
    pub attr_path: String,
    pub name: String,
    pub description: String,
    pub groups: Vec<String>,
    pub source: Source,
}

#[derive(Debug, Serialize)]
pub struct Source {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

pub fn run_export(opts: &Options) -> String {
    let manifest_path = opts
        .manifest
        .as_ref()
        .expect("run_export called without --manifest");
    let manifest_str = fs::read_to_string(manifest_path)
        .unwrap_or_else(|e| panic!("could not read manifest {}: {e}", manifest_path.display()));
    let manifest: Manifest = serde_json::from_str(&manifest_str)
        .unwrap_or_else(|e| panic!("could not parse manifest {}: {e}", manifest_path.display()));

    if manifest.version != MANIFEST_SCHEMA_VERSION {
        panic!(
            "unsupported manifest version {} in {}: this nixdoc supports version {}",
            manifest.version,
            manifest_path.display(),
            MANIFEST_SCHEMA_VERSION,
        );
    }
    for source in &manifest.sources {
        // Reject empty includes
        if matches!(&source.include, Some(names) if names.is_empty()) {
            panic!(
                "manifest source `{}` has an empty `include` list; omit `include` to \
                 keep everything discovered, or list the binding names to select",
                source.path,
            );
        }
        match source.mode {
            // disallow include/exclude in file mode
            Mode::File if source.include.is_some() || source.exclude.is_some() => panic!(
                "manifest source `{}` sets `mode: \"file\"`; it documents the whole \
                 file and cannot also use `include`/`exclude`",
                source.path,
            ),
            // mode deep must be acompanied by include
            Mode::Deep if source.include.is_none() => panic!(
                "manifest source `{}` uses `mode: \"deep\"` without an `include` list; \
                 deep discovery walks every binding at any depth and must be bounded \
                 by an explicit `include`",
                source.path,
            ),
            _ => {}
        }
    }

    let root_dir = opts.root.clone().unwrap_or_else(|| PathBuf::from("."));

    let explicit_desc: HashMap<&str, &str> = manifest
        .groups
        .iter()
        .filter_map(|g| g.description.as_deref().map(|d| (g.id.as_str(), d)))
        .collect();

    let mut entries: Vec<ExportEntry> = Vec::new();
    let mut header_desc: HashMap<String, Vec<String>> = HashMap::new();
    for source in &manifest.sources {
        let (mut source_entries, header) = collect_source(&root_dir, source);
        if let Some(header) = header {
            for group in &source.groups {
                header_desc
                    .entry(group.clone())
                    .or_default()
                    .push(header.clone());
            }
        }
        entries.append(&mut source_entries);
    }

    // Merge if functions reached by several sources
    entries.sort_by(|a, b| a.id.cmp(&b.id));
    let entries = merge_by_id(entries);

    let referenced: HashSet<&str> = entries
        .iter()
        .flat_map(|e| e.groups.iter().map(String::as_str))
        .collect();
    for group in &manifest.groups {
        if !referenced.contains(group.id.as_str()) {
            eprintln!(
                "warning: manifest group `{}` is declared but never referenced by a source",
                group.id
            );
        }
    }
    let mut group_ids: Vec<String> = referenced.iter().map(|s| s.to_string()).collect();
    group_ids.sort();
    let groups: Vec<GroupOut> = group_ids
        .into_iter()
        .map(|id| {
            let description = resolve_group_desc(&id, &explicit_desc, &header_desc);
            if description.is_none() {
                eprintln!(
                    "warning: group `{id}` has no description (no manifest `groups` entry and no `/**` file header)"
                );
            }
            GroupOut { id, description }
        })
        .collect();

    let export = ExportFile {
        schema_version: EXPORT_SCHEMA_VERSION,
        groups,
        entries,
    };
    serde_json::to_string_pretty(&export).expect("failed to serialize export")
}

/// Merge entries sharing the same `id`
/// - keep the first prose, location, union groups.
/// - later entries are effectively dropped
/// - If description strings differ, emits a warning and keep the first.
/// TODO: Maybe also compare positions? This should be rare in practice
fn merge_by_id(entries: Vec<ExportEntry>) -> Vec<ExportEntry> {
    let mut out: Vec<ExportEntry> = Vec::with_capacity(entries.len());
    for mut entry in entries {
        entry.groups.sort();
        entry.groups.dedup();
        match out.last_mut() {
            Some(last) if last.id == entry.id => {
                if last.description != entry.description {
                    eprintln!(
                        "warning: id `{}` is documented differently in two sources; keeping the first",
                        last.id
                    );
                }
                for group in entry.groups {
                    if !last.groups.contains(&group) {
                        last.groups.push(group);
                    }
                }
                last.groups.sort();
            }
            _ => out.push(entry),
        }
    }
    out
}

/// Resolve group description:
/// 1. manifest entry wins -> return it
/// 2. Look if there are file-headers -> return the first contentfull header
/// In case the are multiple file-headers, emit a warning and return the first one
fn resolve_group_desc(
    id: &str,
    explicit: &HashMap<&str, &str>,
    headers: &HashMap<String, Vec<String>>,
) -> Option<String> {
    if let Some(description) = explicit.get(id) {
        return Some((*description).to_string());
    }
    let candidates = headers.get(id)?;
    let mut distinct: Vec<&String> = Vec::new();
    for candidate in candidates {
        if !distinct.contains(&candidate) {
            distinct.push(candidate);
        }
    }
    if distinct.len() > 1 {
        eprintln!(
            "warning: group `{id}` has conflicting `/**` header descriptions across files; \
             using the first (add a manifest `groups` entry to override)"
        );
    }
    distinct.first().map(|s| (*s).to_string())
}

/// Return the first `/**` header comment
/// uses the same RFC145 ast-rules by design
fn file_header_doc(root: &rnix::Root) -> Option<String> {
    let top_expr = root.syntax().first_child()?;
    retrieve_doc_comment(&top_expr, Some(0))
}

/// Discover entries for one source
fn collect_source(root_dir: &Path, source: &ManifestItem) -> (Vec<ExportEntry>, Option<String>) {
    let file_path = root_dir.join(&source.file);
    let src = fs::read_to_string(&file_path)
        .unwrap_or_else(|e| panic!("could not read source {}: {e}", file_path.display()));
    let nix = rnix::Root::parse(&src)
        .ok()
        .unwrap_or_else(|e| panic!("failed to parse source {}: {e}", file_path.display()));

    let header = file_header_doc(&nix);
    let line_starts = line_starts(&src);

    if source.mode == Mode::File {
        let entry = build_file_entry(&nix, source, &line_starts);
        if entry.is_none() {
            eprintln!(
                "warning: source `{}` uses `mode: \"file\"` but its top-level \
                 expression has no `/**` doc-comment",
                source.path
            );
        }
        return (entry.into_iter().collect(), header);
    }

    let apvs = match source.mode {
        Mode::Flat => collect_flat(&nix),
        Mode::Deep => collect_all(&nix),
        Mode::File => unreachable!("`file` mode handled above"),
    };

    let include = source.include.as_deref();
    let empty: Vec<String> = Vec::new();
    let exclude = source.exclude.as_ref().unwrap_or(&empty);

    let entries = apvs
        .into_iter()
        .filter_map(|apv| {
            let name = local_name(&apv);
            if let Some(include) = include {
                if !include.iter().any(|i| i == &name) {
                    return None;
                }
            }
            if exclude.iter().any(|e| e == &name) {
                return None;
            }
            build_entry(
                &apv,
                &name,
                &source.path,
                &source.file,
                &source.groups,
                &line_starts,
            )
        })
        .collect();
    (entries, header)
}

fn build_entry(
    apv: &AttrpathValue,
    name: &str,
    path: &str,
    file: &str,
    groups: &[String],
    line_starts: &[usize],
) -> Option<ExportEntry> {
    let description = retrieve_doc_comment(apv.syntax(), Some(0))?;

    let name_s = name.to_string();
    let empty = String::new();
    let path_s = path.to_string();
    let attr_path = get_title(&path_s, &empty, &name_s);
    let id = get_identifier(&path_s, &empty, &name_s);

    let offset = usize::from(
        apv.attrpath()
            .expect("AttrpathValue without attrpath")
            .syntax()
            .text_range()
            .start(),
    );
    let (line, column) = offset_to_line_col(offset, line_starts);

    Some(ExportEntry {
        id,
        attr_path,
        name: name_s,
        description,
        groups: groups.to_vec(),
        source: Source {
            file: file.to_string(),
            line,
            column,
        },
    })
}

fn build_file_entry(
    nix: &rnix::Root,
    source: &ManifestItem,
    line_starts: &[usize],
) -> Option<ExportEntry> {
    let top = nix.syntax().first_child()?;
    let description = retrieve_doc_comment(&top, Some(0))?;

    let path_s = source.path.clone();
    let empty = String::new();
    let attr_path = get_title(&path_s, &empty, &empty);
    let id = get_identifier(&path_s, &empty, &empty);
    let name = path_s.rsplit('.').next().unwrap_or(&path_s).to_string();

    let offset = usize::from(top.text_range().start());
    let (line, column) = offset_to_line_col(offset, line_starts);

    Some(ExportEntry {
        id,
        attr_path,
        name,
        description,
        groups: source.groups.clone(),
        source: Source {
            file: source.file.clone(),
            line,
            column,
        },
    })
}

fn local_name(apv: &AttrpathValue) -> String {
    apv.attrpath()
        .expect("AttrpathValue without attrpath")
        .to_string()
}

fn collect_flat(root: &rnix::Root) -> Vec<AttrpathValue> {
    let mut preorder = root.syntax().preorder();
    while let Some(ev) = preorder.next() {
        match ev {
            // Skip function-argument patterns, matching collect_entries.
            WalkEvent::Enter(n) if n.kind() == SyntaxKind::NODE_PATTERN => {
                preorder.skip_subtree();
            }
            WalkEvent::Enter(n) if n.kind() == SyntaxKind::NODE_LET_IN => {
                let body = LetIn::cast(n)
                    .expect("NODE_LET_IN did not cast to LetIn")
                    .body()
                    .expect("let-in without body");
                return direct_children_of_first_attrset(body.syntax());
            }
            WalkEvent::Enter(n) if n.kind() == SyntaxKind::NODE_ATTR_SET => {
                return direct_children_of_first_attrset(&n);
            }
            _ => {}
        }
    }
    Vec::new()
}

/// Return the direct `AttrpathValue` children of the first attrset reachable from `node`.
fn direct_children_of_first_attrset(node: &SyntaxNode) -> Vec<AttrpathValue> {
    node.descendants()
        .find_map(AttrSet::cast)
        .map(|set| set.attrpath_values().collect())
        .unwrap_or_default()
}

/// Deep discovery
fn collect_all(root: &rnix::Root) -> Vec<AttrpathValue> {
    let mut out = Vec::new();
    for ev in root.syntax().preorder() {
        if let WalkEvent::Enter(n) = ev {
            if let Some(apv) = AttrpathValue::cast(n) {
                out.push(apv);
            }
        }
    }
    out
}

fn line_starts(src: &str) -> Vec<usize> {
    let mut starts = vec![0usize];
    for (i, b) in src.bytes().enumerate() {
        if b == b'\n' {
            starts.push(i + 1);
        }
    }
    starts
}

fn offset_to_line_col(offset: usize, line_starts: &[usize]) -> (usize, usize) {
    debug_assert_eq!(line_starts.first(), Some(&0));
    let line = line_starts.partition_point(|&s| s <= offset);
    (line, offset - line_starts[line - 1] + 1)
}
