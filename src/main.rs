fn main() {
    if let Err(e) = angel::run() {
        angel::output::stderr::error(&e.to_string());
        std::process::exit(1);
    }
}
