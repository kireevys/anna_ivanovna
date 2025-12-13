use once_cell::sync::Lazy;
use url::Url;

const DEFAULT_API_BASE: &str = "http://localhost:3000/v1/";

pub static API_V1_BASE_URL: Lazy<Url> = Lazy::new(|| {
    // В будущем можно читать из переменных окружения
    Url::parse(DEFAULT_API_BASE).expect("Invalid API_BASE URL")
});
