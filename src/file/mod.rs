//! Code file (or collection of files) import driver.

use std::borrow::Cow;
use std::collections::{hash_map, HashMap};

use leptos::prelude::*;

use gloo_file::FileList;

use url::{ParseError, Url};

use reqwest::Client;

use crate::utils::error::CodeImportError;

mod remote;
mod upload;

mod suffix;

// Hardcoded limits on the scale of imported files.
pub(crate) const MAX_NUM_FILES: usize = 100;
pub(crate) const MAX_FILE_SIZE: usize = 100 * 1024; // 100KB

// Display cut-off lengths in table.
const PATH_LENGTH_CUTOFF: usize = 36;
const LANG_LENGTH_CUTOFF: usize = 10;

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

    /// Returns the display string of a path (which applies a cut-off if it's
    /// too long).
    pub(crate) fn path_display(path: &str) -> String {
        if path.len() > PATH_LENGTH_CUTOFF {
            format!("...{}", &path[(path.len() - PATH_LENGTH_CUTOFF)..])
        } else {
            path.to_string()
        }
    }

    /// Returns the language name of a file extension.
    pub(crate) fn lang_name_of(ext: Option<&str>) -> String {
        if let Some(ext) = ext {
            let lang = suffix::LANGUAGE_MAP.get(ext).copied().unwrap_or("-");
            if lang.len() > LANG_LENGTH_CUTOFF {
                format!("{}...", &lang[..LANG_LENGTH_CUTOFF])
            } else {
                lang.to_string()
            }
        } else {
            "-".to_string()
        }
    }

    /// Fetches the actual content of the text file, making web requests if necessary.
    pub(crate) async fn content(&self, client: &Client) -> Result<Cow<String>, CodeImportError> {
        match self {
            CodeFile::Local { content, .. } => Ok(Cow::Borrowed(content)),

            CodeFile::Remote { url, .. } => {
                let resp = client.get(url.clone()).send().await?;

                if resp.status().is_success() {
                    let text = resp.text().await?;
                    Ok(Cow::Owned(text))
                } else {
                    // probably network error or authorization failure
                    let status = resp.status();
                    let text = resp.text().await?;
                    Err(CodeImportError::status(format!(
                        "file content fetch failed with {}: {}",
                        status, text,
                    )))
                }
            }
        }
    }
}

/// Code import driver.
pub(crate) struct CodeGroup {
    files: HashMap<String, RwSignal<CodeFile>>,
    skipped: bool,
}

impl CodeGroup {
    /// Creates an empty code importer with no files added yet.
    pub(crate) fn new() -> Self {
        CodeGroup {
            files: HashMap::new(),
            skipped: false,
        }
    }

    /// Get the number of files.
    #[inline]
    pub(crate) fn num_files(&self) -> usize {
        self.files.len()
    }

    /// Get the boolean of whether any file had been skipped.
    #[inline]
    pub(crate) fn has_skipped(&self) -> bool {
        self.skipped
    }

    /// Get the approximate total size in bytes of imported files.
    pub(crate) fn total_size(&self) -> Option<usize> {
        self.files
            .values()
            .map(|file| file.read().get_size())
            .sum::<Option<usize>>()
    }

    /// Reset and clear the imported files.
    pub(crate) fn reset(&mut self) {
        self.files.clear();
        self.skipped = false;
    }

    /// Return a sorted, owning collection of the imported files.
    pub(crate) fn sorted_files(&self) -> Vec<(String, RwSignal<CodeFile>)> {
        let mut files: Vec<_> = self
            .files
            .iter()
            .map(|(path, &file)| (path.clone(), file))
            .collect();
        files.sort_by(|(pa, _), (pb, _)| pa.cmp(pb));
        files
    }

    /// Populates the importer with a remote file or a repo of files.
    pub(crate) async fn import_remote(
        &mut self,
        client: RwSignal<Client>,
        url_str: &str,
    ) -> Result<(), CodeImportError> {
        let url = match Url::parse(url_str) {
            Ok(url) => url,
            Err(ParseError::RelativeUrlWithoutBase) => {
                // default to prepending a 'https://' scheme
                Url::parse(format!("https://{}", url_str).as_str())?
            }
            Err(err) => {
                return Err(err.into());
            }
        };

        // first try as URL to github repo
        if let Some(path_info_list) = self.list_github_repo(client, &url).await? {
            for (path, (file_url, approx_size)) in path_info_list {
                self.add_file(path, CodeFile::new_remote(file_url, approx_size))?;
            }
            return Ok(());
        }

        // then try as URL to a single raw file
        if let Some((path, final_url, approx_size)) = self.head_single_file(client, url).await? {
            self.add_file(path, CodeFile::new_remote(final_url, approx_size))?;
            return Ok(());
        }

        Err(CodeImportError::parse(
            "URL not pointing to raw file or GitHub repo",
        ))
    }

    /// Populates the importer with a plain textbox content.
    pub(crate) async fn import_textbox(&mut self, content: String) -> Result<(), CodeImportError> {
        self.add_file(
            "code from the textbox".to_string(),
            CodeFile::new_local("textbox".to_string(), content),
        )?;

        Ok(())
    }

    /// Populates the importer with an uploaded file list.
    pub(crate) async fn import_upload(&mut self, files: FileList) -> Result<(), CodeImportError> {
        // first try as a single archive file
        if files.len() == 1 {
            if let Some(name_data_list) = self.extract_archive(&files[0]).await? {
                for (name, (ext, content)) in name_data_list {
                    self.add_file(name.clone(), CodeFile::new_local(ext, content))?;
                }
                return Ok(());
            }
        }

        // then try as a list of files, only considering valid code files within
        if let Some(name_data_list) = self.list_upload_files(files).await? {
            for (name, (ext, content)) in name_data_list {
                self.add_file(name.clone(), CodeFile::new_local(ext, content))?;
            }
            return Ok(());
        }

        Err(CodeImportError::upload(
            "uploaded files do not contain any code files",
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
                e.insert(RwSignal::new(file));
            }
        }
        Ok(())
    }
}
