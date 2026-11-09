mod config;
mod file_ops;

use std::env;
use std::fs;
use std::path::Path;
use std::ptr::null_mut;
use windows::w;
use windows::core::{PCWSTR, HSTRING};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::{BST_CHECKED};
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Graphics::Gdi::{UpdateWindow, COLOR_WINDOW, HBRUSH, CreateFontW, FW_NORMAL, DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, DEFAULT_QUALITY, DEFAULT_PITCH, FF_SWISS};
use windows::Win32::UI::Input::KeyboardAndMouse::SetFocus;
 
// 虚拟键码
const VK_RETURN: u16 = 0x0D;
const VK_ESCAPE: u16 = 0x1B;
const VK_F1: u16 = 0x70;
const VK_F2: u16 = 0x71;

extern "system" {
    fn GetKeyState(nVirtKey: i32) -> i16;
}

// 控件标识符
const ID_EDIT: i32 = 101;
const ID_CREATE: i32 = 1001;          // 普通回车
const ID_CREATE_SHIFT: i32 = 1007;    // Shift+回车
const ID_CREATE_CTRL: i32 = 1008;     // Ctrl+回车
const ID_TOGGLE_CLIP: i32 = 1002;
const ID_TOGGLE_OPEN: i32 = 1003;
const ID_CANCEL: i32 = 1004;
const ID_FILE_BUTTON: i32 = 1005;
const ID_FOLDER_BUTTON: i32 = 1006;

struct AppState {
    root_dir: String,
    config: config::NewFileConfig,
    hwnd_edit: HWND,
    hwnd_clip: HWND,
    hwnd_open: HWND,
}

fn get_file_type(name: &str, txt_formats: &[String]) -> bool {
    if let Some(ext) = Path::new(name).extension().and_then(|e| e.to_str()) {
        txt_formats.iter().any(|f| f == &format!(".{}", ext.to_lowercase()))
    } else {
        false
    }
}

