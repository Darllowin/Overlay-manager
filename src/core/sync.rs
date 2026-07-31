use anyhow::Result;
use std::io::BufRead;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;

/// Sync event for transmission to the UI.
#[derive(Debug, Clone)]
pub enum SyncEvent {
    /// Sync started for a specific overlay.
    Started(String),
    /// Output line from emaint sync.
    Output(String),
    /// Sync completed.
    Finished(Result<(), String>),
}

/// Run emaint sync -a (all repositories).
pub fn sync_all() -> mpsc::Receiver<SyncEvent> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        tx.send(SyncEvent::Started("all".into())).ok();
        let result = run_emaint_all(&tx);
        tx.send(SyncEvent::Finished(result)).ok();
    });

    rx
}

fn run_emaint_all(tx: &mpsc::Sender<SyncEvent>) -> Result<(), String> {
    use std::io::Write;

    let mut child = Command::new("emaint")
        .args(["sync", "-a"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start emaint: {}", e))?;

    // Automatically answer "y" to confirmation prompt
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(b"y\n").ok();
    }

    if let Some(stdout) = child.stdout.take() {
        let reader = std::io::BufReader::new(stdout);
        for line in reader.lines() {
            match line {
                Ok(text) => {
                    tx.send(SyncEvent::Output(text)).ok();
                }
                Err(_) => break,
            }
        }
    }

    let status = child
        .wait()
        .map_err(|e| format!("emaint wait error: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("emaint exited with code {}", status))
    }
}

/// Start syncing a single overlay in a separate thread.
pub fn sync_repo(name: String) -> mpsc::Receiver<SyncEvent> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        tx.send(SyncEvent::Started(name.clone())).ok();

        let result = run_emaint(&name, &tx);
        tx.send(SyncEvent::Finished(result)).ok();
    });

    rx
}

/// Run emaint sync -r <repo> and stream stdout line by line to the channel.
fn run_emaint(name: &str, tx: &mpsc::Sender<SyncEvent>) -> Result<(), String> {
    use std::io::Write;

    let mut child = Command::new("emaint")
        .args(["sync", "-r", name])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start emaint: {}", e))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(b"y\n").ok();
    }

    // Read stdout line by line
    if let Some(stdout) = child.stdout.take() {
        let reader = std::io::BufReader::new(stdout);
        for line in reader.lines() {
            match line {
                Ok(text) => {
                    tx.send(SyncEvent::Output(text)).ok();
                }
                Err(_) => break,
            }
        }
    }

    let status = child
        .wait()
        .map_err(|e| format!("emaint wait error: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("emaint exited with code {}", status))
    }
}
