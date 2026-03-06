use clipboard_win::get_clipboard_string;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process::Command;

pub fn create_new_file(name: &str, write_clipboard: bool, open_after: bool, root: &str, editor_path: &str) {
    eprintln!("[FILE_OPS] create_new_file: name='{}', write_clipboard={}, open_after={}, root='{}', editor_path='{}'",
              name, write_clipboard, open_after, root, editor_path);
    if name.is_empty() {
        eprintln!("[FILE_OPS] name is empty, returning");
        return;
    }

    let root_path = Path::new(root);
    let target = root_path.join(name);
    eprintln!("[FILE_OPS] target = {:?}", target);

    // create parent directories if specified
    if let Some(parent) = target.parent() {
        if !parent.as_os_str().is_empty() {
            eprintln!("[FILE_OPS] creating parent dirs: {:?}", parent);
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!("[FILE_OPS] failed to create parent dirs: {}", e);
            } else {
                eprintln!("[FILE_OPS] parent dirs created");
            }
        }
    }

    // handle duplicate file names by appending a counter
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
        eprintln!("[FILE_OPS] duplicate, trying new path: {:?}", full_path);
    }

    // ensure parent directories again just in case
    if let Some(parent) = full_path.parent() {
        if !parent.as_os_str().is_empty() {
            eprintln!("[FILE_OPS] re-creating parent dirs (just in case): {:?}", parent);
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!("[FILE_OPS] failed to re-create parent dirs: {}", e);
            }
        }
    }

    // create the file and optionally write clipboard
    eprintln!("[FILE_OPS] creating file at {:?}", full_path);
    match File::create(&full_path) {
        Ok(mut file) => {
            eprintln!("[FILE_OPS] file created successfully");
            if write_clipboard {
                eprintln!("[FILE_OPS] attempting to write clipboard content");
                match get_clipboard_string() {
                    Ok(text) => {
                        if let Err(e) = file.write_all(text.as_bytes()) {
                            eprintln!("[FILE_OPS] failed to write clipboard to file: {}", e);
                        } else {
                            eprintln!("[FILE_OPS] clipboard written to file ({} bytes)", text.len());
                        }
                    }
                    Err(e) => {
                        eprintln!("[FILE_OPS] failed to get clipboard string: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("[FILE_OPS] failed to create file: {}", e);
            return;
        }
    }

    if open_after {
        let mut editor = if editor_path.is_empty() {"notepad.exe"} else {editor_path};
        eprintln!("[FILE_OPS] open_after is true, using editor = {}", editor);
        if !Path::new(editor).exists() && editor != "notepad.exe"{
            eprintln!("[FILE_OPS] editor path '{}' does not exist, falling back to notepad.exe", editor);
            editor = "notepad.exe";
        }
        #[cfg(target_os = "windows")]
        {
            eprintln!("[FILE_OPS] spawning editor: {} with file {:?}", editor, full_path);
            match Command::new(editor)
                .arg(full_path.to_string_lossy().as_ref())
                .spawn()
            {
                Ok(mut child) => eprintln!("[FILE_OPS] editor spawned, pid = {:?}", child.id()),
                Err(e) => eprintln!("[FILE_OPS] failed to spawn editor: {}", e),
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            eprintln!("[FILE_OPS] spawning xdg-open with file {:?}", full_path);
            match Command::new("xdg-open")
                .arg(&full_path.to_string_lossy())
                .spawn()
            {
                Ok(mut child) => eprintln!("[FILE_OPS] xdg-open spawned, pid = {:?}", child.id()),
                Err(e) => eprintln!("[FILE_OPS] failed to spawn xdg-open: {}", e),
            }
        }
    } else {
        eprintln!("[FILE_OPS] open_after is false, not opening");
    }
    eprintln!("[FILE_OPS] create_new_file finished");
}

// recursively create a folder structure and optionally open TotalCMD at the final
// destination. duplicate names are handled by appending a counter like
// create_new_file does.
pub fn create_new_folder(name: &str, open_after: bool, root: &str, totalcmd_path: &str) {
    eprintln!("[FILE_OPS] create_new_folder: name='{}', open_after={}, root='{}', totalcmd_path='{}'",
              name, open_after, root, totalcmd_path);
    if name.is_empty() {
        eprintln!("[FILE_OPS] name is empty, returning");
        return;
    }

    let root_path = Path::new(root);
    let target = root_path.join(name);
    eprintln!("[FILE_OPS] target = {:?}", target);

    // handle duplicates on the last segment
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
        eprintln!("[FILE_OPS] duplicate, trying new path: {:?}", full_path);
    }

    eprintln!("[FILE_OPS] creating directories at {:?}", full_path);
    match fs::create_dir_all(&full_path) {
        Ok(_) => eprintln!("[FILE_OPS] directories created successfully"),
        Err(e) => eprintln!("[FILE_OPS] failed to create directories: {}", e),
    }

    if open_after && Path::new(totalcmd_path).exists() {
        eprintln!("[FILE_OPS] open_after true and TotalCMD exists at {}", totalcmd_path);
        // Convert path to Windows format with backslashes
        let full_path_str = full_path.to_string_lossy().to_string()
            .replace("/", "\\");
        eprintln!("[FILE_OPS] opening TotalCMD with /L={}", full_path_str);
        match Command::new(totalcmd_path)
            .arg("/O")
            .arg("/S")
            .arg(format!("/L={}", full_path_str))
            .spawn()
        {
            Ok(mut child) => eprintln!("[FILE_OPS] TotalCMD spawned, pid = {:?}", child.id()),
            Err(e) => eprintln!("[FILE_OPS] failed to spawn TotalCMD: {}", e),
        }
    } else if open_after {
        eprintln!("[FILE_OPS] open_after true but TotalCMD path does not exist: {}", totalcmd_path);
    } else {
        eprintln!("[FILE_OPS] open_after false, not opening TotalCMD");
    }
    eprintln!("[FILE_OPS] create_new_folder finished");
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