mod config;
mod file_ops;

use fltk::{prelude::*, *};
use std::cell::RefCell;
use std::env;
use std::path::Path;
use std::rc::Rc;

#[cfg(target_os = "windows")]
extern "C" {
    fn FreeConsole() -> i32;
}

fn get_file_type(name: &str, txt_formats: &[String]) -> bool {
    if let Some(ext) = Path::new(name).extension().and_then(|e| e.to_str()) {
        return txt_formats.iter().any(|f| f == &format!(".{}", ext.to_lowercase()));
    }
    false
}

const TXT_FORMATS: &[&str] = &[".txt", ".md", ".ini", ".py", ".xml", ".cs"];

fn main() {
    eprintln!("[MAIN] Program started");
    #[cfg(target_os = "windows")]
    unsafe { 
        eprintln!("[MAIN] Calling FreeConsole()");
        FreeConsole(); 
    }

    let args: Vec<String> = env::args().collect();
    eprintln!("[MAIN] Command line args: {:?}", args);
    let root_dir = if args.len() > 1 {
        args[1].clone()
    } else {
        std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default()
    };
    eprintln!("[MAIN] root_dir = {}", root_dir);

    let app = app::App::default();
    let wind = Rc::new(RefCell::new(window::Window::default()
        .with_size(530, 130)
        .with_label("NewFile")));
    eprintln!("[MAIN] Window created");

    let mut label = frame::Frame::default().with_size(520, 20).with_pos(5, 5);
    label.set_label(&root_dir);

    let txt_filename = Rc::new(RefCell::new(input::Input::default()
        .with_size(520, 30)
        .with_pos(5, 30)));
    eprintln!("[MAIN] Input field created");

    let check_clipboard = Rc::new(RefCell::new(button::CheckButton::default()
        .with_size(200, 25)
        .with_pos(5, 65)
        .with_label("Write Clipboard (F1)")));

    let check_open = Rc::new(RefCell::new(button::CheckButton::default()
        .with_size(200, 25)
        .with_pos(220, 65)
        .with_label("Open After (F2)")));

    let mut btn_folder = button::Button::default()
        .with_size(120, 25)
        .with_pos(280, 100)
        .with_label("Folder (Enter)");

    let mut btn_file = button::Button::default()
        .with_size(120, 25)
        .with_pos(405, 100)
        .with_label("File (Shift+Enter)");

    wind.borrow_mut().end();
    wind.borrow_mut().show();
    eprintln!("[MAIN] Window shown");

    // Set window icon
    if let Ok(exe_path) = std::env::current_exe() {
        eprintln!("[MAIN] exe_path = {:?}", exe_path);
        if let Some(parent) = exe_path.parent() {
            if let Some(parent) = parent.parent() {
                if let Some(parent) = parent.parent() {
                    let ico_path = parent.join("Autorun.ico");
                    eprintln!("[MAIN] Trying icon at: {:?}", ico_path);
                    if ico_path.exists() {
                        if let Ok(icon) = fltk::image::IcoImage::load(&ico_path) {
                            wind.borrow_mut().set_icon(Some(icon));
                            eprintln!("[MAIN] Icon set");
                        } else {
                            eprintln!("[MAIN] Failed to load icon");
                        }
                    } else {
                        eprintln!("[MAIN] Icon file not found");
                    }
                }
            }
        }
    }

    // Load config
    let app_dir = std::env::current_exe()
        .ok()
        .and_then(|pb| pb.parent().map(|p| p.to_path_buf()))
        .unwrap_or_default();
    eprintln!("[MAIN] app_dir = {:?}", app_dir);

    let config = Rc::new(RefCell::new(config::NewFileConfig::new(
        app_dir.to_string_lossy().to_string(),
    )));

    // Set initial checkbox states
    {
        let cfg = config.borrow();
        check_clipboard.borrow_mut().set(cfg.clipboard);
        check_open.borrow_mut().set(cfg.open_after);
        eprintln!("[MAIN] Initial config loaded: clipboard={}, open_after={}, totalcmd_path={}, text_editor={}",
                  cfg.clipboard, cfg.open_after, cfg.totalcmd_path, cfg.text_editor);
    }

    // Debug window title - shows configuration status
    let cfg = config.borrow();
    wind.borrow_mut().set_label(&format!("NewFile [{}{}]",
        if cfg.open_after { "OpenAfter:ON " } else { "OpenAfter:OFF " },
        if cfg.clipboard { "Clipboard:ON" } else { "Clipboard:OFF" }
    ));
    drop(cfg); // release borrow

    let (tx, rx) = fltk::app::channel::<Message>();
    eprintln!("[MAIN] Channel created");

    // Input field event handler (Enter key)
    let tx_clone_for_input = tx.clone();
    let txt_formats_for_input: Vec<String> = TXT_FORMATS.iter().map(|s| s.to_string()).collect();
    let txt_filename_clone_for_input = txt_filename.clone();

    txt_filename.borrow_mut().handle(move |_, event| {
        match event {
            enums::Event::KeyDown => {
                let key = app::event_key();
                if key == enums::Key::Enter {
                    let state = app::event_state();
                    let ctrl_pressed = state.contains(enums::EventState::Ctrl);
                    let shift_pressed = state.contains(enums::EventState::Shift);
                    let name = txt_filename_clone_for_input.borrow().value();
                    let mut is_file = get_file_type(&name, &txt_formats_for_input);
                    if shift_pressed {
                        is_file = !is_file;
                    }
                    eprintln!("[MAIN] Input field Enter: name='{}', is_file={}, ctrl={}, shift={}", name, is_file, ctrl_pressed, shift_pressed);
                    tx_clone_for_input.send(Message::Create(is_file, ctrl_pressed));
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    });

    // Create file button callback
    {
        let tx = tx.clone();
        btn_file.set_callback(move |_| {
            eprintln!("[MAIN] File button clicked");
            tx.send(Message::Create(true, false));
        });
    }

    // Create folder button callback
    {
        let tx = tx.clone();
        btn_folder.set_callback(move |_| {
            eprintln!("[MAIN] Folder button clicked");
            tx.send(Message::Create(false, false));
        });
    }

    // Checkbox event handlers
    {
        let tx = tx.clone();
        check_clipboard.borrow_mut().set_callback(move |c| {
            let checked = c.is_checked();
            eprintln!("[MAIN] Clipboard checkbox toggled: {}", checked);
            tx.send(Message::ToggleClipboard(checked));
        });
    }

    {
        let tx = tx.clone();
        check_open.borrow_mut().set_callback(move |c| {
            let checked = c.is_checked();
            eprintln!("[MAIN] OpenAfter checkbox toggled: {}", checked);
            tx.send(Message::ToggleOpenAfter(checked));
        });
    }

    // Window event handler (global keyboard shortcuts)
    let tx_clone = tx.clone();
    let txt_formats: Vec<String> = TXT_FORMATS.iter().map(|s| s.to_string()).collect();
    let wind_clone = wind.clone();
    let txt_filename_clone = txt_filename.clone();
    let check_clipboard_clone = check_clipboard.clone();
    let check_open_clone = check_open.clone();

    wind.borrow_mut().handle(move |_, event| {
        match event {
            enums::Event::KeyDown => {
                let key = app::event_key();
                let state = app::event_state();
                let ctrl_pressed = state.contains(enums::EventState::Ctrl);
                let shift_pressed = state.contains(enums::EventState::Shift);
                let name = txt_filename_clone.borrow().value();
                let mut is_file = get_file_type(&name, &txt_formats);
                if shift_pressed {
                    is_file = !is_file;
                }
                if key == enums::Key::Enter {
                    eprintln!("[MAIN] Global Enter: name='{}', is_file={}, ctrl={}, shift={}", name, is_file, ctrl_pressed, shift_pressed);
                    tx_clone.send(Message::Create(is_file, ctrl_pressed));
                    true
                } else if key == enums::Key::Escape {
                    eprintln!("[MAIN] Escape pressed, hiding window");
                    wind_clone.borrow_mut().hide();
                    true
                } else if key == enums::Key::F1 {
                    eprintln!("[MAIN] F1 pressed (toggle clipboard)");
                    // Toggle and let callback handle the message
                    let current = {
                        check_clipboard_clone.borrow().is_checked()
                    };
                    let new_val = !current;
                    eprintln!("[MAIN] Toggling clipboard from {} to {}", current, new_val);
                    check_clipboard_clone.borrow_mut().set(new_val);
                    // The callback will send the ToggleClipboard message
                    true
                } else if key == enums::Key::F2 {
                    eprintln!("[MAIN] F2 pressed (toggle open after)");
                    let current = {
                        check_open_clone.borrow().is_checked()
                    };
                    let new_val = !current;
                    eprintln!("[MAIN] Toggling open_after from {} to {}", current, new_val);
                    check_open_clone.borrow_mut().set(new_val);
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    });

    eprintln!("[MAIN] Entering event loop");
    // Main event loop
    while app.wait() {
        if let Some(msg) = rx.recv() {
            eprintln!("[MAIN] Received message: {:?}", msg);
            match msg {
                Message::Create(is_file, toggle_open_after) => {
                    let name = txt_filename.borrow().value();
                    let open_after = if toggle_open_after {
                        !check_open.borrow().is_checked()
                    } else {
                        check_open.borrow().is_checked()
                    };
                    eprintln!("[MAIN] Create message: is_file={}, toggle_open_after={}, name='{}', open_after={}",
                              is_file, toggle_open_after, name, open_after);
                    if is_file {
                        let editor = config.borrow().text_editor.clone();
                        eprintln!("[MAIN] Calling create_new_file with editor_path='{}'", editor);
                        file_ops::create_new_file(
                            &name,
                            check_clipboard.borrow().is_checked(),
                            open_after,
                            &root_dir,
                            &editor,
                        );
                    } else {
                        let totalcmd = {
                            let cfg = config.borrow();
                            cfg.totalcmd_path.clone()
                        };
                        eprintln!("[MAIN] Calling create_new_folder with totalcmd_path='{}'", totalcmd);
                        file_ops::create_new_folder(
                            &name,
                            open_after,
                            &root_dir,
                            &totalcmd,
                        );
                    }
                    eprintln!("[MAIN] Hiding window after creation");
                    wind.borrow_mut().hide();
                }
                Message::ToggleClipboard(checked) => {
                    eprintln!("[MAIN] ToggleClipboard message: checked={}", checked);
                    config.borrow_mut().set_clipboard(checked);
                }
                Message::ToggleOpenAfter(checked) => {
                    eprintln!("[MAIN] ToggleOpenAfter message: checked={}", checked);
                    config.borrow_mut().set_open_after(checked);
                }
            }
        }
    }
    eprintln!("[MAIN] Event loop ended, exiting");
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Create(bool, bool), // is_file, toggle_open_after
    ToggleClipboard(bool),
    ToggleOpenAfter(bool),
}