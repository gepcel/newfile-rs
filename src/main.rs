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


fn main() {
    #[cfg(target_os = "windows")]
    unsafe {
        FreeConsole();
    }

    let args: Vec<String> = env::args().collect();
    let root_dir = if args.len() > 1 {
        args[1].clone()
    } else {
        std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default()
    };

    let app = app::App::default();
    let wind = Rc::new(RefCell::new(window::Window::default()
        .with_size(530, 130)
        .with_label("NewFile")));

    let mut label = frame::Frame::default().with_size(520, 20).with_pos(5, 5);
    label.set_label(&root_dir);

    let txt_filename = Rc::new(RefCell::new(input::Input::default()
        .with_size(520, 30)
        .with_pos(5, 30)));

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

    // Set window icon
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            if let Some(parent) = parent.parent() {
                if let Some(parent) = parent.parent() {
                    let ico_path = parent.join("Autorun.ico");
                    if ico_path.exists() {
                        if let Ok(icon) = fltk::image::IcoImage::load(&ico_path) {
                            wind.borrow_mut().set_icon(Some(icon));
                        }
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

    let config = Rc::new(RefCell::new(config::NewFileConfig::new(
        app_dir.to_string_lossy().to_string(),
    )));

    // grab the list of text formats once; we don't expect it to change at
    // runtime (no UI to edit it) so we can clone out of the config and move
    // that into the various closures below.
    let initial_txt_formats: Vec<String> = {
        let cfg = config.borrow();
        cfg.txt_formats.clone()
    };

    // Set initial checkbox states
    {
        let cfg = config.borrow();
        check_clipboard.borrow_mut().set(cfg.clipboard);
        check_open.borrow_mut().set(cfg.open_after);
    }

    // Debug window title - shows configuration status
    let cfg = config.borrow();
    wind.borrow_mut().set_label(&format!("NewFile [{}{}]",
        if cfg.open_after { "OpenAfter:ON " } else { "OpenAfter:OFF " },
        if cfg.clipboard { "Clipboard:ON" } else { "Clipboard:OFF" }
    ));
    drop(cfg); // release borrow

    let (tx, rx) = fltk::app::channel::<Message>();

    // Input field event handler (Enter key)
    let tx_clone_for_input = tx.clone();
    let txt_formats_for_input: Vec<String> = initial_txt_formats.clone();
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
            tx.send(Message::Create(true, false));
        });
    }

    // Create folder button callback
    {
        let tx = tx.clone();
        btn_folder.set_callback(move |_| {
            tx.send(Message::Create(false, false));
        });
    }

    // Checkbox event handlers
    {
        let tx = tx.clone();
        check_clipboard.borrow_mut().set_callback(move |c| {
            let checked = c.is_checked();
            tx.send(Message::ToggleClipboard(checked));
        });
    }

    {
        let tx = tx.clone();
        check_open.borrow_mut().set_callback(move |c| {
            let checked = c.is_checked();
            tx.send(Message::ToggleOpenAfter(checked));
        });
    }

    // Window event handler (global keyboard shortcuts)
    let tx_clone = tx.clone();
    let txt_formats: Vec<String> = initial_txt_formats.clone();
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
                    tx_clone.send(Message::Create(is_file, ctrl_pressed));
                    true
                } else if key == enums::Key::Escape {
                    wind_clone.borrow_mut().hide();
                    true
                } else if key == enums::Key::F1 {
                    // Toggle and let callback handle the message
                    let current = {
                        check_clipboard_clone.borrow().is_checked()
                    };
                    let new_val = !current;
                    check_clipboard_clone.borrow_mut().set(new_val);
                    // The callback will send the ToggleClipboard message
                    true
                } else if key == enums::Key::F2 {
                    let current = {
                        check_open_clone.borrow().is_checked()
                    };
                    let new_val = !current;
                    check_open_clone.borrow_mut().set(new_val);
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    });

    // Main event loop
    while app.wait() {
        if let Some(msg) = rx.recv() {
            match msg {
                Message::Create(is_file, toggle_open_after) => {
                    let name = txt_filename.borrow().value();
                    let open_after = if toggle_open_after {
                        !check_open.borrow().is_checked()
                    } else {
                        check_open.borrow().is_checked()
                    };
                    if is_file {
                        let editor = config.borrow().text_editor.clone();
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
                        file_ops::create_new_folder(
                            &name,
                            open_after,
                            &root_dir,
                            &totalcmd,
                        );
                    }
                    wind.borrow_mut().hide();
                }
                Message::ToggleClipboard(checked) => {
                    config.borrow_mut().set_clipboard(checked);
                }
                Message::ToggleOpenAfter(checked) => {
                    config.borrow_mut().set_open_after(checked);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Create(bool, bool), // is_file, toggle_open_after
    ToggleClipboard(bool),
    ToggleOpenAfter(bool),
}
