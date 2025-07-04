fn main() {
    use std::fs::File;
    use tracing_subscriber::{fmt, prelude::*};

    let file = File::create("anna_ivanovna.log").expect("Не удалось создать лог-файл");
    let file_layer = fmt::layer().with_writer(file).with_ansi(false);

    tracing_subscriber::registry().with(file_layer).init();

    anna_ivanovna::cli::run().unwrap();
}
