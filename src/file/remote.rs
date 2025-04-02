//! Helper methods for loading remote files or repos.

use std::collections::VecDeque;
use std::mem;

use serde::{Deserialize, Serialize};
use serde_json::Number;

use url::Url;

use reqwest::header::{HeaderMap, ACCEPT};
use reqwest::{Response, StatusCode};

use crate::file::suffix::LANGUAGE_MAP;
use crate::file::{CodeGroup, MAX_FILE_SIZE, MAX_NUM_FILES};
use crate::utils::error::CodeImportError;

/// GitHub user-supplied repo URL host string (must match).
const GITHUB_HOST_STR: &str = "github.com";

/// GitHub API request URL prefix.
const GITHUB_API_PREFIX: &str = "https://api.github.com/repos";

/// GitHub raw content URL prefix.
const GITHUB_RAW_PREFIX: &str = "https://raw.githubusercontent.com";

/// GitHub API repo listing inner tree entry struct.
#[derive(Serialize, Deserialize)]
struct GitHubGetTreeEntry {
    #[serde(rename = "type")]
    o_type: String,
    path: String,
    sha: String,
    size: Option<Number>,
}

/// GitHub API repo listing response body.
#[derive(Serialize, Deserialize)]
struct GitHubGetTreeResponse {
    sha: String,
    tree: Vec<GitHubGetTreeEntry>,
}

/// GitHub API repo metadata response body.
#[derive(Serialize, Deserialize)]
struct GitHubRepoMetaResponse {
    name: String,
    default_branch: String,
}

impl CodeGroup {
    /// Parses the file extension from a URL.
    pub(crate) fn get_url_extension(url: &Url) -> Result<&str, CodeImportError> {
        if let Some(segs) = url.path_segments() {
            if let Some(last) = segs.last() {
                if let Some(dot_pos) = last.rfind('.') {
                    let extension = &last[dot_pos..];
                    if extension.is_empty() {
                        Err(CodeImportError::parse("file URL missing file extension"))
                    } else {
                        // good path
                        Ok(extension)
                    }
                } else {
                    Err(CodeImportError::parse("file URL missing file extension"))
                }
            } else {
                Err(CodeImportError::parse("invalid URL path to raw file"))
            }
        } else {
            Err(CodeImportError::parse("invalid URL path to raw file"))
        }
    }

    /// Validate the form of a remote URL. Returns the full path name on success.
    fn validate_file_url(url: &Url) -> Result<&str, CodeImportError> {
        if url.scheme() != "http" && url.scheme() != "https" {
            return Err(CodeImportError::parse(format!(
                "unsupported URL scheme: {}",
                url.scheme()
            )));
        }

        let ext = Self::get_url_extension(url)?;
        if !LANGUAGE_MAP.contains_key(ext) {
            Err(CodeImportError::exten(format!(
                "file extension '{}' is not code",
                ext
            )))
        } else {
            Ok(url.path().trim_matches('/'))
        }
    }

    /// Handle a redirection response, returning the final URL and response on
    /// success.
    async fn handle_redirection(
        &self,
        url: Url,
        response: Response,
    ) -> Result<(Url, Response), CodeImportError> {
        if response.status().is_redirection() {
            if let Some(location) = response.headers().get("location") {
                if let Ok(location_str) = location.to_str() {
                    if let Ok(redirect_url) = Url::parse(location_str) {
                        log::warn!("URL redirecting to '{}'...", redirect_url);
                        let new_resp = self.client.head(redirect_url.as_str()).send().await?;
                        return Ok((redirect_url, new_resp));
                    } else {
                        // handle relative redirects
                        if let Ok(redirect_url) = url.join(location_str) {
                            log::warn!("URL redirecting to '{}'...", redirect_url);
                            let new_resp = self.client.head(redirect_url.as_str()).send().await?;
                            return Ok((redirect_url, new_resp));
                        }
                    }
                }
            }

            Err(CodeImportError::status(
                "got redirection response but bad location",
            ))
        } else {
            // not a redirection, return the original response
            Ok((url, response))
        }
    }

