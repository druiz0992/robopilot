
use env_logger::Builder;
use std::env;
use std::io::{self, Write};
use chrono::Local;

pub fn init() {
    Builder::new()
        .parse_filters(&env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string())) // Set log level via RUST_LOG env var
        .format(|buf, record| {
            let now = Local::now(); // Get current time with chrono
            let formatted_time = now.format("%Y-%m-%d %H:%M:%S%.3f"); // Format time to include milliseconds
                                                                      // Custom log format: timestamp - log_level - message
            writeln!(
                buf,
                "{:<30} - {:<5} - {}",
                formatted_time,
                record.level(),
                record.args()
            )
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e)) // Handle any formatting errors
        })
        .init(); // Initialize the logger
}