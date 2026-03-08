# NewFile for Total Commander

A lightweight utility for **Total Commander** to quickly create new files and folders with a simple hotkey.
It provides a small GUI window where you enter a name, optionally write clipboard content into new files, and automatically open the created item in your preferred editor or Total Commander itself.

This is a Rust rewrite of the [C# version](https://github.com/gepcel/totalcmd-newfile)

## Project Overview

NewFile-rs is a lightweight file/folder creation tool designed specifically for Total Commander. With a simple GUI and keyboard shortcuts, you can quickly create new files or folders in the current directory, with support for automatic clipboard content writing and auto-opening created files.

This project has **4 branches**, each using a different GUI approach:

| Branch | GUI Framework | Description | Link |
|--------|---------------|-------------|------|
| **main** | FLTK | Default branch, lightweight cross-platform GUI | [View Branch](https://github.com/gepcel/newfile-rs/tree/main) |
| **win10** | Native Win32 API | Optimized for Windows 10, native look and feel | [View Branch](https://github.com/gepcel/newfile-rs/tree/win10) |
| **win32** | Native Win32 API | Pure Win32 API implementation for all Windows versions | [View Branch](https://github.com/gepcel/newfile-rs/tree/win32) |
| **iced** | Iced | Modern cross-platform GUI with Elm-inspired architecture | [View Branch](https://github.com/gepcel/newfile-rs/tree/iced) |

> **💡 Highly Recommended**: For most users, we recommend the **win10 branch** (best native Windows experience) or **main branch** (lightweight and stable). Both branches are well-tested and offer the best stability.

## Branch Comparison

### 🏆 win10 Branch — Native Win32 API
- **GUI Framework**: Raw Windows API via `windows` crate
- **Pros**: 
  - Native Windows look and feel
  - Minimal dependencies, smallest binary size
  - Best performance on Windows 10+
  - No external runtime required
- **Cons**: Windows only
- **Best for**: Windows 10 users who want the most native experience

### 🏆 main Branch — FLTK
- **GUI Framework**: FLTK (Fast Light Toolkit)
- **Pros**: 
  - Lightweight and fast
  - Cross-platform (Windows/Linux/macOS)
  - Stable and mature
  - Small binary size
- **Cons**: Less "native" look compared to Win32 API
- **Best for**: Users who prefer a stable, cross-platform solution

### win32 Branch — Native Win32 API
- **GUI Framework**: Raw Windows API via `windows` crate
- **Pros**: 
  - Pure Win32 implementation
  - Compatible with all Windows versions (7/8/10/11)
  - Minimal dependencies
- **Cons**: Windows only, less optimized than win10 branch
- **Best for**: Users on older Windows versions (7/8)

### iced Branch — Iced Framework
- **GUI Framework**: Iced (Elm-inspired, cross-platform)
- **Pros**: 
  - Modern, reactive UI architecture
  - Cross-platform support
  - Clean and maintainable code
- **Cons**: 
  - Larger binary size
  - Higher resource usage
  - Experimental for this project
- **Best for**: Developers interested in modern Rust GUI patterns

## Features

- **Create files or folders** by typing a name (supports nested paths like `foo/bar/file.txt`).
- **Smart file type detection**: based on extension (e.g., `.txt`, `.md`, `.py`), the Enter key creates a file; otherwise a folder. Use `Shift+Enter` to force the opposite type.
- **Write clipboard content** automatically into newly created text files.
- **Open after creation**:
  - For files: open with your configured text editor (default `notepad.exe`).
  - For folders: open the target folder in Total Commander (if found).
- **Configuration stored** in `NewFile.ini` (next to the executable) – remembers your checkbox preferences and custom paths.
- **Keyboard shortcuts**:
  - `F1` – toggle clipboard writing
  - `F2` – toggle open-after-creation
  - `Enter` – create (file/folder according to extension)
  - `Shift+Enter` – force opposite type (file ↔ folder)
  - `Ctrl+Enter` – switch the `open-after-creation` temperary when creating file or folder
  - `Escape` – close window without creating
  - **Total Commander integration**: run the program from a button or hotkey, passing the target directory as argument.

## Download

Get the latest pre-built executable from the **[Releases](https://github.com/yourusername/newfile-rs/releases)** page.  
Just download `NewFile.exe` and place it anywhere you like.

## How to Use

### Basic Usage

1. Launch `NewFile.exe`. You can optionally pass a directory path as a command‑line argument – the program will use that as the root for new items. If no argument is given, it uses the current working directory.
2. A small window appears:
   - The top line shows the target directory.
   - Enter a name. You can include relative paths, e.g. `docs/notes.txt` or `images/screenshots`.
   - Check or uncheck the two options (or use `F1` / `F2`).
   - Press `Enter` (or click **File**/**Folder**) to create.
3. After creation the window closes automatically. If you checked **Open After**, the file/folder will be opened.

### Integration with Total Commander

Add a button to Total Commander's toolbar or a shortcut key that calls `NewFile.exe` with the current directory as parameter. The default directory to place the NewFile.exe is suggested as `%COMMANDER_PATH%\plugins\addons\NewFile.exe`

- **Command**: `%COMMANDER_PATH%\plugins\addons\NewFile.exe`
- **Parameters**: `%P` (this passes the current panel's directory)

Now whenever you click that button or the shortcut key, the NewFile window opens in the current Total Commander folder.

## Configuration

All settings are stored in `NewFile.ini` located in the same directory as the executable.  
If the file does not exist, it is created with default values.

### Options

- `autowriteclipboard` – `true`/`false` (default `true`)
- `autoopenaftercreated` – `true`/`false` (default `true`)
- `texteditor` – path to your preferred text editor (default `notepad.exe`). The editor is used when opening files after creation.
- `totalcmd` – path to `TOTALCMD64.EXE`. On first run the program tries to locate it automatically (looking three levels up from its own location). If that fails, you can set the correct path manually in the ini file.
- `txtformat`: Comma-separated list of text file extensions. If absent the program falls back to a built-in default (`.txt,.md,.ini,.py,.xml,.cs,.ps1,.bat`).

Example `NewFile.ini` (added `txtformat` key to control which extensions are considered text files):

```ini
[Configuration]
autowriteclipboard = true
autoopenaftercreated = true
texteditor = "C:\Program Files\Notepad++\notepad++.exe"
totalcmd = C:\totalcmd\TOTALCMD64.EXE
txtformat = .txt,.md,.ini,.py,.xml,.cs,.ps1,.bat
```

## Building from Source

If you prefer to compile the program yourself:

1. Ensure you have [Rust](https://rustup.rs/) installed.
2. Clone the repository:

  ```bash
  git clone https://github.com/yourusername/newfile-rs.git
  cd newfile-rs
  ```

3. Build the release version:
  ```bash
  cargo build --release
  # or
  ./build.ps1
  ```

4. The executable will be at `target/release/NewFile.exe` (or newfile on Linux, but the program is primarily intended for Windows).

# Requirements

Windows (tested on 7/10/11). The code contains conditional compilation for Windows, but it may work on Linux with minor adjustments.

Total Commander (optional) – if you want folder opening to work.

# License
This project is licensed under the MIT License. See the [LICENSE](https://license/) file for details.

------

**Enjoy fast file creation with Total Commander!**

If you encounter any issues, please open an issue.
