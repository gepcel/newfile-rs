use configparser::ini::Ini;
use std::path::Path;
use dunce;

pub struct NewFileConfig {
    pub clipboard: bool,
    pub open_after: bool,
    pub totalcmd_path: String,
    pub text_editor: String,
    config_path: String,
    ini: Ini,
    pub need_save: bool,
}

impl NewFileConfig {
    pub fn new(current_path: String) -> Self {
        let config_path = format!("{}\\NewFile.ini", current_path);

        let mut ini = Ini::new();
        let mut clipboard = true;
        let mut open_after = true;
        let mut totalcmd_path = "C:\\python\\tools\\totalcmd\\TOTALCMD64.EXE".to_string();
        let mut text_editor = "notepad.exe".to_string();

        if Path::new(&config_path).exists() {
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
            }
        } else {
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

            let possible_path = format!("{}\\..\\..\\..\\TOTALCMD64.EXE", current_path);
            if Path::new(&possible_path).exists() {
                totalcmd_path = match dunce::canonicalize(&possible_path) {
                    Ok(p) => p.display().to_string(),
                    Err(_) => possible_path,
                };
                ini.set("Configuration", "totalcmd", Some(totalcmd_path.clone()));
            }

            let _ = ini.write(&config_path);
        }

        Self {
            clipboard,
            open_after,
            totalcmd_path,
            text_editor,
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
            if self.ini.write(&self.config_path).is_ok() {
                self.need_save = false;
            }
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