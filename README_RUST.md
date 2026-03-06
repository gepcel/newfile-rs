# NewFile - Rust GUI Application

A Rust rewrite of the C# Windows Forms utility for creating files and folders with clipboard support.

## Features

- Create new files and folders from a GUI
- Write clipboard content to new files
- Automatically open created items in Total Commander
- INI-based configuration persistence
- Keyboard shortcuts (Enter, Shift+Enter, F1, F2, Escape)
- Recursive folder/file creation (e.g. `path/to/dir` or `path/to/file.txt`)

## Building

```bash
cargo build --release
```

The executable will be in `target/release/newfile.exe`

## Running

```bash
# With root directory
cargo run -- "C:\path\to\directory"

# Without root directory (uses current directory)
cargo run
```

## Configuration

The application stores settings in `NewFile.ini`:
- `autowriteclipboard`: Whether to write clipboard to new files
- `autoopenaftercreated`: Whether to open items after creation
- `txtformat`: Comma-separated list of text file extensions (not currently used)
- `totalcmd`: Path to Total Commander executable

## Dependencies

- `fltk`: Lightweight GUI toolkit used for the interface
- `configparser`: INI file parsing
- `clipboard-win`: Windows clipboard access
- `tokio`: Async runtime

## Differences from C# version

- Uses Rust's memory safety and performance
- Native Windows GUI via Iced framework
- Cross-platform compatible (can run on Linux/macOS with modifications)
- Type-safe configuration handling
