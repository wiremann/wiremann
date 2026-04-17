fn main() {
    #[cfg(windows)]
    {
        winres::WindowsResource::new()
            .set_icon("assets/icon.ico")
            .compile()
            .unwrap();
    }
}
