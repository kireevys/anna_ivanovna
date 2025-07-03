#[cfg(windows)]
fn main() {
    use std::env;
    use std::process::Command;

    // Проверяем, есть ли переменная окружения, указывающая на терминал
    let is_terminal = env::var("TERM").is_ok() || env::var("WT_SESSION").is_ok();

    if !is_terminal {
        // Открываем PowerShell с командой help
        let exe = env::current_exe().unwrap();
        Command::new("powershell")
            .arg("-NoExit")
            .arg("-Command")
            .arg(format!("{} --help; pause", exe.display()))
            .spawn()
            .unwrap();
        return;
    }

    anna_ivanovna::cli::run().unwrap();
}

#[cfg(not(windows))]
fn main() {
    anna_ivanovna::cli::run().unwrap();
}
