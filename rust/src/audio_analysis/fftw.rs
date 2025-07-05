#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types)]
pub struct fftw_complex {
    pub re: f64,
    pub im: f64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types)]
#[allow(non_upper_case_globals)]
pub enum fftw_direction {
    FFTW_FORWARD = -1,
    FFTW_BACKWARD = 1,
}

#[allow(non_camel_case_types)]
pub type fftw_plan = *mut core::ffi::c_void;

extern "C" {
    #[allow(non_camel_case_types)]
    pub fn fftw_create_plan(n: i32, dir: fftw_direction, flags: i32) -> fftw_plan;

    #[allow(non_camel_case_types)]
    pub fn fftw_one(plan: fftw_plan, input: *mut fftw_complex, output: *mut fftw_complex);

    #[allow(non_camel_case_types)]
    pub fn fftw_destroy_plan(plan: fftw_plan);
}
