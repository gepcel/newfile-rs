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
        eprintln!("[CONFIG] new() called with current_path = {}", current_path);
        let config_path = format!("{}\\NewFile.ini", current_path);
        eprintln!("[CONFIG] config_path = {}", config_path);

        let mut ini = Ini::new();
        let mut clipboard = true;
        let mut open_after = true;
        let mut totalcmd_path = "C:\\python\\tools\\totalcmd\\TOTALCMD64.EXE".to_string();
        let mut text_editor = "notepad.exe".to_string();

        if Path::new(&config_path).exists() {
            eprintln!("[CONFIG] Config file exists, loading...");
            if ini.load(&config_path).is_ok() {
                eprintln!("[CONFIG] Loaded INI successfully");
                if let Some(val) = ini.get("Configuration", "autowriteclipboard") {
                    clipboard = val.to_lowercase() == "true";
                    eprintln!("[CONFIG] autowriteclipboard = {} (loaded)", clipboard);
                }
                if let Some(val) = ini.get("Configuration", "autoopenaftercreated") {
                    open_after = val.to_lowercase() == "true";
                    eprintln!("[CONFIG] autoopenaftercreated = {} (loaded)", open_after);
                }
                if let Some(val) = ini.get("Configuration", "totalcmd") {
                    if Path::new(&val).exists() {
                        totalcmd_path = val;
                        eprintln!("[CONFIG] totalcmd = {} (loaded and exists)", totalcmd_path);
                    } else {
                        eprintln!("[CONFIG] totalcmd path from config does NOT exist: {}", val);
                    }
                }
                if let Some(val) = ini.get("Configuration", "texteditor") {
                    text_editor = val.trim().trim_matches('"').to_string();
                    eprintln!("[CONFIG] texteditor = {} (loaded)", text_editor);
                }
            } else {
                eprintln!("[CONFIG] Failed to load INI file");
            }
        } else {
            eprintln!("[CONFIG] Config file does NOT exist, creating new one");
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
            let possible_path = format!("{}\\..\\..\\..\\TOTALCMD64.EXE", current_path);
            eprintln!("[CONFIG] Trying to find TotalCMD at: {}", possible_path);
            if Path::new(&possible_path).exists() {
                eprintln!("[CONFIG] Found TotalCMD at {}", possible_path);
                // Modified: handle canonicalize error instead of unwrap
                totalcmd_path = match dunce::canonicalize(&possible_path) {
                    Ok(p) => {
                        let s = p.display().to_string();
                        eprintln!("[CONFIG] Canonicalized TotalCMD path = {}", s);
                        s
                    },
                    Err(e) => {
                        eprintln!("[CONFIG] Failed to canonicalize TotalCMD path: {}, using original", e);
                        possible_path
                    }
                };
                ini.set("Configuration", "totalcmd", Some(totalcmd_path.clone()));
            } else {
                eprintln!("[CONFIG] TotalCMD not found at {}", possible_path);
            }

            eprintln!("[CONFIG] Writing initial config file to {}", config_path);
            let write_result = ini.write(&config_path);
            if let Err(e) = write_result {
                eprintln!("[CONFIG] Failed to write initial config: {}", e);
            } else {
                eprintln!("[CONFIG] Initial config written successfully");
            }
        }

        eprintln!("[CONFIG] new() returning with clipboard={}, open_after={}, totalcmd_path={}, text_editor={}",
                  clipboard, open_after, totalcmd_path, text_editor);
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
        eprintln!("[CONFIG] update() called, need_save = {}", self.need_save);
        if self.need_save {
            eprintln!("[CONFIG] Saving configuration to {}", self.config_path);
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
            let write_result = self.ini.write(&self.config_path);
            if let Err(e) = write_result {
                eprintln!("[CONFIG] Failed to write config: {} (keeping need_save=true)", e);
                // keep need_save = true
            } else {
                eprintln!("[CONFIG] Config written successfully");
                self.need_save = false;
            }
        }
    }

    pub fn set_clipboard(&mut self, value: bool) {
        eprintln!("[CONFIG] set_clipboard({}) called", value);
        self.clipboard = value;
        self.need_save = true;
        self.update();
    }

    pub fn set_open_after(&mut self, value: bool) {
        eprintln!("[CONFIG] set_open_after({}) called", value);
        self.open_after = value;
        self.need_save = true;
        self.update();
    }
}