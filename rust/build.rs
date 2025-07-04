// build.rs
use std::path::PathBuf;

fn main() {
    let src = PathBuf::from("c_src/fftw_1997");

    // core runtime & planner
    let mut files = vec![
        "config.c",
        "executor.c",
        "fftwnd.c",
        "planner.c",
        "putils.c",
        "timer.c",
        "generic.c",
        "rader.c",
        "twiddle.c",
        "malloc.c",
        "wisdom.c",
        "wisdomio.c",
    ];

    // forward no-twiddle codelets 1 … 64
    files.extend([
        "fn_1.c", "fn_2.c", "fn_3.c", "fn_4.c", "fn_5.c", "fn_6.c", "fn_7.c", "fn_8.c", "fn_9.c", "fn_10.c", "fn_11.c",
        "fn_12.c", "fn_13.c", "fn_14.c", "fn_15.c", "fn_16.c", "fn_32.c", "fn_64.c",
    ]);

    // inverse no-twiddle codelets
    files.extend([
        "fni_1.c", "fni_2.c", "fni_3.c", "fni_4.c", "fni_5.c", "fni_6.c", "fni_7.c", "fni_8.c", "fni_9.c", "fni_10.c",
        "fni_11.c", "fni_12.c", "fni_13.c", "fni_14.c", "fni_15.c", "fni_16.c", "fni_32.c", "fni_64.c",
    ]);

    // forward twiddle codelets (needed only so config.c links)
    files.extend([
        "ftw_2.c", "ftw_3.c", "ftw_4.c", "ftw_5.c", "ftw_6.c", "ftw_7.c", "ftw_8.c", "ftw_9.c", "ftw_10.c", "ftw_16.c",
        "ftw_32.c", "ftw_64.c",
    ]);

    // inverse twiddle codelets
    files.extend([
        "ftwi_2.c",
        "ftwi_3.c",
        "ftwi_4.c",
        "ftwi_5.c",
        "ftwi_6.c",
        "ftwi_7.c",
        "ftwi_8.c",
        "ftwi_9.c",
        "ftwi_10.c",
        "ftwi_16.c",
        "ftwi_32.c",
        "ftwi_64.c",
    ]);

    cc::Build::new()
        .files(files.into_iter().map(|f| src.join(f)))
        .include(&src) // picks up config.h, fftw.h, fftw-int.h
        .define("HAVE_CONFIG_H", None) // needed because codelets #include "config.h"
        .flag_if_supported("-std=c99")
        .compile("fftw1997");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=c_src/fftw_1997/");
}
