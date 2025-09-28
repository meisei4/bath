fn main() {
    if std::env::var("CARGO_FEATURE_OPENGL_11").is_ok()
        || std::env::var("CARGO_FEATURE_OPENGL_21").is_ok()
        || std::env::var("CARGO_FEATURE_OPENGL_33").is_ok()
        || std::env::var("CARGO_FEATURE_OPENGL_ES_20").is_ok()
        || std::env::var("CARGO_FEATURE_OPENGL_ES_30").is_ok()
    {
        println!("cargo:rustc-link-lib=GL");
    }
}
