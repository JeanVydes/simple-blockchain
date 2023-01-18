#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Logger {
    pub name: String,
}

impl Logger {
    pub fn log(&self, message: &str) {
        let log_line = format!("[{}] {}", self.name, message);
        println!("{}", log_line);
    }
}