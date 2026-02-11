use std::io::Write;
use std::process::{Command, Stdio};

/// Copy text to the system clipboard.
/// Uses pbcopy on macOS, wl-copy on Wayland, xclip on X11.
pub fn copy_to_clipboard(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "macos")]
    let (cmd, args): (&str, Vec<&str>) = ("pbcopy", vec![]);

    #[cfg(target_os = "linux")]
    let (cmd, args): (&str, Vec<&str>) = {
        let session_type = std::env::var("XDG_SESSION_TYPE").unwrap_or_default();
        if session_type == "wayland" {
            ("wl-copy", vec![])
        } else {
            ("xclip", vec!["-selection", "clipboard"])
        }
    };

    let mut child = Command::new(cmd)
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to spawn {cmd}: {e}"))?;

    if let Some(ref mut stdin) = child.stdin {
        stdin.write_all(text.as_bytes())?;
    }

    let status = child.wait()?;
    if !status.success() {
        return Err(format!("{cmd} exited with status {status}").into());
    }

    Ok(())
}
