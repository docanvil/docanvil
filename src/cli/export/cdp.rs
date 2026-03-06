use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::process::{Child, ChildStderr, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use base64::Engine;
use serde_json::{Value, json};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message, WebSocket};

use crate::error::{Error, Result};

// ── CdpSession ────────────────────────────────────────────────────────────────

/// A synchronous Chrome DevTools Protocol session over a WebSocket.
pub struct CdpSession {
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
    next_id: u32,
    /// Events received while waiting for a command response, buffered for later consumption.
    event_buffer: Vec<Value>,
}

impl CdpSession {
    /// Connect to a Chrome DevTools WebSocket endpoint.
    pub fn connect(ws_url: &str) -> Result<Self> {
        let (mut socket, _) = tungstenite::connect(ws_url)
            .map_err(|e| Error::General(format!("failed to connect to Chrome CDP: {e}")))?;

        // Set a short read timeout so our message-polling loops don't block forever.
        if let MaybeTlsStream::Plain(tcp) = socket.get_mut() {
            tcp.set_read_timeout(Some(Duration::from_millis(200)))
                .map_err(|e| Error::General(format!("failed to set CDP read timeout: {e}")))?;
        }

        Ok(Self {
            socket,
            next_id: 1,
            event_buffer: Vec::new(),
        })
    }

    /// Send a CDP command and wait for the matching response.
    ///
    /// Any events received en route are pushed onto `event_buffer`.
    pub fn send_command(&mut self, method: &str, params: Value) -> Result<Value> {
        let id = self.next_id;
        self.next_id += 1;

        let cmd = json!({ "id": id, "method": method, "params": params });
        self.socket
            .send(Message::Text(cmd.to_string().into()))
            .map_err(|e| Error::General(format!("CDP send error: {e}")))?;

        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            if Instant::now() > deadline {
                return Err(Error::General(format!(
                    "timeout waiting for CDP response to {method}"
                )));
            }

            match self.socket.read() {
                Ok(Message::Text(text)) => {
                    let v: Value = serde_json::from_str(&text)
                        .map_err(|e| Error::General(format!("CDP JSON parse error: {e}")))?;
                    if v.get("id").and_then(Value::as_u64) == Some(u64::from(id)) {
                        if let Some(err) = v.get("error") {
                            return Err(Error::General(format!("CDP command error: {err}")));
                        }
                        return Ok(v["result"].clone());
                    } else if v.get("method").is_some() {
                        self.event_buffer.push(v);
                    }
                }
                Err(tungstenite::Error::Io(e))
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    // Read timeout — loop again
                }
                Err(e) => return Err(Error::General(format!("CDP read error: {e}"))),
                _ => {} // Binary / ping / pong / close — ignore
            }
        }
    }

    /// Wait for a specific CDP event method, with a deadline.
    ///
    /// Drains `event_buffer` first; then reads fresh messages until the event
    /// arrives or the timeout expires.
    pub fn wait_for_event(&mut self, method: &str, timeout: Duration) -> Result<Value> {
        // Check buffered events first.
        if let Some(pos) = self
            .event_buffer
            .iter()
            .position(|e| e.get("method").and_then(Value::as_str) == Some(method))
        {
            return Ok(self.event_buffer.remove(pos));
        }

        let deadline = Instant::now() + timeout;
        loop {
            if Instant::now() > deadline {
                return Err(Error::General(format!(
                    "timeout waiting for CDP event {method}"
                )));
            }

            match self.socket.read() {
                Ok(Message::Text(text)) => {
                    let v: Value = serde_json::from_str(&text)
                        .map_err(|e| Error::General(format!("CDP JSON parse error: {e}")))?;
                    if v.get("method").and_then(Value::as_str) == Some(method) {
                        return Ok(v);
                    } else if v.get("method").is_some() {
                        self.event_buffer.push(v);
                    }
                    // Command responses received here are discarded — we're only waiting for events.
                }
                Err(tungstenite::Error::Io(e))
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    // Read timeout — loop again
                }
                Err(e) => return Err(Error::General(format!("CDP read error: {e}"))),
                _ => {}
            }
        }
    }

    /// Evaluate a JavaScript expression in the page context.
    pub fn evaluate(&mut self, expr: &str) -> Result<Value> {
        self.send_command(
            "Runtime.evaluate",
            json!({
                "expression": expr,
                "returnByValue": true,
                "awaitPromise": true,
            }),
        )
    }
}