unsafe fn get_edit_text(hwnd: HWND) -> String {
    let mut buf = [0u16; 512];
    let len = GetWindowTextW(hwnd, &mut buf) as usize;
    String::from_utf16_lossy(&buf[..len])
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppState;
    if !state_ptr.is_null() {
        let state = &mut *state_ptr;
        match msg {
            WM_COMMAND => {
                let id = ((wparam.0 as u32) & 0xFFFF) as i32;
                match id {
                    ID_CREATE => {
                        let name = get_edit_text(state.hwnd_edit);
                        let mut is_file = get_file_type(&name, &state.config.txt_formats);
                        // Shift 按下时反转类型
                        if GetKeyState(0x10) < 0 { // VK_SHIFT
                            is_file = !is_file;
                        }
                        // Ctrl 按下时反转“打开后”选项
                        let ctrl = GetKeyState(0x11) < 0; // VK_CONTROL
                        let open_after = if ctrl {
                            !(SendMessageW(state.hwnd_open, BM_GETCHECK, WPARAM(0), LPARAM(0)).0 as i32 == BST_CHECKED.0 as i32)
                        } else {
                            SendMessageW(state.hwnd_open, BM_GETCHECK, WPARAM(0), LPARAM(0)).0 as i32 == BST_CHECKED.0 as i32
                        };
                        if is_file {
                            file_ops::create_new_file(
                                &name,
                                SendMessageW(state.hwnd_clip, BM_GETCHECK, WPARAM(0), LPARAM(0)).0 as i32 == BST_CHECKED.0 as i32,
                                open_after,
                                &state.root_dir,
                                &state.config.text_editor,
                            );
                        } else {
                            file_ops::create_new_folder(
                                &name,
                                open_after,
                                &state.root_dir,
                                &state.config.totalcmd_path,
                            );
                        }
                        PostQuitMessage(0);
                    }
                    ID_CREATE_SHIFT => {
                        // Shift+Enter：强制反转类型，不改变打开选项
                        let name = get_edit_text(state.hwnd_edit);
                        let is_file = !get_file_type(&name, &state.config.txt_formats);
                        let open_after = SendMessageW(state.hwnd_open, BM_GETCHECK, WPARAM(0), LPARAM(0)).0 as i32 == BST_CHECKED.0 as i32;
                        if is_file {
                            file_ops::create_new_file(
                                &name,
                                SendMessageW(state.hwnd_clip, BM_GETCHECK, WPARAM(0), LPARAM(0)).0 as i32 == BST_CHECKED.0 as i32,
                                open_after,
                                &state.root_dir,
                                &state.config.text_editor,
                            );
                        } else {
                            file_ops::create_new_folder(
                                &name,
                                open_after,
                                &state.root_dir,
                                &state.config.totalcmd_path,
                            );
                        }
                        PostQuitMessage(0);
                    }
                    ID_CREATE_CTRL => {
                        // Ctrl+Enter：强制反转“打开后”选项，不改变类型
                        let name = get_edit_text(state.hwnd_edit);
                        let is_file = get_file_type(&name, &state.config.txt_formats);
                        let open_after = !(SendMessageW(state.hwnd_open, BM_GETCHECK, WPARAM(0), LPARAM(0)).0 as i32 == BST_CHECKED.0 as i32);
                        if is_file {
                            file_ops::create_new_file(
                                &name,
                                SendMessageW(state.hwnd_clip, BM_GETCHECK, WPARAM(0), LPARAM(0)).0 as i32 == BST_CHECKED.0 as i32,
                                open_after,
                                &state.root_dir,
                                &state.config.text_editor,
                            );
                        } else {
                            file_ops::create_new_folder(
                                &name,
                                open_after,
                                &state.root_dir,
                                &state.config.totalcmd_path,
                            );
                        }
                        PostQuitMessage(0);
                    }
                    ID_FILE_BUTTON => {
                        let name = get_edit_text(state.hwnd_edit);
                        file_ops::create_new_file(
                            &name,
                            SendMessageW(state.hwnd_clip, BM_GETCHECK, WPARAM(0), LPARAM(0)).0 as i32 == BST_CHECKED.0 as i32,
                            SendMessageW(state.hwnd_open, BM_GETCHECK, WPARAM(0), LPARAM(0)).0 as i32 == BST_CHECKED.0 as i32,
                            &state.root_dir,
                            &state.config.text_editor,
                        );
                        PostQuitMessage(0);
                    }
                    ID_FOLDER_BUTTON => {
                        let name = get_edit_text(state.hwnd_edit);
                        file_ops::create_new_folder(
                            &name,
                            SendMessageW(state.hwnd_open, BM_GETCHECK, WPARAM(0), LPARAM(0)).0 as i32 == BST_CHECKED.0 as i32,
                            &state.root_dir,
                            &state.config.totalcmd_path,
                        );
                        PostQuitMessage(0);
                    }
                    ID_TOGGLE_CLIP => {
                        let from_keyboard = lparam.0 == 0; // 加速键产生时 lParam 为 0
                        let current = SendMessageW(state.hwnd_clip, BM_GETCHECK, WPARAM(0), LPARAM(0)).0 as i32 == BST_CHECKED.0 as i32;
                        let new_state = if from_keyboard { !current } else { current };
                        if from_keyboard {
                            // 快捷键需要手动更新控件状态
                            SendMessageW(state.hwnd_clip, BM_SETCHECK, WPARAM(new_state as usize), LPARAM(0));
                        }
                        state.config.set_clipboard(new_state);
                    }
                    ID_TOGGLE_OPEN => {
                        let from_keyboard = lparam.0 == 0;
                        let current = SendMessageW(state.hwnd_open, BM_GETCHECK, WPARAM(0), LPARAM(0)).0 as i32 == BST_CHECKED.0 as i32;
                        let new_state = if from_keyboard { !current } else { current };
                        if from_keyboard {
                            SendMessageW(state.hwnd_open, BM_SETCHECK, WPARAM(new_state as usize), LPARAM(0));
                        }
                        state.config.set_open_after(new_state);
                    } 
                    ID_CANCEL => {
                        PostQuitMessage(0);
                    }
                    _ => {}
                }
            }
            WM_DESTROY => {
                PostQuitMessage(0);
            }
            _ => return DefWindowProcW(hwnd, msg, wparam, lparam),
        }
        return LRESULT(0);
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

unsafe fn create_controls(hwnd: HWND, state: &mut AppState) {
    let hinst = GetModuleHandleW(None).unwrap();
    let hfont = CreateFontW(-16, 0, 0, 0, FW_NORMAL.0 as i32, 0, 0, 0, DEFAULT_CHARSET.0 as u32, OUT_DEFAULT_PRECIS.0 as u32, CLIP_DEFAULT_PRECIS.0 as u32, DEFAULT_QUALITY.0 as u32, DEFAULT_PITCH.0 as u32 | FF_SWISS.0 as u32, w!("Segoe UI"));

    // 静态文本显示当前路径
    let label_hwnd = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        w!("STATIC"),
        w!(""),
        WS_CHILD | WS_VISIBLE,
        5, 5, 560, 24,
        hwnd,
        HMENU(0),
        hinst,
        Some(null_mut()),
    );
    if label_hwnd.0 != 0 {
        SetWindowTextW(label_hwnd, &HSTRING::from(&state.root_dir));
        SendMessageW(label_hwnd, WM_SETFONT, WPARAM(hfont.0 as usize), LPARAM(1));
    }

    // 编辑框
    state.hwnd_edit = CreateWindowExW(
        WINDOW_EX_STYLE(WS_EX_CLIENTEDGE.0),
        w!("EDIT"),
        PCWSTR::null(),
        WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | (ES_AUTOHSCROLL as u32)),
        5, 30, 560, 32,
        hwnd,
        HMENU(ID_EDIT as isize),
        hinst,
        None,
    );
    SendMessageW(state.hwnd_edit, WM_SETFONT, WPARAM(hfont.0 as usize), LPARAM(1));

    // 复选框：Write Clipboard (F1)
    state.hwnd_clip = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        w!("BUTTON"),
        w!("Write Clipboard (F1)"),
        WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | (BS_AUTOCHECKBOX as u32)),
        5, 68, 165, 28,
        hwnd,
        HMENU(ID_TOGGLE_CLIP as isize),
        hinst,
        Some(null_mut()),
    );
    SendMessageW(state.hwnd_clip, BM_SETCHECK, WPARAM(state.config.clipboard as usize), LPARAM(0));
    SendMessageW(state.hwnd_clip, WM_SETFONT, WPARAM(hfont.0 as usize), LPARAM(1));

    // 复选框：Open After (F2)
    state.hwnd_open = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        w!("BUTTON"),
        w!("Open After (F2)"),
        WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | (BS_AUTOCHECKBOX as u32)),
        175, 68, 130, 28,
        hwnd,
        HMENU(ID_TOGGLE_OPEN as isize),
        hinst,
        Some(null_mut()),
    );
    SendMessageW(state.hwnd_open, BM_SETCHECK, WPARAM(state.config.open_after as usize), LPARAM(0));
    SendMessageW(state.hwnd_open, WM_SETFONT, WPARAM(hfont.0 as usize), LPARAM(1));

    // 按钮：Folder (Enter)
    let hwnd_folder = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        w!("BUTTON"),
        w!("Folder (Enter)"),
        WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | (BS_PUSHBUTTON as u32)),
        310, 68, 115, 28,
        hwnd,
        HMENU(ID_FOLDER_BUTTON as isize),
        hinst,
        Some(null_mut()),
    );
    SendMessageW(hwnd_folder, WM_SETFONT, WPARAM(hfont.0 as usize), LPARAM(1));

    // 按钮：File (Shift+Enter)
    let hwnd_file = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        w!("BUTTON"),
        w!("File (Shift+Enter)"),
        WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | (BS_PUSHBUTTON as u32)),
        430, 68, 130, 28,
        hwnd,
        HMENU(ID_FILE_BUTTON as isize),
        hinst,
        Some(null_mut()),
    );
    SendMessageW(hwnd_file, WM_SETFONT, WPARAM(hfont.0 as usize), LPARAM(1));
}

