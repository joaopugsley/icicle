use notify::*;
use std::{path::Path, time::Duration, ffi::OsStr, fs::{File, self}, io::Read};

fn generate_jsfl_template(filename: String, content: String) -> String {
    let mut new_content = String::new();
    for line in content.lines() {
        new_content.push_str(&format!("{}\\n", line));
    }
    let template = format!(
        "fl.outputPanel.clear();\nfl.outputPanel.trace('[FLAWATCH] Refreshing...');\nvar file_name = '{}';\nvar file_content = '{}';\nvar doc=fl.getDocumentDOM();\nvar tl=doc.getTimeline();\ntl.layers[0].frames[0].actionScript=file_content;\nfl.openDocument('file:///' + file_name);\nfl.saveDocument(fl.getDocumentDOM(), 'file:///' + file_name);\nvar now = new Date();\nfl.outputPanel.clear();\nfl.outputPanel.trace('[FLAWATCH] Refreshed! ' + now);", 
        filename, new_content
    );
    template
}

fn main() {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher: Box<dyn Watcher> = if RecommendedWatcher::kind() == WatcherKind::PollWatcher {
        let config = Config::default().with_poll_interval(Duration::from_secs(1));
        Box::new(PollWatcher::new(tx, config).unwrap())
    } else {
        Box::new(RecommendedWatcher::new(tx, Config::default()).unwrap())
    };

    watcher
        .watch(Path::new("./src/"), RecursiveMode::Recursive)
        .unwrap();

    for event in rx {
        match event {
            Ok(e) => {
                for path in &e.paths {
                    if let Some("as") = path.extension().and_then(OsStr::to_str) {
                        let fla_path = path.with_extension("fla");
                        if fla_path.exists() && fla_path.file_stem() == path.file_stem() {
                            if let Ok(mut file) = File::open(path) {
                                let mut content = String::new();
                                if file.read_to_string(&mut content).is_ok() {
                                    let original_file_name = fla_path.file_stem().unwrap().to_string_lossy().to_string();
                                    let new_file_name = format!("{}_update.jsfl", original_file_name);
                                    if let Err(err) = fs::write(fla_path.parent().unwrap().join(new_file_name), generate_jsfl_template(original_file_name, content)) {
                                        eprintln!("Error {:?}", err);
                                    }
                                }
                            }
                        }
                    }
                };
            }
            Err(e) => println!("watch error {:?}", e)
        }
    }
}