// ── Chrome launch ─────────────────────────────────────────────────────────────

/// Spawn Chrome in headless remote-debugging mode and return a connected `CdpSession`.
///
/// A temporary user-data directory is created for this Chrome instance so that it
/// does not conflict with an already-running Chrome (which holds the default profile
/// lock). The caller is responsible for removing the returned directory path after
/// Chrome has been killed.
pub fn spawn_chrome_for_cdp(chrome: &Path) -> Result<(Child, CdpSession, std::path::PathBuf)> {
    // Isolated profile dir — avoids profile-lock conflicts when Chrome is already running.
    let user_data_dir =
        std::env::temp_dir().join(format!("docanvil-chrome-{}", std::process::id()));
    std::fs::create_dir_all(&user_data_dir)
        .map_err(|e| Error::General(format!("failed to create Chrome profile dir: {e}")))?;

    let mut child = std::process::Command::new(chrome)
        .arg("--headless=new")
        .arg("--no-sandbox")
        .arg("--disable-gpu")
        .arg("--remote-debugging-port=0")
        .arg("--no-first-run")
        .arg("--disable-sync")
        .arg("--disable-extensions")
        .arg(format!("--user-data-dir={}", user_data_dir.display()))
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| Error::General(format!("failed to launch Chrome: {e}")))?;

    let stderr = child.stderr.take().expect("stderr was piped");

    let result = (|| -> Result<CdpSession> {
        let port = wait_for_chrome_ready(stderr, Duration::from_secs(15))?;
        let ws_url = get_page_ws_url(port)?;
        CdpSession::connect(&ws_url)
    })();

    match result {
        Ok(session) => Ok((child, session, user_data_dir)),
        Err(e) => {
            child.kill().ok();
            child.wait().ok();
            let _ = std::fs::remove_dir_all(&user_data_dir);
            Err(e)
        }
    }
}

/// Read Chrome's stderr until the DevTools WebSocket URL appears, then return the port.
///
/// All lines Chrome writes are collected; if Chrome exits early or the timeout fires
/// they are included in the error message so the caller can see what went wrong.
fn wait_for_chrome_ready(stderr: ChildStderr, timeout: Duration) -> Result<u16> {
    let (tx, rx) = mpsc::channel::<std::io::Result<String>>();

    std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            let done = line.is_err();
            let _ = tx.send(line);
            if done {
                break;
            }
        }
    });

    let mut collected: Vec<String> = Vec::new();

    let deadline = Instant::now() + timeout;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(chrome_startup_error(
                "Chrome did not start in time (DevTools URL not found)",
                &collected,
            ));
        }

        match rx.recv_timeout(remaining) {
            Ok(Ok(line)) => {
                // Chrome prints: DevTools listening on ws://127.0.0.1:PORT/devtools/browser/UUID
                // Newer builds may prefix the line with a log-level timestamp, so search
                // for the known marker anywhere in the line rather than anchoring at the start.
                const MARKER: &str = "DevTools listening on ws://";
                if let Some(pos) = line.find(MARKER)
                    && let Some(port) = parse_port_from_devtools_url(&line[pos + MARKER.len()..])
                {
                    return Ok(port);
                }
                collected.push(line);
            }
            Ok(Err(e)) => return Err(Error::General(format!("reading Chrome stderr: {e}"))),
            Err(mpsc::RecvTimeoutError::Timeout) => {
                return Err(chrome_startup_error(
                    "Chrome did not start in time (DevTools URL not found)",
                    &collected,
                ));
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err(chrome_startup_error(
                    "Chrome exited before DevTools became ready",
                    &collected,
                ));
            }
        }
    }
}