    /// Check if a URL points to a single regular remote file. The URL could be
    /// not pointing to a file; in that case, the function returns `None`.
    /// Otherwise, a tuple of three things is returned: the full path name, a
    /// possibly-updated URL (after redirection), and an approximate size.
    pub(crate) async fn head_single_file(
        &mut self,
        url: Url,
    ) -> Result<Option<(String, Url, usize)>, CodeImportError> {
        let response = self.client.head(url.as_str()).send().await?;
        let (final_url, response) = self.handle_redirection(url, response).await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            return Err(CodeImportError::status(format!(
                "URL check failed with: {}: {}",
                status, text
            )));
        }

        // check for response headers that might indicate a file; Content-Type
        // header should be present for files
        if let Some(content_type) = response.headers().get("content-type") {
            let content_type_str = content_type.to_str().unwrap_or_default();
            if content_type_str.contains("text/html")
                || content_type_str.contains("application/xhtml")
            {
                return Ok(None);
            }

            let mut approx_size = 0;
            if let Some(length) = response.headers().get("content-length") {
                if let Ok(size) = length.to_str().unwrap_or("0").parse::<usize>() {
                    approx_size = size;
                    if size > MAX_FILE_SIZE {
                        self.skipped = true;
                        return Err(CodeImportError::limit(format!(
                            "remote file too large ({}KB >= max {}KB)",
                            size / 1024,
                            MAX_FILE_SIZE / 1024
                        )));
                    }
                }
            }

            // if we got here, it's likely a file
            let path = Self::validate_file_url(&final_url)?;
            return Ok(Some((path.to_string(), final_url, approx_size)));
        }

