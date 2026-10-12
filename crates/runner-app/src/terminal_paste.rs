use gpui::{ClipboardEntry, ClipboardItem, Image};
use runner_core::protocol::{ClientError, DaemonClient};
use runner_core::Runtime;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TerminalPaste {
    Image(Image),
    Text(String),
}

pub fn prepare_image_paste(
    client: &DaemonClient,
    image: Image,
    runtime: Option<Runtime>,
) -> Result<&'static [u8], ClientError> {
    #[cfg(target_os = "macos")]
    {
        client.session_paste_image(image.bytes, image.format.mime_type())?;
    }
    #[cfg(not(target_os = "macos"))]
    {
        // Only macOS needs pasteboard restoration. On Windows the agent reads
        // the original clipboard; sending a DIB as JSON can exceed MAX_FRAME.
        let _ = (client, image);
    }
    // Claude Code binds image paste to Alt+V on Windows.
    if cfg!(windows) && runtime == Some(Runtime::ClaudeCode) {
        Ok(b"\x1bv")
    } else {
        Ok(b"\x16")
    }
}

pub fn resolve_terminal_paste(
    item: Option<&ClipboardItem>,
    file_paths: impl FnOnce() -> Vec<String>,
) -> Option<TerminalPaste> {
    if let Some(image) = item.and_then(|item| {
        item.entries().iter().find_map(|entry| match entry {
            ClipboardEntry::Image(image) => Some(image.clone()),
            ClipboardEntry::String(_) | ClipboardEntry::ExternalPaths(_) => None,
        })
    }) {
        return Some(TerminalPaste::Image(image));
    }

    let mut paths = file_paths();
    if paths.is_empty() {
        paths = item
            .into_iter()
            .flat_map(ClipboardItem::entries)
            .filter_map(|entry| match entry {
                ClipboardEntry::ExternalPaths(paths) => Some(paths.paths()),
                ClipboardEntry::String(_) | ClipboardEntry::Image(_) => None,
            })
            .flatten()
            .map(|path| path.to_string_lossy().into_owned())
            .collect();
    }
    if !paths.is_empty() {
        return Some(TerminalPaste::Text(format_pasted_paths(&paths)));
    }

    item.filter(|item| {
        item.entries()
            .iter()
            .any(|entry| matches!(entry, ClipboardEntry::String(_)))
    })
    .and_then(ClipboardItem::text)
    .map(TerminalPaste::Text)
}

pub fn shell_quote_path(path: &str) -> String {
    if !path.chars().any(needs_quoting) {
        return path.to_owned();
    }
    format!("'{}'", path.replace('\'', "'\\''"))
}

pub fn format_pasted_paths(paths: &[String]) -> String {
    paths
        .iter()
        .map(|path| shell_quote_path(path))
        .collect::<Vec<_>>()
        .join(" ")
}

