use std::process::Command;
use chrono::Utc;
use sha2::{Sha256, Digest};
use crate::config::load_config;

pub fn clear() {
    if cfg!(target_os = "windows") {
        let _ = Command::new("cmd").arg("/c").arg("cls").status();
    } else {
        let _ = Command::new("clear").status();
    }
}

pub fn fix_wine_path(path: String) -> String {
    let disk = load_config("custom_disk").unwrap_or("C".to_string());
    let disk_path = format!("{disk}:/users/");
    return path
        .replace("/home/", &disk_path)
        .replace("Documentos", "Documents");
}

pub fn generate_content_hash(content: String) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    let result = hasher.finalize();
    format!("{:x}", result)
}

pub fn generate_jsfl_template(file_name: String, content: String) -> String {
    let mut new_content = String::new();
    for line in content.lines() {
        let fixed_line = line.replace(r#"\"#, r#"\\"#).to_string();
        new_content.push_str(&format!("{fixed_line}\\n"));
    }
    let dt = Utc::now();
    let timestamp: i64 = dt.timestamp();
    let timeline_layer = load_config("custom_script_layer").unwrap_or("1".to_string());
    let auto_publish = load_config("auto_publish").unwrap_or_default() == "true";
    let template = format!("//{timestamp}\nfl.outputPanel.clear();\nfl.outputPanel.trace('[ICICLE] Refreshing...');\nvar loc = fl.scriptURI;\nvar scriptFolder = loc.substring(0, loc.lastIndexOf('/') + 1);\nvar file_path = scriptFolder + '{file_name}.fla';\nvar file_content = '{new_content}';\nvar action_layer = {timeline_layer};\nvar auto_publish = {auto_publish};\nif (!fl.fileExists(file_path)) {{\n    fl.outputPanel.clear();\n    fl.outputPanel.trace('[ICICLE] ERROR: ' + file_path + ' does not exist.');\n}} else {{\n    fl.openDocument(file_path);\n    var doc = fl.getDocumentDOM();\n    var tl = doc.getTimeline();\n    if(!tl.layers[action_layer]) {{\n        fl.outputPanel.clear();\n        fl.outputPanel.trace('[ICICLE] ActionScript layer default is: 1, if you want to use another layer modify the Icicle config or create a new one.');\n    }} else {{\n        tl.layers[action_layer].frames[0].actionScript = file_content;\n        fl.saveDocument(fl.getDocumentDOM());\n        var now = new Date();\n        fl.outputPanel.clear();\n        fl.outputPanel.trace('[ICICLE] Refreshed! ' + now);\n        if(auto_publish == true) {{\n		    doc.publish();\n        }}\n    }}\n}}");
    
    template
}