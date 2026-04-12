use std::fs;

const OWN_DIR: &str = "./own";

// Ensures the ./own directory exists on startup
pub fn init_own_dir() {
    fs::create_dir_all(OWN_DIR)
        .expect("Failed to create ./own directory");
}

// Returns the full path for a file in ./own
pub fn own_file_path(filename: &str) -> String {
    format!("{}/{}", OWN_DIR, filename)
}

// Returns true if a file exists in ./own
pub fn file_exists(filename: &str) -> bool {
    std::path::Path::new(&own_file_path(filename)).exists()
}

// Returns list of filenames in ./own
pub fn list_files() -> Vec<String> {
    match fs::read_dir(OWN_DIR) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .filter_map(|e| e.file_name().into_string().ok())
            .collect(),
        Err(_) => {
            eprintln!("[Files] Could not read {} directory", OWN_DIR);
            Vec::new()
        }
    }
}