fn main() {
    let current = std::env::current_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
    let mut args = env::args().skip(1); // skip args[0]
    let root_dir = match(args.next(), args.next()) {
        (Some(p), None) if fs::metadata(&p).is_ok_and(|m| m.is_dir()) => p, // if args[1] is valid and len(args)==2
        _ => current,
    };

    unsafe {
        let _ = SetProcessDPIAware();
        let hinst = GetModuleHandleW(None).unwrap();
        let class_name = w!("NewFileClass");

        // 从嵌入的资源加载图标（ID = 1）
        let hicon = LoadIconW(hinst, PCWSTR(1 as *const u16)).unwrap_or_default();
        let hcursor = LoadCursorW(None, IDC_ARROW).unwrap_or_default();

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            hInstance: hinst,
            lpszClassName: class_name,
            lpfnWndProc: Some(wndproc),
            hbrBackground: HBRUSH((COLOR_WINDOW.0 as isize) + 1),
            hIcon: hicon,
            hIconSm: hicon,
            hCursor: hcursor,
            ..Default::default()
        };
        RegisterClassExW(&wc);

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            class_name,
            w!("NewFile"),
            WS_OVERLAPPEDWINDOW & !WS_THICKFRAME,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            585,
            145,
            HWND(0),
            HMENU(0),
            hinst,
            Some(null_mut()),
        );

        let state = Box::new(AppState {
            root_dir: root_dir.clone(),
            config: config::NewFileConfig::new(),
            hwnd_edit: HWND(0),
            hwnd_clip: HWND(0),
            hwnd_open: HWND(0),
        });
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(state) as isize);

        let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppState;
        create_controls(hwnd, &mut *state_ptr);

        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);

        // 自动聚焦到编辑框
        SetFocus((*state_ptr).hwnd_edit);

        // 加速键表：普通回车、Shift+回车、Ctrl+回车、Esc、F1、F2
        let accels = [
            ACCEL { fVirt: FVIRTKEY, key: VK_RETURN as u16, cmd: ID_CREATE as u16 },
            ACCEL { fVirt: FVIRTKEY | FSHIFT, key: VK_RETURN as u16, cmd: ID_CREATE_SHIFT as u16 },
            ACCEL { fVirt: FVIRTKEY | FCONTROL, key: VK_RETURN as u16, cmd: ID_CREATE_CTRL as u16 },
            ACCEL { fVirt: FVIRTKEY, key: VK_ESCAPE as u16, cmd: ID_CANCEL as u16 },
            ACCEL { fVirt: FVIRTKEY, key: VK_F1 as u16, cmd: ID_TOGGLE_CLIP as u16 },
            ACCEL { fVirt: FVIRTKEY, key: VK_F2 as u16, cmd: ID_TOGGLE_OPEN as u16 },
        ];
        let haccel = CreateAcceleratorTableW(&accels).unwrap();

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND(0), 0, 0).into() {
            if TranslateAcceleratorW(hwnd, haccel, &msg) == 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}