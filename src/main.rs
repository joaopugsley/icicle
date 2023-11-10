use notify::*;
use chrono::Utc;
use std::{path::{Path, PathBuf}, time::Duration, ffi::OsStr, fs::{File, self, OpenOptions}, io::{Read, self, Write}, process::Command, env};
use rfd::FileDialog;

fn fix_wine_path(path: String) -> String {
    let disk = load_config("custom_disk").unwrap_or("C".to_string());
    let disk_path = format!("{}:/users/", disk);
    return path
        .replace("/home/", &disk_path)
        .replace("Documentos", "Documents");
}

fn generate_jsfl_template(file_path: String, content: String) -> String {
    let mut new_content = String::new();
    for line in content.lines() {
        new_content.push_str(&format!("{}\\n", line));
    }
    let dt = Utc::now();
    let timestamp: i64 = dt.timestamp();
    let template = format!("//{}\nfl.outputPanel.clear();\nfl.outputPanel.trace('[ICICLE] Refreshing...');\nvar file_path = 'file:///{}';\nif (!fl.fileExists(file_path)) {{\n    fl.outputPanel.clear();\n    fl.outputPanel.trace('[ICICLE] ERROR: ' + file_path + ' does not exist.');\n}} else {{\n    fl.openDocument(file_path);\n    var file_content = '{}';\n    var doc = fl.getDocumentDOM();\n    var tl = doc.getTimeline();\n    tl.layers[0].frames[0].actionScript = file_content;\n    fl.saveDocument(fl.getDocumentDOM());\n    var now = new Date();\n    fl.outputPanel.clear();\n    fl.outputPanel.trace('[ICICLE] Refreshed! ' + now);\n}}",
        timestamp, file_path, new_content
    );
    template
}

fn start_watcher(flash_exe_path: String) {
    clear();
    println!("[ICICLE] Started! Current Flash executable path: {:?}", flash_exe_path);
    println!("[ICICLE] Waiting for file changes...");

    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher: Box<dyn Watcher> = if RecommendedWatcher::kind() == WatcherKind::PollWatcher {
        let config = Config::default().with_poll_interval(Duration::from_secs(2));
        Box::new(PollWatcher::new(tx, config).unwrap())
    } else {
        Box::new(RecommendedWatcher::new(tx, Config::default()).unwrap())
    };

    watcher
        .watch(Path::new("./"), RecursiveMode::Recursive)
        .unwrap();

    for event in rx {
        match event {
            Ok(e) if matches!(e.kind, EventKind::Modify(_)) => {
                for path in &e.paths {
                    if let Some("as") = path.extension().and_then(OsStr::to_str) {
                        let fla_path = path.with_extension("fla");
                        if fla_path.exists() && fla_path.file_stem() == path.file_stem() {
                            clear();
                            println!("[ICICLE] File changed, refreshing...");
                            if let Ok(mut file) = File::open(path) {
                                let mut content = String::new();
                                if file.read_to_string(&mut content).is_ok() {
                                    let original_file_name = fla_path.file_stem().unwrap().to_string_lossy().to_string();
                                    let new_file_name = format!("{}_update.jsfl", original_file_name);
                                    let beauty_file_path = fla_path.as_os_str().to_str().unwrap().replace("\\", "/");
                                    let beauty_jsfl_path = fla_path.with_file_name(new_file_name.clone()).to_string_lossy().to_string().replace("\\", "/");
                                    
                                    #[cfg(target_os = "windows")]
                                    let jsfl_template = generate_jsfl_template(beauty_file_path, content);
                                    #[cfg(not(target_os = "windows"))]
                                    let jsfl_template = generate_jsfl_template(fix_wine_path(beauty_file_path), content);

                                    match fs::write(fla_path.parent().unwrap().join(&new_file_name), &jsfl_template) {
                                        Ok(()) => {
                                            #[cfg(target_os = "windows")]
                                            let result = Command::new(&flash_exe_path).arg("-RunScript").arg(beauty_jsfl_path).output();
                                            #[cfg(not(target_os = "windows"))]
                                            let result = Command::new("wine").arg(&flash_exe_path).arg("-RunScript").arg(fix_wine_path(beauty_jsfl_path)).output();

                                            match result {
                                                Ok(_) => {
                                                    clear();
                                                    println!("[ICICLE] Waiting for file changes...");
                                                }
                                                Err(erro) => println!("Error: {:?}", erro)
                                            }
                                        }
                                        Err(er) => println!("Error while creating jsfl {:?}", er)
                                    }
                                }
                            }
                        }
                    }
                };
            }
            Err(e) => println!("Watch error {:?}", e),
            _ => {} // ignoring other events
        }
    }
}

fn clear() {
    if cfg!(target_os = "windows") {
        let _ = Command::new("cmd").arg("/c").arg("cls").status();
    } else {
        let _ = Command::new("clear").status();
    }
}

fn get_os_config_path() -> Option<String> {
    #[cfg(target_os = "windows")]
    let (config_dir, config_file) = (env::var("APPDATA"), "Icicle\\config.cfg");

    #[cfg(not(target_os = "windows"))]
    let (config_dir, config_file) = (env::var("HOME"), ".icicle/config.cfg");

    match config_dir {
        Ok(dir) => {
            let config_path = format!("{}/{}", dir, config_file);
            Some(config_path)
        }
        Err(_) => None,
    }
}

fn save_config(key: &str, value: &str) -> io::Result<()> {
    if let Some(config_path) = get_os_config_path() {
        if let Some(parent_dir) = PathBuf::from(&config_path).parent() {
            fs::create_dir_all(parent_dir)?;
        }
        let line = format!("{}={}\n", key, value);
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

fn load_config(key: &str) -> Option<String> {
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