fn needs_quoting(character: char) -> bool {
    character.is_whitespace()
        || matches!(
            character,
            '!' | '"'
                | '#'
                | '$'
                | '&'
                | '\''
                | '('
                | ')'
                | '*'
                | ';'
                | '<'
                | '>'
                | '?'
                | '['
                | '\\'
                | ']'
                | '^'
                | '`'
                | '{'
                | '|'
                | '}'
                | '~'
        )
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use gpui::{ClipboardItem, Image};
    use runner_core::protocol::{EventSubscription, Request, Response, Transport};

    use super::*;

    #[derive(Default)]
    struct ClipboardTransport {
        requests: Mutex<Vec<Request>>,
        error: Option<ClientError>,
    }

    impl Transport for ClipboardTransport {
        fn call(&self, request: Request) -> Result<Response, ClientError> {
            assert!(matches!(request, Request::session_paste_image { .. }));
            self.requests.lock().unwrap().push(request);
            Ok(Response::session_paste_image(match &self.error {
                Some(error) => Err(error.clone()),
                None => Ok(()),
            }))
        }

        fn subscribe(&self) -> Box<dyn EventSubscription> {
            panic!("image paste must not subscribe to daemon events")
        }
    }

    #[test]
    fn image_paste_restores_the_clipboard_only_on_macos() {
        for format in [gpui::ImageFormat::Png, gpui::ImageFormat::Jpeg] {
            let transport = Arc::new(ClipboardTransport::default());
            let client = DaemonClient::new(transport.clone());
            let bytes = vec![1, 2, 3];
            let image = Image::from_bytes(format, bytes.clone());
            assert_eq!(prepare_image_paste(&client, image, None).unwrap(), b"\x16");
            let requests = transport.requests.lock().unwrap();
            if cfg!(target_os = "macos") {
                assert_eq!(requests.len(), 1);
                let Request::session_paste_image {
                    bytes: restored,
                    mime_type,
                } = &requests[0]
                else {
                    unreachable!()
                };
                assert_eq!(restored, &bytes);
                assert_eq!(mime_type, format.mime_type());
            } else {
                assert!(requests.is_empty());
            }
        }
    }

    #[test]
    fn image_paste_propagates_restoration_errors_only_on_macos() {
        let error = ClientError::msg("clipboard restoration failed");
        let client = DaemonClient::new(Arc::new(ClipboardTransport {
            error: Some(error.clone()),
            ..Default::default()
        }));
        let result = prepare_image_paste(&client, Image::empty(), None);
        if cfg!(target_os = "macos") {
            assert_eq!(result, Err(error));
        } else {
            assert_eq!(result, Ok(b"\x16".as_slice()));
        }
    }

    #[cfg(windows)]
    #[test]
    fn windows_large_bitmap_paste_sends_no_image_request() {
        let transport = Arc::new(ClipboardTransport::default());
        let client = DaemonClient::new(transport.clone());
        let image = Image::from_bytes(
            gpui::ImageFormat::Bmp,
            vec![0; runner_core::protocol::wire::MAX_FRAME + 1],
        );
        assert_eq!(
            prepare_image_paste(&client, image, Some(Runtime::ClaudeCode)).unwrap(),
            b"\x1bv"
        );
        assert!(transport.requests.lock().unwrap().is_empty());
    }

    #[test]
    fn image_paste_selects_the_runtime_and_platform_shortcut() {
        let client = DaemonClient::new(Arc::new(ClipboardTransport::default()));
        for runtime in Runtime::ALL {
            let input = prepare_image_paste(&client, Image::empty(), Some(runtime)).unwrap();
            let expected: &[u8] = if cfg!(windows) && runtime == Runtime::ClaudeCode {
                b"\x1bv"
            } else {
                b"\x16"
            };
            assert_eq!(input, expected, "{}", runtime.key());
        }
    }

    #[test]
    fn shell_quote_path_matches_the_react_contract() {
        assert_eq!(shell_quote_path("/tmp/plain.png"), "/tmp/plain.png");
        assert_eq!(
            shell_quote_path("/tmp/two words.png"),
            "'/tmp/two words.png'"
        );
        assert_eq!(shell_quote_path("/tmp/it's.png"), "'/tmp/it'\\''s.png'");

        for character in [
            ' ', '\t', '\n', '!', '"', '#', '$', '&', '\'', '(', ')', '*', ';', '<', '>', '?', '[',
            '\\', ']', '^', '`', '{', '|', '}', '~',
        ] {
            let path = format!("/tmp/a{character}b");
            let quoted = shell_quote_path(&path);
            assert!(quoted.starts_with('\'') && quoted.ends_with('\''));
        }
    }

    #[test]
    fn format_pasted_paths_space_joins_multiple_files() {
        assert_eq!(
            format_pasted_paths(&["/tmp/a.png".into(), "/tmp/two words.jpg".into()]),
            "/tmp/a.png '/tmp/two words.jpg'"
        );
    }

    #[test]
    fn paste_resolution_prefers_image_then_file_urls_then_plain_text() {
        let image = Image::empty();
        assert_eq!(
            resolve_terminal_paste(Some(&ClipboardItem::new_image(&image)), || {
                panic!("file URLs must not be read for image paste")
            }),
            Some(TerminalPaste::Image(image))
        );

        let text = ClipboardItem::new_string("picture.png".into());
        assert_eq!(
            resolve_terminal_paste(Some(&text), || vec!["/tmp/two words/picture.png".into()]),
            Some(TerminalPaste::Text("'/tmp/two words/picture.png'".into()))
        );
        assert_eq!(
            resolve_terminal_paste(Some(&text), Vec::new),
            Some(TerminalPaste::Text("picture.png".into()))
        );
        assert_eq!(
            resolve_terminal_paste(None, || vec!["/tmp/file-only.png".into()]),
            Some(TerminalPaste::Text("/tmp/file-only.png".into()))
        );
        assert_eq!(resolve_terminal_paste(None, Vec::new), None);
    }
}
