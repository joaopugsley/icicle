mod watcher;
mod config;
mod utils;

use watcher::start_watcher;
use utils::clear;
use config::{load_config, save_config};
use rfd::FileDialog;

fn main() {
    if let Some(flash_path) = load_config("flash_path") {
        start_watcher(flash_path);
    } else {
        clear();
        println!("[ICICLE] Error: Flash executable could not be found. Please select it using the file explorer. You will just need to do this once.");
        let file: Option<std::path::PathBuf> = FileDialog::new()
            .set_title("Select the FLASH executable")
            .add_filter("Executable", &["exe"])
            .pick_file();

        match file {
            Some(path) => {
                let file_path = path.to_string_lossy().replace("\\", "/");
                let _ = save_config("flash_path", &file_path);
                start_watcher(file_path);
            }
            None => println!("No file selected."),
        }
    }
}