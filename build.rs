fn main() {
    println!("cargo:rerun-if-changed=assets/imgcrop.ico");

    #[cfg(target_os = "windows")]
    {
        winres::WindowsResource::new()
            .set_icon("assets/imgcrop.ico")
            .compile()
            .expect("could not embed the application icon into the Windows executable");
    }
}
