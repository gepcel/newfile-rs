use configparser::ini::Ini;
use std::path::Path;
use dunce;
use std::env;

pub struct NewFileConfig {
    pub clipboard: bool,
    pub open_after: bool,
    pub totalcmd_path: String,
    pub text_editor: String,
    /// list of file extensions treated as text files; read from ini key
    /// `txtformat` (comma‑separated) or fall back to a hard‑coded default
    /// if the key is missing.
    pub txt_formats: Vec<String>,
    config_path: String,
    ini: Ini,
    pub need_save: bool,
}

/// 获取程序所在目录作为配置文件保存路径
fn get_exe_dir() -> String {
    env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_string_lossy().to_string()))
        .unwrap_or_else(|| ".".to_string())
}

impl NewFileConfig {
    pub fn new() -> Self {
        let config_path = format!("{}\\NewFile.ini", get_exe_dir());

        let mut ini = Ini::new();
        let mut clipboard = true;
        let mut open_after = true;
        let mut totalcmd_path = "C:\\python\\tools\\totalcmd\\TOTALCMD64.EXE".to_string();
        let mut text_editor = "notepad.exe".to_string();
        // default set of extensions for the txtformat key.  includes .ps1
        // and .bat per the new requirement.
        let default_txtformats = ".txt,.md,.ini,.py,.xml,.cs,.ps1,.bat";
        let mut txt_formats: Vec<String> = default_txtformats
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .map(|s| if s.starts_with('.') { s } else { format!(".{}", s) })
            .collect();

        if Path::new(&config_path).exists() {
            // Load from existing ini file
            if ini.load(&config_path).is_ok() {
                if let Some(val) = ini.get("Configuration", "autowriteclipboard") {
                    clipboard = val.to_lowercase() == "true";
                }
                if let Some(val) = ini.get("Configuration", "autoopenaftercreated") {
                    open_after = val.to_lowercase() == "true";
                }
                if let Some(val) = ini.get("Configuration", "totalcmd") {
                    if Path::new(&val).exists() {
                        totalcmd_path = val;
                    }
                }
                if let Some(val) = ini.get("Configuration", "texteditor") {
                    text_editor = val.trim().trim_matches('"').to_string();
                }

                // if annotation present, override defaults
                if let Some(val) = ini.get("Configuration", "txtformat") {
                    let parsed: Vec<String> = val
                        .split(',')
                        .map(|s| s.trim().to_lowercase())
                        .filter(|s| !s.is_empty())
                        .map(|s| if s.starts_with('.') { s } else { format!(".{}", s) })
                        .collect();
                    if !parsed.is_empty() {
                        txt_formats = parsed;
                    }
                }
                // make sure the key exists in the file so users see it
                if ini.get("Configuration", "txtformat").is_none() {
                    ini.set(
                        "Configuration",
                        "txtformat",
                        Some(default_txtformats.to_string()),
                    );
                    let _ = ini.write(&config_path);
                }
            }
        } else {
            // Create new ini file
            ini.set(
                "Configuration",
                "autowriteclipboard",
                Some(format!("{}", clipboard)),
            );
            ini.set(
                "Configuration",
                "autoopenaftercreated",
                Some(format!("{}", open_after)),
            );
            ini.set(
                "Configuration",
                "texteditor",
                Some(text_editor.clone()),
            );

            // Try to find TotalCMD
            // let possible_path = format!("{}\\..\\..\\..\\TOTALCMD64.EXE", current_path);
            let possible_path = format!("{}\\..\\..\\..\\TOTALCMD64.EXE", config_path);
            if Path::new(&possible_path).exists() {
                totalcmd_path = dunce::canonicalize(&possible_path).unwrap().display().to_string();
                ini.set("Configuration", "totalcmd", Some(totalcmd_path.clone()));
            }

            // ensure ini file contains the txtformat key with default value
            ini.set(
                "Configuration",
                "txtformat",
                Some(default_txtformats.to_string()),
            );
            let _ = ini.write(&config_path);
        }

        Self {
            clipboard,
            open_after,
            totalcmd_path,
            text_editor,
            txt_formats,
            config_path,
            ini,
            need_save: false,
        }
    }

    pub fn update(&mut self) {
        if self.need_save {
            self.ini.set(
                "Configuration",
                "autowriteclipboard",
                Some(format!("{}", self.clipboard)),
            );
            self.ini.set(
                "Configuration",
                "autoopenaftercreated",
                Some(format!("{}", self.open_after)),
            );
            self.ini.set(
                "Configuration",
                "texteditor",
                Some(self.text_editor.clone()),
            );
            // persist current txt_formats list as comma separated string
            let txt_joined = self.txt_formats.join(",");
            self.ini.set("Configuration", "txtformat", Some(txt_joined));
            let _ = self.ini.write(&self.config_path);
            self.need_save = false;
        }
    }

    pub fn set_clipboard(&mut self, value: bool) {
        self.clipboard = value;
        self.need_save = true;
        self.update();
    }

    pub fn set_open_after(&mut self, value: bool) {
        self.open_after = value;
        self.need_save = true;
        self.update();
    }
}
