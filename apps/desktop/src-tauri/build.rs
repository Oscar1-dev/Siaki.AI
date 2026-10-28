fn main() {
    let attributes = tauri_build::Attributes::new().windows_attributes(
        tauri_build::WindowsAttributes::new().static_vc_runtime(true),
    );
    tauri_build::try_build(attributes).expect("failed to run tauri-build");
}
