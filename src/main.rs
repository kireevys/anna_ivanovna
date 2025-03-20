use anna_ivanovna::cli;

fn main() {
    println!(
        "{}",
        cli::run().map_or_else(|e| format!("Ошибка: {e:?}"), |()| "Успех".to_string())
    );
}
