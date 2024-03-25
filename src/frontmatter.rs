use std::{
    fs,
    path::{Path, PathBuf},
};

use gray_matter::engine::YAML;
use gray_matter::Matter;
use std::fmt;

use relative_path::RelativePath;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Frontmatter {
    pub doc_location: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum FrontmatterErrorKind {
    InvalidYaml,
    DocLocationFileNotFound,
    DocLocationConflictWithContent,
    DocLocationNotRelativePath,
}

#[derive(Debug, PartialEq)]
pub struct FrontmatterError {
    pub message: String,
    pub kind: FrontmatterErrorKind,
}

impl fmt::Display for FrontmatterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write the error message to the formatter
        write!(f, "FrontmatterError: {}", self.message)
    }
}

/// Returns the actual content of a markdown file, if the frontmatter has a doc_location field.
/// It returns None if the frontmatter is not present.
/// It returns an error if the frontmatter is present but invalid. This includes:
///     - Invalid yaml frontmatter
///     - Invalid doc_location type
///     - doc_location file is not readable or not found
///     - doc_location field is not a relative path
///     - doc_location file is not utf8
pub fn get_imported_content(
    file_path: &Path,
    markdown: &str,
) -> Result<Option<String>, FrontmatterError> {
    let matter = Matter::<YAML>::new();

    let result = matter.parse(markdown);

    // If the frontmatter is not present, we return None
    if result.data.is_none() {
        return Ok(None);
    }

    let pod = result.data.unwrap();
    let has_content = !result.content.trim().is_empty();
    match pod.deserialize::<Frontmatter>() {
        Ok(metadata) => {
            let abs_import = match metadata.doc_location {
                Some(doc_location) => {
                    if has_content {
                        return Err(FrontmatterError {
                            message: format!(
                                "{:?}: doc_location: if this field is specified no other content is allowed for the doc-comment.",
                                file_path
                            ),
                            kind: FrontmatterErrorKind::DocLocationConflictWithContent,
                        });
                    }

                    let import_path: PathBuf = PathBuf::from(&doc_location);
                    let relative_path = RelativePath::from_path(&import_path);

                    match relative_path {
                        Ok(rel) => Ok(Some(rel.to_path(file_path.parent().unwrap()))),
                        Err(e) => Err(FrontmatterError {
                            message: format!("{:?}: doc_location: field must be a path relative to the current file. Error: {} - {}", file_path, doc_location, e),
                            kind: FrontmatterErrorKind::DocLocationNotRelativePath,
                        }),
                    }
                }
                // doc_location: field doesn't exist. Since it is optional, we return None
                None => Ok(None),
            };

            match abs_import {
                Ok(Some(path)) => match fs::read_to_string(&path) {
                    Ok(content) => Ok(Some(content)),
                    Err(e) => Err(FrontmatterError {
                        message: format!(
                            "{:?}: Failed to read doc_location file: {:?} {}",
                            file_path, path, e
                        ),
                        kind: FrontmatterErrorKind::DocLocationFileNotFound,
                    }),
                },
                Ok(None) => Ok(None),
                Err(e) => Err(e),
            }
        }

        Err(e) => {
            let message = format!(
                "{:?}: Failed to parse frontmatter metadata - {} YAML:{}:{}",
                file_path,
                e,
                e.line(),
                e.column()
            );
            Err(FrontmatterError {
                message,
                kind: FrontmatterErrorKind::InvalidYaml,
            })
        }
    }
}
