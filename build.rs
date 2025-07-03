#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("icon.ico");
    res.compile().unwrap();
}

#[cfg(not(windows))]
fn main() {
    println!("Good choice of OS!")
}