/// Build an error that includes everything Chrome printed to stderr, so the user
/// can see GPU errors, sandbox failures, missing library messages, and so on.
fn chrome_startup_error(reason: &str, chrome_output: &[String]) -> Error {
    if chrome_output.is_empty() {
        Error::General(format!("{reason}\n  (Chrome produced no output on stderr)"))
    } else {
        let lines = chrome_output
            .iter()
            .map(|l| format!("  {l}"))
            .collect::<Vec<_>>()
            .join("\n");
        Error::General(format!("{reason}\nChrome output:\n{lines}"))
    }
}

/// Parse `127.0.0.1:PORT/...` and return the port number.
fn parse_port_from_devtools_url(rest: &str) -> Option<u16> {
    // rest = "127.0.0.1:PORT/devtools/browser/UUID"
    let colon = rest.find(':')?;
    let after_colon = &rest[colon + 1..];
    let slash = after_colon.find('/')?;
    after_colon[..slash].parse::<u16>().ok()
}

/// HTTP GET `/json` from the Chrome DevTools endpoint and return the first page's WebSocket URL.
///
/// Parses the HTTP response properly — reads headers line-by-line, extracts
/// `Content-Length`, then reads exactly that many bytes for the body.  This
/// avoids the two failure modes of the naive `read_to_string` approach:
///   • HTTP/1.1 keep-alive: connection stays open, `read_to_string` blocks until
///     the read timeout fires (returning EAGAIN on macOS).
///   • HTTP/1.0 workaround: Chrome's DevTools server ignores HTTP/1.0 and closes
///     the connection immediately, leaving `read_to_string` with an empty string.
fn get_page_ws_url(port: u16) -> Result<String> {
    let mut stream = TcpStream::connect(format!("127.0.0.1:{port}"))
        .map_err(|e| Error::General(format!("failed to connect to Chrome DevTools: {e}")))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| Error::General(format!("set_read_timeout: {e}")))?;

    let request =
        format!("GET /json HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n");
    stream
        .write_all(request.as_bytes())
        .map_err(|e| Error::General(format!("failed to send DevTools HTTP request: {e}")))?;

    // Read headers line-by-line, extracting Content-Length.
    let mut reader = BufReader::new(&mut stream);
    let mut content_length: Option<usize> = None;
    loop {
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .map_err(|e| Error::General(format!("failed to read DevTools response header: {e}")))?;
        // Blank line signals end of headers.
        if line == "\r\n" || line.is_empty() {
            break;
        }
        if line.to_ascii_lowercase().starts_with("content-length:")
            && let Some((_, v)) = line.split_once(':')
        {
            content_length = v.trim().parse().ok();
        }
    }

    // Read the body.  If Content-Length is present (Chrome always sends it) we
    // read exactly that many bytes; otherwise we drain until EOF/timeout.
    let body = if let Some(len) = content_length {
        let mut buf = vec![0u8; len];
        reader
            .read_exact(&mut buf)
            .map_err(|e| Error::General(format!("failed to read DevTools response body: {e}")))?;
        String::from_utf8_lossy(&buf).into_owned()
    } else {
        let mut body = String::new();
        match reader.read_to_string(&mut body) {
            Ok(_) => {}
            // EAGAIN / TimedOut just means the connection was still open — use whatever we got.
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) => {}
            Err(e) => {
                return Err(Error::General(format!(
                    "failed to read DevTools response body: {e}"
                )));
            }
        }
        body
    };

    let targets: Vec<Value> = serde_json::from_str(&body).map_err(|e| {
        // Include a snippet of the raw body so it is easy to diagnose future
        // protocol changes without recompiling with debug prints.
        let preview: String = body.chars().take(200).collect();
        Error::General(format!(
            "failed to parse DevTools JSON: {e}\n  body: {preview:?}"
        ))
    })?;

    targets
        .into_iter()
        .find(|t| t["type"] == "page")
        .and_then(|t| t["webSocketDebuggerUrl"].as_str().map(String::from))
        .ok_or_else(|| Error::General("no page target found in Chrome DevTools".into()))
}

// ── Mermaid readiness ─────────────────────────────────────────────────────────

