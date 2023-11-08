use notify::*;
use chrono::Utc;
use std::{path::Path, time::Duration, ffi::OsStr, fs::{File, self}, io::Read};

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


fn main() {
    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher: Box<dyn Watcher> = if RecommendedWatcher::kind() == WatcherKind::PollWatcher {
        let config = Config::default().with_poll_interval(Duration::from_secs(2));
        Box::new(PollWatcher::new(tx, config).unwrap())
    } else {
        Box::new(RecommendedWatcher::new(tx, Config::default()).unwrap())
    };

    watcher
        .watch(Path::new("./src/"), RecursiveMode::Recursive)
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
                                    let original_file_name = fla_path.file_stem().unwrap().to_string_lossy().to_string();
                                    let new_file_name = format!("{}_update.jsfl", original_file_name);
                                    match fs::write(fla_path.parent().unwrap().join(new_file_name), generate_jsfl_template(original_file_name, content)) {
                                        Ok(()) => {
                                            println!("generated :)")
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