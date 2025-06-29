use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::{BATH_HEIGHT, BATH_WIDTH};
use bath::render::renderer::Renderer;
use raylib::color::Color;
use raylib::ffi;
use raylib::math::{rvec2, Matrix, Vector2, Vector3};

fn main() {
    let mut render = RaylibRenderer::init(BATH_WIDTH, BATH_HEIGHT);
    let width = render.handle.get_screen_width() as f32;
    let height = render.handle.get_screen_height() as f32;
    while !render.handle.window_should_close() {
        let _ = render.handle.begin_drawing(&render.thread);
        let fovy = 90.0_f32;
        let position = Vector3::new(0.0, 6.0, 8.5);
        let target = Vector3::new(0.0, 0.0, 0.0);
        let up = Vector3::new(0.0, 1.0, 0.0);
        let mat_proj = Matrix::perspective(fovy.to_radians(), width / height, 0.025, 500.0);
        let mat_view = Matrix::look_at(position, target, up);
        unsafe {
            ffi::rlClearColor(0, 0, 0, 1);
            ffi::rlEnableDepthTest();
            ffi::rlSetMatrixModelview(mat_view.into());
            ffi::rlSetMatrixProjection(mat_proj.into());
            draw_grid(30, 0.3, Color::PINK);
            ffi::rlDrawRenderBatchActive();
            ffi::rlMatrixMode(ffi::RL_PROJECTION as i32);
            ffi::rlLoadIdentity();
            ffi::rlOrtho(0.0, width as f64, height as f64, 0.0, 0.0, 1.0);
            ffi::rlMatrixMode(ffi::RL_MODELVIEW as i32);
            ffi::rlLoadIdentity();
            draw_rectangle_2d(
                rvec2(width / 4.0, height / 4.0),
                rvec2(width / 2.0, height / 2.0),
                Color::SKYBLUE,
            );
            ffi::rlDrawRenderBatchActive();
        }
    }
}

unsafe fn draw_rectangle_2d(position: Vector2, size: Vector2, color: Color) {
    ffi::rlBegin(ffi::RL_TRIANGLES as i32);
    ffi::rlColor4ub(color.r, color.g, color.b, color.a);
    ffi::rlVertex2i(position.x as i32, position.y as i32);
    ffi::rlVertex2i(position.x as i32, (position.y + size.y) as i32);
    ffi::rlVertex2i((position.x + size.x) as i32, (position.y + size.y) as i32);
    ffi::rlVertex2i(position.x as i32, position.y as i32);
    ffi::rlVertex2i((position.x + size.x) as i32, (position.y + size.y) as i32);
    ffi::rlVertex2i((position.x + size.x) as i32, position.y as i32);
    ffi::rlEnd();
}

unsafe fn draw_grid(slices: i32, spacing: f32, color: Color) {
    let half_slices = slices / 2;
    ffi::rlBegin(ffi::RL_LINES as i32);
    for i in -half_slices..=half_slices {
        ffi::rlColor4ub(color.r, color.g, color.b, color.a);
        ffi::rlVertex3f(i as f32 * spacing, 0.0, -half_slices as f32 * spacing);
        ffi::rlVertex3f(i as f32 * spacing, 0.0, half_slices as f32 * spacing);
        ffi::rlVertex3f(-half_slices as f32 * spacing, 0.0, i as f32 * spacing);
        ffi::rlVertex3f(half_slices as f32 * spacing, 0.0, i as f32 * spacing);
    }
    ffi::rlEnd();
}
