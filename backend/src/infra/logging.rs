use std::fs::{OpenOptions, create_dir_all};
use std::path::Path;
use tracing_subscriber::fmt::writer::MakeWriterExt;

pub fn init(dir: &Path, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let logdir = dir.join("logs");
    let logfile = logdir.join(filename);
    if let Err(e) = create_dir_all(&logdir) {
        eprintln!("Не удалось создать директорию для логов: {e}");
        std::process::exit(1);
    }
    let file = OpenOptions::new().create(true).append(true).open(logfile)?;
    let writer = std::io::stderr.and(file);
    tracing_subscriber::fmt()
        .with_writer(writer)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .init();
    Ok(())
}
