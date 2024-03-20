use std::{collections::HashMap, fs, path::PathBuf, process::Command};

use serde::{Deserialize, Serialize};
use yaml_front_matter::{Document, YamlFrontMatter};

fn find_repo_root() -> Option<PathBuf> {
    let output = Command::new("git")
        .args(&["rev-parse", "--show-toplevel"])
        .output()
        .ok()?
        .stdout;

    let path_str = String::from_utf8(output).ok()?.trim().to_string();
    Some(PathBuf::from(path_str))
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Matter {
    #[serde(flatten)]
    pub content: HashMap<String, serde_yaml::Value>,
}

/// Returns the actual content of a markdown file, if the frontmatter has an import field.
pub fn get_imported_content(file_path: &PathBuf, markdown: Option<&String>) -> Option<String> {
    if markdown.is_none() {
        return None;
    }

    match YamlFrontMatter::parse::<Matter>(&markdown.unwrap()) {
        Ok(document) => {
            let metadata = document.metadata.content;

            let abs_import = metadata.get("import").map(|field| {
                let import_val = field
                    .as_str()
                    .expect("Frontmatter: import field must be a string");
                match PathBuf::from(import_val).is_relative() {
                    true => PathBuf::from_iter(vec![
                        // Cannot fail because every file has a parent directory
                        file_path.parent().unwrap().to_path_buf(),
                        PathBuf::from(import_val),
                    ]),
                    false => PathBuf::from_iter(vec![
                        find_repo_root()
                        .expect("Could not find root directory of repository. Make sure you have git installed and are in a git repository"),
                        PathBuf::from(format!(".{import_val}")),
                    ]),
                }
            });

            abs_import.map(|path| {
                fs::read_to_string(&path)
                    .expect(format!("Could not read file: {:?}", &path).as_str())
            })
        }
        Err(e) => {
            return None;
        }
    }
}
