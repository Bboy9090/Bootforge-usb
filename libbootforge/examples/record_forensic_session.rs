use libbootforge::{ForensicEventMonitor, Result, SessionRecorder};
use std::path::PathBuf;
use std::time::Duration;

fn main() -> Result<()> {
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("bootforge-usb-session.jsonl"));

    let mut watcher = ForensicEventMonitor::new()?;
    let mut recorder = SessionRecorder::create(&output)?;

    eprintln!("recording passive USB events to {}", output.display());
    loop {
        for event in watcher.wait_for_events(Duration::from_secs(1))? {
            let envelope = recorder.append(event)?;
            println!("{}", envelope.to_json_line()?);
        }
    }
}
