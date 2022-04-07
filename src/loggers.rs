use std::io::Write;

pub fn create_logging_directory() -> std::io::Result<()> {
    let logging_directory = get_logging_directory();
    if !std::path::Path::new(logging_directory.as_str()).is_dir() {
        std::fs::create_dir(logging_directory)?;
    }
    Ok(())
}

pub fn get_logging_directory() -> String {
    return format!("./{}", clap::crate_name!());
}

pub fn log_to_ui(error: &str) {
    println!("GL-HOOK-ERR: Error: {}", error);
}

// This function is used for logging.
pub fn log_to_file(message: &str) -> std::io::Result<()> {
    let dt = chrono::Utc::now();
    let message = format!("{}: {}", dt.format("%Y-%m-%dT%H:%M:%S"), message);
    let logging_directory_file = format!("{}/error.log", get_logging_directory());
    let path = std::path::Path::new(logging_directory_file.as_str());
    let mut file = if path.exists() {
        std::fs::OpenOptions::new()
            .append(true)
            .open(logging_directory_file)?
    } else {
        std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(logging_directory_file)?
    };
    file.write_all(format!("{}\n", message).as_ref())?;

    Ok(())
}
