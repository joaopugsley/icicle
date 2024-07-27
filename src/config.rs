use std::fs::OpenOptions;
use std::io::{Write, Read};
use std::{fs, env, io};
use std::path::PathBuf;

fn get_os_config_path() -> Option<String> {
    #[cfg(target_os = "windows")]
    let (config_dir, config_file) = (env::var("APPDATA"), "Icicle\\config.cfg");

    #[cfg(not(target_os = "windows"))]
    let (config_dir, config_file) = (env::var("HOME"), ".icicle/config.cfg");

    match config_dir {
        Ok(dir) => {
            let config_path = format!("{dir}/{config_file}");
            Some(config_path)
        }
        Err(_) => None,
    }
}

pub fn save_config(key: &str, value: &str) -> io::Result<()> {
    if let Some(config_path) = get_os_config_path() {
        if let Some(parent_dir) = PathBuf::from(&config_path).parent() {
            fs::create_dir_all(parent_dir)?;
        }
        let line = format!("{key}={value}\n");
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(&config_path)?;
        file.write_all(line.as_bytes())?;
    } else {
        println!("Error: Could not get config path.")
    }
    Ok(())
}

pub fn load_config(key: &str) -> Option<String> {
    if let Ok(mut file) = fs::File::open("icicle.cfg") {
        let mut content = String::new();
        if file.read_to_string(&mut content).is_ok() {
            for line in content.lines() {
                let parts: Vec<&str> = line.splitn(2, '=').collect();
                if parts.len() == 2 && parts[0] == key {
                    return Some(parts[1].to_string());
                }
            }
        }
    }
    if let Some(config_path) = get_os_config_path() {
        if let Ok(file_content) = fs::read_to_string(&config_path) {
            for line in file_content.lines() {
                let parts: Vec<&str> = line.splitn(2, '=').collect();
                if parts.len() == 2 && parts[0] == key {
                    return Some(parts[1].to_string());
                }
            }
        }
    }
    None
}