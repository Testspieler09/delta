#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        println!("[INFO]: {}", format!($($arg)*));
    };
}

#[macro_export]
macro_rules! error_print {
    ($($arg:tt)*) => {
        eprintln!("[ERROR]: {}", format!($($arg)*));
    };
}
