//! Code file (or collection of files) import driver.

use std::collections::{hash_map, HashMap};

use url::Url;

use reqwest::Client;

use crate::utils::error::CodeImportError;

mod local;
mod remote;
mod suffix;

// Hardcoded limits on the scale of imported files.
pub(crate) const MAX_NUM_FILES: usize = 100;
pub(crate) const MAX_FILE_SIZE: usize = 100 * 1024; // 100KB

// Display cut-off lengths in table.
const PATH_LENGTH_CUTOFF: usize = 47;
const LANG_LENGTH_CUTOFF: usize = 14;

/// Handle to a single code file.
pub(crate) enum CodeFile {
    /// Content of a local file.
    Local { ext: String, content: String },
    /// URL to a raw file.
    Remote { url: Url, approx_size: usize },
}

impl CodeFile {
    fn new_local(ext: String, content: String) -> Self {
        CodeFile::Local { ext, content }
    }

    fn new_remote(url: Url, approx_size: usize) -> Self {
        CodeFile::Remote { url, approx_size }
    }

    /// Returns the (approximate) size in bytes of the file.
    pub(crate) fn get_size(&self) -> Option<usize> {
        match self {
            CodeFile::Local { content, .. } => Some(content.len()),
            CodeFile::Remote { approx_size, .. } => {
                if *approx_size == 0 {
                    None
                } else {
                    Some(*approx_size)
                }
            }
        }
    }

    /// Returns the file extension.
    pub(crate) fn get_ext(&self) -> Option<&str> {
        match self {
            CodeFile::Local { ext, .. } => Some(ext),
            CodeFile::Remote { url, .. } => CodeGroup::get_url_extension(url).ok(),
        }
    }
}

/// Code import driver.
pub(crate) struct CodeGroup {
    lang_map: suffix::LanguageMap,
    client: Client,

    files: HashMap<String, CodeFile>,
    skipped: bool,
}

impl CodeGroup {
    /// Creates an empty code importer with no files added yet.
    pub(crate) fn new() -> Self {
        CodeGroup {
            lang_map: suffix::LanguageMap::new(),
            files: HashMap::new(),
            client: Client::new(),
            skipped: false,
        }
    }

    /// Returns the display string of a path (which applies a cut-off if it's
    /// too long).
    pub(crate) fn path_display(&self, path: &str) -> String {
        if path.len() > PATH_LENGTH_CUTOFF {
            format!("...{}", &path[(path.len() - PATH_LENGTH_CUTOFF)..])
        } else {
            path.to_string()
        }
    }

    /// Returns the language name of a file extension.
    pub(crate) fn lang_name_of(&self, ext: Option<&str>) -> String {
        if let Some(ext) = ext {
            let lang = self.lang_map.query(ext).unwrap_or("-");
            if lang.len() > LANG_LENGTH_CUTOFF {
                format!("{}...", &lang[..LANG_LENGTH_CUTOFF])
            } else {
                lang.to_string()
            }
        } else {
            "-".to_string()
        }
    }

    /// Get the number of files.
    #[inline]
    pub(crate) fn num_files(&self) -> usize {
        self.files.len()
    }

    /// Get the boolean of whether any file had been skipped.
    #[inline]
    pub(crate) fn skipped(&self) -> bool {
        self.skipped
    }

    /// Get the approximate total size in bytes of imported files.
    pub(crate) fn total_size(&self) -> Option<usize> {
        self.files
            .values()
            .map(|file| file.get_size())
            .sum::<Option<usize>>()
    }

    /// Reset and clear the imported files.
    pub(crate) fn reset(&mut self) {
        self.files.clear();
        self.skipped = false;
    }

    /// Return an iterator of the imported files.
    pub(crate) fn files(&self) -> impl Iterator<Item = (&String, &CodeFile)> {
        self.files.iter()
    }

    /// Adds a remote file or a repo of files to the importer.
    pub(crate) async fn add_remote(&mut self, url_str: &str) -> Result<(), CodeImportError> {
        let url = Url::parse(url_str)?;

        // first try as URL to github repo
        if let Some(path_info_list) = self.list_github_repo(&url).await? {
            for (path, (file_url, approx_size)) in path_info_list {
                self.add_file(path, CodeFile::new_remote(file_url, approx_size))?;
            }
            return Ok(());
        }

        // then try as URL to a single raw file
        if let Some((path, final_url, approx_size)) = self.head_single_file(url).await? {
            self.add_file(path, CodeFile::new_remote(final_url, approx_size))?;
            return Ok(());
        }

        Err(CodeImportError::parse(
            "URL not pointing to raw file or GitHub repo",
        ))
    }

    /// Helper method to add a file to the importer.
    fn add_file(&mut self, name: String, file: CodeFile) -> Result<(), CodeImportError> {
        match self.files.entry(name) {
            hash_map::Entry::Occupied(e) => {
                return Err(CodeImportError::exists(format!(
                    "file name '{}' already exists",
                    e.key()
                )));
            }
            hash_map::Entry::Vacant(e) => {
                e.insert(file);
            }
        }
        Ok(())
    }
}
