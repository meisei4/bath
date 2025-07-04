#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct fftw_complex {
    pub re: f64,
    pub im: f64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum fftw_direction {
    FFTW_FORWARD = -1,
    FFTW_BACKWARD = 1,
}

pub type fftw_plan = *mut core::ffi::c_void;

extern "C" {
    pub fn fftw_create_plan(n: i32, dir: fftw_direction, flags: i32) -> fftw_plan;

    pub fn fftw_one(plan: fftw_plan, input: *mut fftw_complex, output: *mut fftw_complex);

    pub fn fftw_destroy_plan(plan: fftw_plan);
}
