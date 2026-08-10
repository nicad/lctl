fn main() {
    if let Err(e) = angel::run() {
        angel::print_error(&e);
        std::process::exit(1);
    }
}
