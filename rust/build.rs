use std::path::PathBuf;

fn main() {
    let src = PathBuf::from("c_src/fftw_1997");

    cc::Build::new()
        .files(
            [
                "fftwnd.c",
                "planner.c",
                "malloc.c",
                "fn_2.c",
                "fn_4.c",
                "fn_8.c",
                "fn_16.c",
                "fn_32.c",
                "fn_64.c",
            ]
            .iter()
            .map(|f| src.join(f)),
        )
        .include(&src)
        .define("HAVE_CONFIG_H", None) // or remove if unused in headers
        .flag_if_supported("-std=c99")
        .compile("fftw1997");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=c_src/fftw_1997/");
}