        // if no content type header, we can't be sure - assume it's not a file
        Ok(None)
    }

    /// Parse a user-supplied GitHub repo URL into owner, repo, and tree.
    async fn dissect_github_url(
        &self,
        url: &Url,
    ) -> Result<(String, String, String), CodeImportError> {
        if let Some(segs) = url.path_segments() {
            let segs = segs
                .filter(|s| !s.is_empty())
                .take(4)
                .collect::<Vec<&str>>();
            if segs.len() < 2 {
                return Err(CodeImportError::github(
                    "repo URL must contain owner and repo name",
                ));
            } else if ((segs.len() > 2 && segs[2] != "tree") || segs.len() > 4)
                && !segs.last().unwrap().is_empty()
            {
                return Err(CodeImportError::github(
                    "repo URL should not carry path to specific file",
                ));
            }

            let owner = segs[0].to_string();
            let repo = segs[1].to_string();

            // extract tree (branch/tag)
            let tree = if segs.len() >= 4 && segs[2] == "tree" {
                segs[3].to_string()
            } else {
                // not present, query repo metadata for default branch
                let mut headers = HeaderMap::new();
                headers.insert(ACCEPT, "application/vnd.github+json".parse()?);
                headers.insert("X-GitHub-Api-Version", "2022-11-28".parse()?);
                let response = self
                    .client
                    .get(format!("{}/{}/{}", GITHUB_API_PREFIX, owner, repo))
                    .headers(headers)
                    .send()
                    .await?;

                if response.status() == StatusCode::FORBIDDEN {
                    // probably getting rate limited by GitHub
                    return Err(CodeImportError::github(format!(
                        "repo metadata query failed with: {}, rate limited?",
                        response.status()
                    )));
                } else if !response.status().is_success() {
                    return Err(CodeImportError::github(format!(
                        "repo metadata query failed with: {}",
                        response.status()
                    )));
                }
                let resp = response.json::<GitHubRepoMetaResponse>().await?;
                resp.default_branch
            };

            Ok((owner, repo, tree))
        } else {
            Err(CodeImportError::parse("invalid URL path to GitHub repo"))
        }
    }

    /// BFS traverse the repo tree starting from root, gathering files into the
    /// info_list result.
    async fn bfs_traverse_tree(
        &mut self,
        bfs_queue: &mut VecDeque<(String, String)>,
        path_info_list: &mut Vec<(String, (Url, usize))>,
        owner: &str,
        repo: &str,
    ) -> Result<(), CodeImportError> {
        // record repo tree root SHA1 value to make composing raw content
        // URLs easier
        let mut root_sha = String::new();

        while let Some((path, tree)) = bfs_queue.pop_front() {
            // make a "Get a tree" API request
            let mut headers = HeaderMap::new();
            headers.insert(ACCEPT, "application/vnd.github+json".parse()?);
            headers.insert("X-GitHub-Api-Version", "2022-11-28".parse()?);
            let response = self
                .client
                .get(format!(
                    "{}/{}/{}/git/trees/{}",
                    GITHUB_API_PREFIX, owner, repo, tree
                ))
                .headers(headers)
                .send()
                .await?;

            if response.status() == StatusCode::FORBIDDEN {
                // probably getting rate limited by GitHub
                return Err(CodeImportError::github(format!(
                    "repo URL listing failed with: {}, rate limited?",
                    response.status()
                )));
            } else if !response.status().is_success() {
                return Err(CodeImportError::github(format!(
                    "repo URL listing failed with: {}",
                    response.status()
                )));
            }
            let mut resp = response.json::<GitHubGetTreeResponse>().await?;

            // record root SHA1 if at root
            if root_sha.is_empty() {
                assert!(path.is_empty());
                root_sha = mem::take(&mut resp.sha);
            }

            let full_path = |p: &str, pc: &str| {
                if p.is_empty() {
                    pc.to_string()
                } else {
                    format!("{}/{}", p, pc)
                }
            };

            // loop through all entries of the tree
            for entry in resp.tree {
                match entry.o_type.as_str() {
                    "blob" => {
                        // regular file, add if is a code file
                        if let Some(dot_pos) = entry.path.rfind('.') {
                            let extension = &entry.path[dot_pos..];
                            if !extension.is_empty() && LANGUAGE_MAP.contains_key(extension) {
                                let this_path =
                                    format!("{}/{}", repo, full_path(&path, &entry.path));
                                let raw_url = Url::parse(
                                    format!(
                                        "{}/{}/{}/{}/{}",
                                        GITHUB_RAW_PREFIX,
                                        owner,
                                        repo,
                                        root_sha,
                                        full_path(&path, &entry.path)
                                    )
                                    .as_str(),
                                )?;

                                let approx_size =
                                    entry.size.map(|s| s.as_u64().unwrap_or(0)).unwrap_or(0)
                                        as usize; // 0 means unclear size
                                if approx_size > MAX_FILE_SIZE {
                                    self.skipped = true;
                                    continue; // skip too-large file
                                }

                                path_info_list.push((this_path, (raw_url, approx_size)));

                                if path_info_list.len() >= MAX_NUM_FILES {
                                    return Ok(());
                                }
                            }
                        }
                    }

                    "tree" => {
                        // subdirectory, add to BFS queue
                        bfs_queue.push_back((full_path(&path, &entry.path), entry.sha));
                    }

                    _ => {} // submodules ignored
                }
            }
        }

        Ok(())
    }

    /// Try to treat URL as a GitHub repo and list its files sequentially, taking
    /// at most MAX_NUM_FILES and skipping any file larger than MAX_FILE_SIZE. If
    /// the link does not seem to be a GitHub repo, return None.
    pub(crate) async fn list_github_repo(
        &mut self,
        url: &Url,
    ) -> Result<Option<Vec<(String, (Url, usize))>>, CodeImportError> {
        if url.scheme() != "http" && url.scheme() != "https" {
            return Err(CodeImportError::parse(format!(
                "unsupported URL scheme: {}",
                url.scheme()
            )));
        } else if url.host_str() != Some(GITHUB_HOST_STR) {
            // not a GitHub repo URL
            return Ok(None);
        }

        let (owner, repo, tree) = self.dissect_github_url(url).await?;

        // BFS traversal of the repo tree
        let mut path_info_list = Vec::new();
        let mut bfs_queue = VecDeque::new();
        bfs_queue.push_back(("".to_string(), tree));
        self.bfs_traverse_tree(&mut bfs_queue, &mut path_info_list, &owner, &repo)
            .await?;

        if path_info_list.is_empty() {
            Err(CodeImportError::github(format!(
                "repo '{}' does not contain any code files",
                repo
            )))
        } else {
            Ok(Some(path_info_list))
        }
    }
}
