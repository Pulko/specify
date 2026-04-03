fn main() -> std::process::ExitCode {
    match specify::run() {
        Ok(code) => std::process::ExitCode::from(code as u8),
        Err(e) => {
            eprintln!("{e:#}");
            std::process::ExitCode::from(1)
        }
    }
}
