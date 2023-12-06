use colored::Colorize;

pub fn info(message: &str) {
    println!("{} {}", "[INFO]".green(), message);
}

pub fn error(message: &str) {
    eprintln!("{} {}", "[ERROR]".red(), message);
}

