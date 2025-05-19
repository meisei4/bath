fn main() {
    cc::Build::new()
        .warnings(false)
        .file("tsf.c")
        .include(".")
        .compile("tsf");
}
