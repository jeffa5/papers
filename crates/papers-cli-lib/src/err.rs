/// Print an error.
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        eprintln!("error: {}", format_args!($($arg)*))
    }};
}
