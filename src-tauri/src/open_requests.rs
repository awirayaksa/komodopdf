//! OS file-open requests survive startup until the webview subscribes and drains them.
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Default)]
pub struct OpenRequests(Mutex<VecDeque<String>>);

impl OpenRequests {
    pub fn push(&self, paths: impl IntoIterator<Item = String>) {
        self.0.lock().unwrap().extend(paths);
    }

    pub fn drain(&self) -> Vec<String> {
        self.0.lock().unwrap().drain(..).collect()
    }
}

/// `args` includes the executable, both for std::env and single-instance IPC.
/// Never split on spaces or decode percent escapes in a native filesystem path.
pub fn pdf_paths(args: impl IntoIterator<Item = String>, cwd: &Path) -> Vec<PathBuf> {
    args.into_iter()
        .skip(1)
        .filter_map(|arg| {
            let arg = arg
                .strip_prefix('"')
                .and_then(|s| s.strip_suffix('"'))
                .unwrap_or(&arg);
            if arg.is_empty() || arg.starts_with('-') {
                return None;
            }
            // A drive letter is a path, not a URL scheme.
            let drive =
                arg.as_bytes().get(1) == Some(&b':') && arg.as_bytes()[0].is_ascii_alphabetic();
            let path = if !drive {
                match url::Url::parse(arg) {
                    Ok(url) => {
                        if url.scheme() != "file"
                            || url.query().is_some()
                            || url.fragment().is_some()
                        {
                            return None;
                        }
                        url.to_file_path().ok()?
                    }
                    Err(_) => PathBuf::from(arg),
                }
            } else {
                PathBuf::from(arg)
            };
            if !path.extension()?.eq_ignore_ascii_case("pdf") {
                return None;
            }
            Some(if path.is_absolute() {
                path
            } else {
                cwd.join(path)
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(args: &[&str], cwd: &Path) -> Vec<PathBuf> {
        pdf_paths(args.iter().map(|s| s.to_string()), cwd)
    }

    #[test]
    fn native_paths_keep_spaces_unicode_and_url_punctuation() {
        let cwd = std::env::temp_dir();
        assert_eq!(
            paths(&["Komodo.exe", "日本語 資料.PDF", "100%20 #1.pdf"], &cwd),
            vec![cwd.join("日本語 資料.PDF"), cwd.join("100%20 #1.pdf")]
        );
        assert_eq!(
            paths(&["executable.pdf", "\"quoted file.pdf\""], &cwd),
            vec![cwd.join("quoted file.pdf")]
        );
    }

    #[test]
    fn file_urls_are_decoded_once_and_network_urls_are_ignored() {
        let path = std::env::temp_dir().join("日本語 100%20 #1.pdf");
        let url = url::Url::from_file_path(&path).unwrap();
        assert_eq!(
            paths(&["app", url.as_str()], &std::env::temp_dir()),
            vec![path]
        );
        assert!(paths(
            &[
                "app",
                "https://example.com/a.pdf",
                "--help",
                "note.txt",
                "file:///a.pdf?query",
                "file:///a.pdf#page=1"
            ],
            &std::env::temp_dir()
        )
        .is_empty());
    }

    #[test]
    fn queued_cold_start_and_second_launch_are_consumed_in_order_once() {
        let requests = OpenRequests::default();
        requests.push(["cold.pdf".into()]);
        requests.push(["second.pdf".into(), "third.pdf".into()]);
        assert_eq!(
            requests.drain(),
            vec!["cold.pdf", "second.pdf", "third.pdf"]
        );
        assert!(requests.drain().is_empty());
        requests.push(["later.pdf".into()]);
        assert_eq!(requests.drain(), vec!["later.pdf"]);
    }

    #[cfg(windows)]
    #[test]
    fn windows_drive_unc_and_secondary_working_directory() {
        let cwd = Path::new(r"D:\別の フォルダ");
        assert_eq!(
            paths(
                &[
                    "app.exe",
                    r"C:\Users\日本語\a b.pdf",
                    r"\\server\share\a b.pdf",
                    "relative.pdf",
                    "file:///C:/Users/%E6%97%A5%E6%9C%AC%E8%AA%9E/a%20b.pdf",
                    "file://server/share/a%20b.pdf"
                ],
                cwd
            ),
            vec![
                PathBuf::from(r"C:\Users\日本語\a b.pdf"),
                PathBuf::from(r"\\server\share\a b.pdf"),
                cwd.join("relative.pdf"),
                PathBuf::from(r"C:\Users\日本語\a b.pdf"),
                PathBuf::from(r"\\server\share\a b.pdf")
            ]
        );
    }
}
