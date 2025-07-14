fn main() {
    if std::env::var("CARGO_FEATURE_OPENGL_11").is_ok() {
        println!("cargo:rustc-link-lib=GL");
    }
}
