use libc::{c_char, c_int};
//TODO: this is all the same shit as rustysynth jesus
#[repr(C)]
pub struct tsf {
    _unused: [u8; 0],
}

#[link(name = "tsf", kind = "static")]
extern "C" {
    pub fn tsf_load_filename(path: *const c_char) -> *mut tsf;
    pub fn tsf_get_presetcount(f: *const tsf) -> c_int;
    pub fn tsf_get_presetname(f: *const tsf, preset: c_int) -> *const c_char;
    pub fn tsf_close(f: *mut tsf);
}
