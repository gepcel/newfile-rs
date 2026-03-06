mod config;
mod file_ops;

use std::env;
use std::path::Path;
use std::ptr::null_mut;
use windows::w;
use windows::core::{PCWSTR, HSTRING};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::{BST_CHECKED};
use windows::Win32::UI::WindowsAndMessaging::{*, RegisterClassW, WNDCLASSW};
use windows::Win32::Graphics::Gdi::{UpdateWindow, GetStockObject, DEFAULT_GUI_FONT, COLOR_WINDOW, HBRUSH};
use windows::Win32::UI::WindowsAndMessaging::SetProcessDPIAware;

// numeric key codes (avoid importing VK_* names)
const VK_RETURN: u16 = 0x0D;
const VK_ESCAPE: u16 = 0x1B;
const VK_F1: u16 = 0x70;
const VK_F2: u16 = 0x71;

// manual key state functions; avoid extra features
extern "system" {
    fn GetKeyState(nVirtKey: i32) -> i16;
}

// control identifiers
const ID_EDIT: i32 = 101;
const ID_CREATE: i32 = 1001;
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
        return txt_formats.iter().any(|f| f == &format!(".{}", ext.to_lowercase()));
    }
    false
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
                    ID_FILE_BUTTON | ID_CREATE => {
                        let name = get_edit_text(state.hwnd_edit);
                        let mut is_file = get_file_type(&name, &state.config.txt_formats);
                        if GetKeyState(0x10) < 0 {
                            is_file = !is_file;
                        }
                        let ctrl = GetKeyState(0x11) < 0;
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
                        let checked = SendMessageW(state.hwnd_clip, BM_GETCHECK, WPARAM(0), LPARAM(0)).0 as i32 == BST_CHECKED.0 as i32;
                        state.config.set_clipboard(checked);
                    }
                    ID_TOGGLE_OPEN => {
                        let checked = SendMessageW(state.hwnd_open, BM_GETCHECK, WPARAM(0), LPARAM(0)).0 as i32 == BST_CHECKED.0 as i32;
                        state.config.set_open_after(checked);
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
    let hfont = GetStockObject(DEFAULT_GUI_FONT); // 获取系统字体
    // 创建静态文本并设置字体
    let label_hwnd = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        w!("STATIC"),
        w!("") ,
        WS_CHILD | WS_VISIBLE,
        5,
        5,
        530,
        20,
        hwnd,
        HMENU(0),
        hinst,
        Some(null_mut()),
    );
    if label_hwnd.0 != 0 {
        SetWindowTextW(label_hwnd, &HSTRING::from(&state.root_dir));
        SendMessageW(label_hwnd, WM_SETFONT, WPARAM(hfont.0 as usize), LPARAM(1));
    }
    // 创建编辑框并设置字体
    state.hwnd_edit = CreateWindowExW(
        WINDOW_EX_STYLE(WS_EX_CLIENTEDGE.0),
        w!("EDIT"),
        PCWSTR::null(),
        WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | (ES_AUTOHSCROLL as u32)),
        5,
        30,
        530,
        30,
        hwnd,
        HMENU(ID_EDIT as isize),
        hinst,
        None,
    );
    SendMessageW(state.hwnd_edit, WM_SETFONT, WPARAM(hfont.0 as usize), LPARAM(1));
    // 创建复选框 Write Clipboard 并设置字体
    state.hwnd_clip = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        w!("BUTTON"),
        w!("Write Clipboard (F1)"),
        WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | (BS_AUTOCHECKBOX as u32)),
        5,
        65,
        155,
        25,
        hwnd,
        HMENU(ID_TOGGLE_CLIP as isize),
        hinst,
        Some(null_mut()),
    );
    SendMessageW(state.hwnd_clip, BM_SETCHECK, WPARAM(state.config.clipboard as usize), LPARAM(0));
    SendMessageW(state.hwnd_clip, WM_SETFONT, WPARAM(hfont.0 as usize), LPARAM(1));
    // 创建复选框 Open After 并设置字体
    state.hwnd_open = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        w!("BUTTON"),
        w!("Open After (F2)"),
        WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | (BS_AUTOCHECKBOX as u32)),
        165,
        65,
        120,
        25,
        hwnd,
        HMENU(ID_TOGGLE_OPEN as isize),
        hinst,
        Some(null_mut()),
    );
    SendMessageW(state.hwnd_open, BM_SETCHECK, WPARAM(state.config.open_after as usize), LPARAM(0));
    SendMessageW(state.hwnd_open, WM_SETFONT, WPARAM(hfont.0 as usize), LPARAM(1));
    // 创建按钮 Folder (Enter) 并设置字体
    let hwnd_folder = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        w!("BUTTON"),
        w!("Folder (Enter)"),
        WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | (BS_PUSHBUTTON as u32)),
        290,
        65,
        110,
        25,
        hwnd,
        HMENU(ID_FOLDER_BUTTON as isize),
        hinst,
        Some(null_mut()),
    );
    SendMessageW(hwnd_folder, WM_SETFONT, WPARAM(hfont.0 as usize), LPARAM(1));
    // 创建按钮 File (Shift+Enter) 并设置字体
    let hwnd_file = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        w!("BUTTON"),
        w!("File (Shift+Enter)"),
        WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | (BS_PUSHBUTTON as u32)),
        405,
        65,
        120,
        25,
        hwnd,
        HMENU(ID_FILE_BUTTON as isize),
        hinst,
        Some(null_mut()),
    );
    SendMessageW(hwnd_file, WM_SETFONT, WPARAM(hfont.0 as usize), LPARAM(1));
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

    unsafe {
        let _ = SetProcessDPIAware();  // 忽略错误
        let hinst = GetModuleHandleW(None).unwrap();
        let class_name = w!("NewFileClass");
        let wc = WNDCLASSW {
            hInstance: hinst,
            lpszClassName: class_name,
            lpfnWndProc: Some(wndproc),
            hbrBackground: HBRUSH((COLOR_WINDOW.0 as isize) + 1),  // 设置背景刷
            ..Default::default()
        };
        RegisterClassW(&wc);

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            class_name,
            w!("NewFile"),
            WS_OVERLAPPEDWINDOW & !WS_THICKFRAME,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            550,
            130,
            HWND(0),
            HMENU(0),
            hinst,
            Some(null_mut()),
        );

        let state = Box::new(AppState {
            root_dir: root_dir.clone(),
            config: config::NewFileConfig::new(
                root_dir.clone(),
            ),
            hwnd_edit: HWND(0),
            hwnd_clip: HWND(0),
            hwnd_open: HWND(0),
        });
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(state) as isize);

        let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppState;
        create_controls(hwnd, &mut *state_ptr);
        
        // 为所有控件设置系统字体
        let hfont = GetStockObject(DEFAULT_GUI_FONT);
        // 遍历所有需要设置字体的控件句柄
        let controls = [
            (*state_ptr).hwnd_edit,
            (*state_ptr).hwnd_clip,
            (*state_ptr).hwnd_open,
            // 还有静态文本，但它的句柄在create_controls内部未保存，可以再获取或保存
            // 这里假设你保存了静态文本句柄，或者你可以通过FindWindowEx获取
        ];
        for &ctrl in &controls {
            SendMessageW(ctrl, WM_SETFONT, WPARAM(hfont.0 as usize), LPARAM(1));
        }
        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);

        let accels = [
            ACCEL { fVirt: FVIRTKEY, key: VK_RETURN as u16, cmd: ID_CREATE as u16 },
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
