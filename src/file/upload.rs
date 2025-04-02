//! Helper methods for uploading local files or archive.

use std::io::{Cursor, Read, Seek, SeekFrom};
use std::ops::Deref;

use gloo_file::futures::{read_as_bytes, read_as_text};
use gloo_file::{File, FileList};

use zip::ZipArchive;

use flate2::read::GzDecoder;
use tar::Archive as TarArchive;

use sevenz_rust::{Error as SevenZError, Password, SevenZReader};

use crate::file::suffix::LANGUAGE_MAP;
use crate::file::{CodeGroup, MAX_FILE_SIZE, MAX_NUM_FILES};
use crate::utils::error::CodeImportError;

impl CodeGroup {
    /// Extract all valid code files from the given file list.
    pub(crate) async fn list_upload_files(
        &mut self,
        files: FileList,
    ) -> Result<Option<Vec<(String, (String, String))>>, CodeImportError> {
        let mut name_data_list = Vec::new();

        for file in files.deref() {
            let name = file.name();
            if name.is_empty() {
                return Err(CodeImportError::parse("encountered empty file name"));
            }

            if let Some(dot_pos) = name.rfind('.') {
                let extension = &name[dot_pos..];
                if !extension.is_empty() && LANGUAGE_MAP.contains_key(extension) {
                    if (file.size() as usize) > MAX_FILE_SIZE {
                        self.skipped = true;
                        continue;
                    }

                    let ext = extension.to_string();
                    name_data_list.push((name, (ext, read_as_text(file.deref()).await?)));

                    if name_data_list.len() >= MAX_NUM_FILES {
                        break;
                    }
                }
            }
        }

        if name_data_list.is_empty() {
            Err(CodeImportError::upload(
                "uploaded files do not contain any code files",
            ))
        } else {
            Ok(Some(name_data_list))
        }
    }

    /// Extract code files from a zip archive.
    async fn extract_zip(
        &mut self,
        archive: impl Read + Seek,
    ) -> Result<Vec<(String, (String, String))>, CodeImportError> {
        let mut name_data_list = Vec::new();

        let mut archive = ZipArchive::new(archive)
            .map_err(|_| CodeImportError::upload("failed to read uploaded zip archive"))?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|_| {
                CodeImportError::upload(format!("failed to read file at zip index {}", i))
            })?;

            if file.is_file() {
                let name = file.name();
                if let Some(dot_pos) = name.rfind('.') {
                    let extension = &name[dot_pos..];
                    if !extension.is_empty() && LANGUAGE_MAP.contains_key(extension) {
                        if (file.size() as usize) > MAX_FILE_SIZE {
                            self.skipped = true;
                            continue;
                        }

                        let name = name.to_string();
                        let ext = extension.to_string();
                        let mut content = String::new();
                        file.read_to_string(&mut content)?;
                        name_data_list.push((name, (ext, content)));

                        if name_data_list.len() >= MAX_NUM_FILES {
                            break;
                        }
                    }
                }
            }
        }

        Ok(name_data_list)
    }

    /// Extract code files from a tar archive.
    async fn extract_tar(
        &mut self,
        archive: impl Read,
    ) -> Result<Vec<(String, (String, String))>, CodeImportError> {
        let mut name_data_list = Vec::new();

        let mut archive = TarArchive::new(archive);
        for entry in archive
            .entries()
            .map_err(|_| CodeImportError::upload("failed to read uploaded tar archive"))?
        {
            let mut file = entry
                .map_err(|_| CodeImportError::upload("failed to read entry from tar archive"))?;

            if file.header().entry_type().is_file() {
                let name = file.path().map_err(|_| {
                    CodeImportError::upload("failed to get file path from tar archive")
                })?;
                let name = name.to_string_lossy().to_string();
                if let Some(dot_pos) = name.rfind('.') {
                    let extension = &name[dot_pos..];
                    if !extension.is_empty() && LANGUAGE_MAP.contains_key(extension) {
                        if (file.size() as usize) > MAX_FILE_SIZE {
                            self.skipped = true;
                            continue;
                        }

                        let ext = extension.to_string();
                        let mut content = String::new();
                        file.read_to_string(&mut content)?;
                        name_data_list.push((name, (ext, content)));

                        if name_data_list.len() >= MAX_NUM_FILES {
                            break;
                        }
                    }
                }
            }
        }

        Ok(name_data_list)
    }

    /// Extract code files from a 7z archive.
    async fn extract_7z(
        &mut self,
        mut archive: impl Read + Seek,
    ) -> Result<Vec<(String, (String, String))>, CodeImportError> {
        let mut name_data_list = Vec::new();

        let start_pos = archive.stream_position()?;
        let reader_len = archive.seek(SeekFrom::End(0))?;
        archive.seek(SeekFrom::Start(start_pos))?;

        let mut archive = SevenZReader::new(&mut archive, reader_len, Password::empty())
            .map_err(|_| CodeImportError::upload("failed to read uploaded 7z archive"))?;
        archive
            .for_each_entries(|entry, reader| {
                // cannot skip any entry in case the solid archive option is on
                let mut content = Vec::new();
                reader.read_to_end(&mut content)?;

                if !entry.is_anti_item && !entry.is_directory {
                    let name = &entry.name;
                    if let Some(dot_pos) = name.rfind('.') {
                        let extension = &name[dot_pos..];
                        if !extension.is_empty() && LANGUAGE_MAP.contains_key(extension) {
                            if (entry.size as usize) > MAX_FILE_SIZE {
                                self.skipped = true;
                                return Ok(true); // continue
                            }

                            let ext = extension.to_string();
                            name_data_list.push((
                                name.to_string(),
                                (
                                    ext,
                                    String::from_utf8(content)
                                        .map_err(|err| SevenZError::other(err.to_string()))?,
                                ),
                            ));

                            if name_data_list.len() >= MAX_NUM_FILES {
                                return Ok(false); // early break
                            }
                        }
                    }
                }
                Ok(true) // continue
            })
            .map_err(|_| {
                CodeImportError::upload("failed to decode from 7z archive, password issue?")
            })?;

        Ok(name_data_list)
    }

    /// Try to treat the input file as an archive and extract valid code files
    /// from it.
    pub(crate) async fn extract_archive(
        &mut self,
        file: &File,
    ) -> Result<Option<Vec<(String, (String, String))>>, CodeImportError> {
        let name = file.name();
        if name.is_empty() {
            return Err(CodeImportError::parse("encountered empty file name"));
        }

        if let Some(dot_pos) = name.rfind('.') {
            let name_data_list = match &name[dot_pos..] {
                ".zip" => {
                    self.extract_zip(Cursor::new(read_as_bytes(file.deref()).await?))
                        .await?
                }
                ".tar" => {
                    self.extract_tar(Cursor::new(read_as_bytes(file.deref()).await?))
                        .await?
                }
                ".gz" | ".tgz" => {
                    self.extract_tar(GzDecoder::new(Cursor::new(
                        read_as_bytes(file.deref()).await?,
                    )))
                    .await?
                }
                ".7z" => {
                    self.extract_7z(Cursor::new(read_as_bytes(file.deref()).await?))
                        .await?
                }
                _ => {
                    // unsupported archive type
                    return Ok(None);
                }
            };

            if name_data_list.is_empty() {
                Err(CodeImportError::upload(
                    "uploaded archive do not contain any code files",
                ))
            } else {
                Ok(Some(name_data_list))
            }
        } else {
            Ok(None)
        }
    }
}
