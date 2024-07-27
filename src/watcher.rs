use notify::{RecommendedWatcher, Watcher, RecursiveMode, PollWatcher, Config, WatcherKind, EventKind};
use std::{path::{Path, PathBuf}, time::Duration, ffi::OsStr, fs, io, process::Command};
use crate::utils::{clear, fix_wine_path, generate_content_hash, generate_jsfl_template, read_as_file_and_content};

pub fn start_watcher(flash_exe_path: String) {
    clear();
    println!("[ICICLE] Started! Current Flash executable path: {:?}", flash_exe_path);
    println!("[ICICLE] Waiting for file changes...");

    let mut last_generated_hash = String::new();
    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher: Box<dyn Watcher> = create_watcher(tx);

    watcher
        .watch(Path::new("./"), RecursiveMode::Recursive)
        .unwrap();

    for event in rx {
        match handle_event(event, &mut last_generated_hash, &flash_exe_path) {
            Ok(()) => {
                clear();
                println!("[ICICLE] Waiting for file changes...");
            }
            Err(error) => {
                eprintln!("[ICICLE] Error: {:?}", error);
            }
        }
    }
}

fn create_watcher(tx: std::sync::mpsc::Sender<Result<notify::Event, notify::Error>>) -> Box<dyn Watcher> {
    if RecommendedWatcher::kind() == WatcherKind::PollWatcher {
        let config = Config::default().with_poll_interval(Duration::from_secs(2));
        Box::new(PollWatcher::new(tx, config).expect("[ICICLE] Error: failed to create PollWatcher"))
    } else {
        Box::new(RecommendedWatcher::new(tx, Config::default()).expect("[ICICLE] Error: failed to create RecommendedWatcher"))
    }
}

fn handle_event(event: Result<notify::Event, notify::Error>, last_generated_hash: &mut String, flash_exe_path: &str) -> Result<(), io::Error> {
    match event {
        Ok(e) if matches!(e.kind, EventKind::Modify(_)) => {
            for path in &e.paths {
                if let Some("as") = path.extension().and_then(OsStr::to_str) {
                    if let Some((fla_path, content)) = read_as_file_and_content(&path) {
                        if fla_path.exists() && fla_path.file_stem() == path.file_stem() {
                            let new_content_hash = generate_content_hash(content.clone());
                            if last_generated_hash.as_str() != new_content_hash {
                                handle_file_change(
                                    fla_path,
                                    &new_content_hash,
                                    last_generated_hash,
                                    flash_exe_path,
                                )?;
                            }
                        }
                    }
                }
            }
        }
        Err(e) => eprintln!("[ICICLE] Error: watch error {:?}", e),
        _ => {} // ignoring other events
    }
    Ok(())
}

fn handle_file_change(fla_path: PathBuf, new_content_hash: &str, last_generated_hash: &mut String, flash_exe_path: &str) -> Result<(), io::Error> {
    last_generated_hash.clear();
    last_generated_hash.push_str(new_content_hash);
        
    clear();
    println!("[ICICLE] File changed, refreshing...");

    let original_file_name = fla_path.file_stem().unwrap().to_string_lossy().to_string();
    let new_file_name = format!("{original_file_name}_update.jsfl");
    let jsfl_template = generate_jsfl_template(original_file_name, new_content_hash.to_owned());
    let beauty_jsfl_path = fla_path.with_file_name(new_file_name.clone()).to_string_lossy().to_string().replace("\\", "/");

    fs::write(fla_path.parent().unwrap().join(&new_file_name), &jsfl_template)?;

    #[cfg(target_os = "windows")]
    let result = Command::new(flash_exe_path).arg("-RunScript").arg(beauty_jsfl_path).output();

    #[cfg(not(target_os = "windows"))]
    let result = Command::new("wine").arg(flash_exe_path).arg("-RunScript").arg(fix_wine_path(beauty_jsfl_path)).output();

    match result {
        Ok(_) => Ok(()),
        Err(error) => {
            eprintln!("[ICICLE] Error: {:?}", error);
            Ok(())
        }
    }
}