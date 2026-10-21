fn main() {
    if let Err(e) = lctl::run() {
        lctl::print_error(&e);
        std::process::exit(1);
    }
}