/// Poll until all Mermaid diagrams on the page have been rendered as SVG,
/// or until the timeout expires (graceful — returns `Ok(())` either way).
pub fn wait_for_mermaid(session: &mut CdpSession, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    let expr = "document.querySelectorAll('.mermaid').length > 0 && \
                document.querySelectorAll('.mermaid svg').length >= \
                document.querySelectorAll('.mermaid').length";

    loop {
        if Instant::now() > deadline {
            return; // Diagrams didn't finish in time — carry on without them.
        }

        if let Ok(result) = session.evaluate(expr)
            && result["result"]["value"].as_bool() == Some(true)
        {
            return;
        }

        std::thread::sleep(Duration::from_millis(200));
    }
}

// ── PDF rendering ─────────────────────────────────────────────────────────────

/// Returns `(width_inches, height_inches)` for a named paper size.
/// Defaults to A4 for unrecognised values.
fn paper_dimensions(size: &str) -> (f64, f64) {
    match size.to_lowercase().as_str() {
        "a3" => (11.69, 16.54),
        "a4" => (8.27, 11.69),
        "a5" => (5.83, 8.27),
        "letter" => (8.50, 11.00),
        "legal" => (8.50, 14.00),
        "tabloid" => (11.00, 17.00),
        _ => (8.27, 11.69), // A4 fallback
    }
}

/// Options for [`render_to_pdf_cdp`].
pub struct PdfRenderOptions<'a> {
    pub project_title: &'a str,
    pub pdf_author: Option<&'a str>,
    pub wait_mermaid: bool,
    pub paper_size: Option<&'a str>,
    pub accent_color: Option<&'a str>,
    pub quiet: bool,
}

/// Use Chrome DevTools Protocol to navigate to `html_path` and print it as a PDF.
///
/// Handles Mermaid polling (when `wait_mermaid` is `true`), running headers,
/// and page-number footers — all without Python or Playwright.
pub fn render_to_pdf_cdp(
    chrome: &Path,
    html_path: &Path,
    out_path: &Path,
    opts: PdfRenderOptions<'_>,
) -> Result<()> {
    let PdfRenderOptions {
        project_title,
        pdf_author,
        wait_mermaid,
        paper_size,
        accent_color,
        quiet,
    } = opts;
    if !quiet {
        eprintln!("Launching Chrome…");
    }
    let (mut child, mut session, user_data_dir) = spawn_chrome_for_cdp(chrome)?;
    if !quiet {
        eprintln!("Chrome ready.");
    }

    let result = (|| -> Result<()> {
        session.send_command("Page.enable", json!({}))?;

        if !quiet {
            eprintln!("Loading page…");
        }
        let file_url = format!("file://{}", html_path.display());
        session.send_command("Page.navigate", json!({ "url": file_url }))?;
        session.wait_for_event("Page.loadEventFired", Duration::from_secs(30))?;

        if wait_mermaid {
            if !quiet {
                eprintln!("Waiting for Mermaid diagrams…");
            }
            wait_for_mermaid(&mut session, Duration::from_secs(15));
        }

        if !quiet {
            eprintln!("Printing to PDF…");
        }
        let header_template = build_header_template(project_title, pdf_author, accent_color);
        let footer_template = build_footer_template();

        let (paper_width, paper_height) = paper_dimensions(paper_size.unwrap_or("A4"));

        let pdf_result = session.send_command(
            "Page.printToPDF",
            json!({
                "paperWidth":  paper_width,
                "paperHeight": paper_height,
                "marginTop":    0.787, // ≈ 2 cm
                "marginBottom": 0.787,
                "marginLeft":   0.984, // ≈ 2.5 cm
                "marginRight":  0.984,
                "printBackground": true,
                "displayHeaderFooter": true,
                "headerTemplate": header_template,
                "footerTemplate": footer_template,
            }),
        )?;

        let data_b64 = pdf_result["data"]
            .as_str()
            .ok_or_else(|| Error::General("CDP printToPDF returned no data field".into()))?;

        let pdf_bytes = base64::engine::general_purpose::STANDARD
            .decode(data_b64)
            .map_err(|e| Error::General(format!("failed to decode PDF base64: {e}")))?;

        std::fs::write(out_path, &pdf_bytes)
            .map_err(|e| Error::General(format!("failed to write PDF: {e}")))?;

        Ok(())
    })();

    child.kill().ok();
    child.wait().ok();
    let _ = std::fs::remove_dir_all(&user_data_dir);

    result
}

