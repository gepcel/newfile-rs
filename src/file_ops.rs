use clipboard_win::get_clipboard_string;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process::Command;

pub fn create_new_file(name: &str, write_clipboard: bool, open_after: bool, root: &str, editor_path: &str) {
    if name.is_empty() {
        return;
    }

    let root_path = Path::new(root);
    let target = root_path.join(name);

    if let Some(parent) = target.parent() {
        if !parent.as_os_str().is_empty() {
            let _ = fs::create_dir_all(parent);
        }
    }

    let mut counter = 0;
    let mut full_path = target.clone();
    while full_path.exists() {
        counter += 1;
        let parent = full_path.parent().unwrap_or_else(|| Path::new(""));
        let stem = full_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        let ext = full_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let file_name = if ext.is_empty() {
            format!("{}{}", stem, counter)
        } else {
            format!("{}{}.{}", stem, counter, ext)
        };
        full_path = parent.join(file_name);
    }

    if let Some(parent) = full_path.parent() {
        if !parent.as_os_str().is_empty() {
            let _ = fs::create_dir_all(parent);
        }
    }

    if let Ok(mut file) = File::create(&full_path) {
        if write_clipboard {
            if let Ok(text) = get_clipboard_string() {
                let _ = file.write_all(text.as_bytes());
            }
        }
    }

    if open_after {
        let mut editor = if editor_path.is_empty() {"notepad.exe"} else {editor_path};
        if !Path::new(editor).exists() && editor != "notepad.exe"{
            editor = "notepad.exe";
        }
        #[cfg(target_os = "windows")]
        {
            let _ = Command::new(editor)
                .arg(full_path.to_string_lossy().as_ref())
                .spawn();
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = Command::new("xdg-open")
                .arg(&full_path.to_string_lossy())
                .spawn();
        }
    }
}

pub fn create_new_folder(name: &str, open_after: bool, root: &str, totalcmd_path: &str) {
    if name.is_empty() {
        return;
    }

    let root_path = Path::new(root);
    let target = root_path.join(name);

    let mut counter = 0;
    let mut full_path = target.clone();
    while full_path.exists() {
        counter += 1;
        let parent = target.parent().unwrap_or_else(|| Path::new(""));
        let stem = target
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        let new_name = format!("{}{}", stem, counter);
        full_path = parent.join(new_name);
    }

    let _ = fs::create_dir_all(&full_path);

    if open_after && Path::new(totalcmd_path).exists() {
        let full_path_str = full_path.to_string_lossy().to_string()
            .replace("/", "\\");
        let _ = Command::new(totalcmd_path)
            .arg("/O")
            .arg("/S")
            .arg(format!("/L={}", full_path_str))
            .spawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn create_nested_folder() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_string_lossy().to_string();

        create_new_folder("foo/bar", false, &root, "nope");
        assert!(tmp.path().join("foo").is_dir());
        assert!(tmp.path().join("foo/bar").is_dir());

        create_new_folder("foo/bar", false, &root, "nope");
        assert!(tmp.path().join("foo/bar1").is_dir());
    }

    #[test]
    fn create_nested_file() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_string_lossy().to_string();

        create_new_file("a/b/file.txt", false, false, &root);
        assert!(tmp.path().join("a/b/file.txt").is_file());

        create_new_file("a/b/file.txt", false, false, &root);
        assert!(tmp.path().join("a/b/file1.txt").is_file());
    }
}