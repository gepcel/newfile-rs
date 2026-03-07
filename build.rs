fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("ico.ico");   // 指定图标文件
    res.compile().unwrap();
}