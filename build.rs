fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set("FileDescription", "NewFile Plugin");
        res.set_icon("Autorun.ico");
        res.compile().unwrap();
    }
}