use std::{fs::create_dir_all, path::Path};

use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt::writer::MakeWriterExt;

pub fn init(dir: &Path, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let logdir = dir.join("logs");
    if let Err(e) = create_dir_all(&logdir) {
        eprintln!("Не удалось создать директорию для логов: {e}");
        std::process::exit(1);
    }

    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix(filename)
        .max_log_files(7)
        .build(&logdir)?;

    let writer = std::io::stderr.and(file_appender);
    tracing_subscriber::fmt()
        .with_writer(writer)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .init();
    Ok(())
}