/// Build the CDP header template HTML (project title left, author right).
///
/// The accent colour is used for the thin rule separating the header from the page body,
/// so the running header picks up the project's brand colour automatically.
fn build_header_template(
    project_title: &str,
    pdf_author: Option<&str>,
    accent_color: Option<&str>,
) -> String {
    let title_escaped = crate::util::html_escape(project_title);
    let author_span = pdf_author
        .map(|a| {
            format!(
                r#"<span style="font-style:italic">{}</span>"#,
                crate::util::html_escape(a)
            )
        })
        .unwrap_or_default();
    let rule_color = accent_color.unwrap_or("#6366f1");

    format!(
        r#"<div style="font-size:7.5pt;color:#94a3b8;width:100%;padding:0 2.5cm 4pt;box-sizing:border-box;display:flex;justify-content:space-between;align-items:center;font-family:Georgia,serif;border-bottom:0.75pt solid {rule_color};"><span>{title_escaped}</span>{author_span}</div>"#
    )
}

/// Build the CDP footer template HTML (page number, right-aligned).
fn build_footer_template() -> String {
    r#"<div style="font-size:7.5pt;color:#94a3b8;width:100%;padding:4pt 2.5cm 0;box-sizing:border-box;text-align:right;font-family:Georgia,serif;"><span class="pageNumber"></span></div>"#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_port_from_devtools_url_valid() {
        assert_eq!(
            parse_port_from_devtools_url("127.0.0.1:12345/devtools/browser/abc"),
            Some(12345)
        );
    }

    #[test]
    fn parse_port_from_devtools_url_no_slash() {
        assert_eq!(parse_port_from_devtools_url("127.0.0.1:9222"), None);
    }

    #[test]
    fn build_header_template_with_author() {
        let h = build_header_template("My Docs", Some("Jane Doe"), None);
        assert!(h.contains("My Docs"));
        assert!(h.contains("Jane Doe"));
        assert!(h.contains("justify-content:space-between"));
    }

    #[test]
    fn build_header_template_no_author() {
        let h = build_header_template("My Docs", None, None);
        assert!(h.contains("My Docs"));
        assert!(!h.contains("Jane"));
    }

    #[test]
    fn build_header_template_uses_accent_color() {
        let h = build_header_template("My Docs", None, Some("#3b82f6"));
        assert!(h.contains("#3b82f6"));
    }

    #[test]
    fn build_header_template_default_accent_when_none() {
        let h = build_header_template("My Docs", None, None);
        // Falls back to the default indigo primary
        assert!(h.contains("#6366f1"));
    }

    #[test]
    fn build_footer_template_contains_page_number() {
        let f = build_footer_template();
        assert!(f.contains("pageNumber"));
    }

    #[test]
    fn build_header_escapes_html() {
        let h = build_header_template("A & B <Docs>", None, None);
        assert!(h.contains("A &amp; B &lt;Docs&gt;"));
        assert!(!h.contains("A & B <Docs>"));
    }

    #[test]
    fn paper_dimensions_known_sizes() {
        assert_eq!(paper_dimensions("A4"), (8.27, 11.69));
        assert_eq!(paper_dimensions("a4"), (8.27, 11.69));
        assert_eq!(paper_dimensions("A3"), (11.69, 16.54));
        assert_eq!(paper_dimensions("A5"), (5.83, 8.27));
        assert_eq!(paper_dimensions("Letter"), (8.50, 11.00));
        assert_eq!(paper_dimensions("legal"), (8.50, 14.00));
        assert_eq!(paper_dimensions("TABLOID"), (11.00, 17.00));
    }

    #[test]
    fn paper_dimensions_unknown_falls_back_to_a4() {
        assert_eq!(paper_dimensions("B5"), (8.27, 11.69));
        assert_eq!(paper_dimensions(""), (8.27, 11.69));
        assert_eq!(paper_dimensions("folio"), (8.27, 11.69));
    }
}
