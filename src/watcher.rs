use notify::{RecommendedWatcher, Watcher, RecursiveMode, PollWatcher, Config, WatcherKind, EventKind};
use std::{path::Path, time::Duration, ffi::OsStr, fs::{File, self}, io::Read, process::Command};
use crate::utils::{clear, fix_wine_path, generate_content_hash, generate_jsfl_template};

pub fn start_watcher(flash_exe_path: String) {
    clear();
    println!("[ICICLE] Started! Current Flash executable path: {:?}", flash_exe_path);
    println!("[ICICLE] Waiting for file changes...");

    let mut last_generated_hash = String::new();

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
                            if let Ok(mut file) = File::open(path) {
                                let mut content = String::new();
                                if file.read_to_string(&mut content).is_ok() {
                                    let new_content_hash = generate_content_hash(content.clone());
                                    if last_generated_hash != new_content_hash {
                                        last_generated_hash = new_content_hash.clone();

                                        clear();
                                        println!("[ICICLE] File changed, refreshing...");

                                        let original_file_name = fla_path.file_stem().unwrap().to_string_lossy().to_string();
                                        let new_file_name = format!("{original_file_name}_update.jsfl");
                                        let jsfl_template = generate_jsfl_template(original_file_name, content);
                                        let beauty_jsfl_path = fla_path.with_file_name(new_file_name.clone()).to_string_lossy().to_string().replace("\\", "/");
                                        
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
                    }
                };
            }
            Err(e) => println!("Watch error {:?}", e),
            _ => {} // ignoring other events
        }
    }
}