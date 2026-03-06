cd c:\python\project\TotalcmdPlugin\NewFile-rust
cargo clean
cargo build --release
$editbin = "C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.50.35717\bin\Hostx64\x64\editbin.exe"
& $editbin /SUBSYSTEM:WINDOWS target/release/newfile.exe
Write-Host "Build complete! Executable at: target/release/newfile.exe"