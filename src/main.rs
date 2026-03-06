mod config;
mod file_ops;

use iced::keyboard::{self, KeyCode};
use iced::widget::{button, checkbox, container, text, text_input, Row, Column};
use iced::{executor, Application, Command, Element, Length, Settings, Subscription, Theme};
use iced_native::{self, Event, subscription as native_subscription};

use std::env;
use std::path::Path;

fn get_file_type(name: &str, txt_formats: &[String]) -> bool {
    if let Some(ext) = Path::new(name).extension().and_then(|e| e.to_str()) {
        return txt_formats.iter().any(|f| f == &format!(".{}", ext.to_lowercase()));
    }
    false
}

#[derive(Debug, Clone)]
enum Message {
    FilenameChanged(String),
    CreateFile,
    CreateFolder,
    ToggleClipboard(bool),
    ToggleOpenAfter(bool),
    KeyEvent(Event),
}

struct NewFileApp {
    root_dir: String,
    filename: String,
    // no explicit widget state needed with current iced API
    clipboard_checked: bool,
    open_checked: bool,
    config: config::NewFileConfig,
}

impl Application for NewFileApp {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = String;
    type Theme = Theme;

    // We can use the default theme implementation, so no need to override `theme` or `style`.

    fn new(flags: String) -> (Self, Command<Message>) {
        let root_dir = flags;

        let exe_path = env::current_exe().unwrap_or_default();
        let app_dir = exe_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_default();
        let cfg = config::NewFileConfig::new(app_dir.to_string_lossy().to_string());
        let clipboard = cfg.clipboard;
        let open = cfg.open_after;

        (
            Self {
                root_dir,
                filename: String::new(),
                // states are not required
                clipboard_checked: clipboard,
                open_checked: open,
                config: cfg,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        format!(
            "NewFile [{}{}]",
            if self.open_checked { "OpenAfter:ON " } else { "OpenAfter:OFF " },
            if self.clipboard_checked { "Clipboard:ON" } else { "Clipboard:OFF" }
        )
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::FilenameChanged(val) => {
                self.filename = val;
            }
            Message::CreateFile => {
                file_ops::create_new_file(
                    &self.filename,
                    self.clipboard_checked,
                    self.open_checked,
                    &self.root_dir,
                    &self.config.text_editor,
                );
                std::process::exit(0);
            }
            Message::CreateFolder => {
                file_ops::create_new_folder(
                    &self.filename,
                    self.open_checked,
                    &self.root_dir,
                    &self.config.totalcmd_path,
                );
                std::process::exit(0);
            }
            Message::ToggleClipboard(val) => {
                self.clipboard_checked = val;
                self.config.set_clipboard(val);
            }
            Message::ToggleOpenAfter(val) => {
                self.open_checked = val;
                self.config.set_open_after(val);
            }
            Message::KeyEvent(event) => {
                if let Event::Keyboard(key_event) = event {
                    if let keyboard::Event::KeyPressed { key_code, modifiers } = key_event {
                        match key_code {
                            KeyCode::Enter => {
                                let mut is_file = get_file_type(&self.filename, &self.config.txt_formats);
                                if modifiers.shift() {
                                    is_file = !is_file;
                                }
                                let ctrl = modifiers.control();
                                let open_after = if ctrl { !self.open_checked } else { self.open_checked };
                                if is_file {
                                    file_ops::create_new_file(
                                        &self.filename,
                                        self.clipboard_checked,
                                        open_after,
                                        &self.root_dir,
                                        &self.config.text_editor,
                                    );
                                } else {
                                    file_ops::create_new_folder(
                                        &self.filename,
                                        open_after,
                                        &self.root_dir,
                                        &self.config.totalcmd_path,
                                    );
                                }
                                std::process::exit(0);
                            }
                            KeyCode::Escape => {
                                std::process::exit(0);
                            }
                            KeyCode::F1 => {
                                self.clipboard_checked = !self.clipboard_checked;
                                self.config.set_clipboard(self.clipboard_checked);
                            }
                            KeyCode::F2 => {
                                self.open_checked = !self.open_checked;
                                self.config.set_open_after(self.open_checked);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        native_subscription::events().map(Message::KeyEvent)
    }

    fn view(&self) -> Element<'_, Message> {
        let label = text::Text::new(&self.root_dir).size(16);

        let text_input = text_input::TextInput::new(
            "File or folder name...",
            &self.filename,
        )
        .on_input(Message::FilenameChanged)
        .padding(5)
        .size(20);

        let checkbox_clip = checkbox::Checkbox::new(
            "Write Clipboard (F1)",
            self.clipboard_checked,
            Message::ToggleClipboard,
        );

        let checkbox_open = checkbox::Checkbox::new(
            "Open After (F2)",
            self.open_checked,
            Message::ToggleOpenAfter,
        );

        let btn_folder = button::Button::new(text::Text::new("Folder (Enter)"))
            .on_press(Message::CreateFolder);

        let btn_file = button::Button::new(text::Text::new("File (Shift+Enter)"))
            .on_press(Message::CreateFile);

        let row1 = Row::new().spacing(20).push(checkbox_clip).push(checkbox_open);
        let row2 = Row::new().spacing(20).push(btn_folder).push(btn_file);

        let content = Column::new()
            .spacing(10)
            .push(label)
            .push(text_input)
            .push(row1)
            .push(row2)
            .padding(20);

        container::Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let root_dir = if args.len() > 1 {
        args[1].clone()
    } else {
        std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default()
    };

    let settings = Settings {
        window: iced::window::Settings {
            size: (530, 130),
            ..iced::window::Settings::default()
        },
        flags: root_dir,
        ..Settings::default()
    };

    NewFileApp::run(settings).unwrap();
}
