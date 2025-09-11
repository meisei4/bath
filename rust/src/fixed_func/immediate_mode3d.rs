use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::CameraProjection;
use raylib::ffi::{
    rlDisablePointMode, rlDisableWireMode, rlDrawRenderBatchActive, rlEnableDepthTest, rlEnablePointMode,
    rlEnableWireMode, rlFrustum, rlLoadIdentity, rlMatrixMode, rlMultMatrixf, rlOrtho, rlPopMatrix, rlPushMatrix,
    rlSetLineWidth, rlViewport, DrawModelEx, DrawModelWiresEx, DrawTriangle3D, RL_MODELVIEW, RL_PROJECTION,
};
use raylib::math::Matrix;
use raylib::{ffi, MintVec3};

#[derive(Clone, Copy)]
pub struct Viewport {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

pub fn cell_viewport(view_index: i32, grid_columns: i32, grid_rows: i32, render_w: i32, render_h: i32) -> Viewport {
    let cell_w = render_w / grid_columns;
    let cell_h = render_h / grid_rows;
    let col = view_index % grid_columns;
    let row = view_index / grid_columns;
    let row_inv = (grid_rows - 1) - row; // GL origin bottom-left
    Viewport {
        x: col * cell_w,
        y: row_inv * cell_h,
        w: cell_w,
        h: cell_h,
    }
}

pub struct Immediate3D;

impl Immediate3D {
    #[inline]
    pub fn draw_model_ex(
        &mut self,
        model: impl AsRef<ffi::Model>,
        pos: impl Into<MintVec3>,
        axis: impl Into<MintVec3>,
        angle_deg: f32,
        scale: impl Into<MintVec3>,
        tint: impl Into<Color>,
    ) {
        unsafe {
            DrawModelEx(
                *model.as_ref(),
                pos.into(),
                axis.into(),
                angle_deg,
                scale.into(),
                tint.into(),
            )
        }
    }

    #[inline]
    pub fn draw_model_wires_ex(
        &mut self,
        model: impl AsRef<ffi::Model>,
        pos: impl Into<MintVec3>,
        axis: impl Into<MintVec3>,
        angle_deg: f32,
        scale: impl Into<MintVec3>,
        tint: impl Into<Color>,
    ) {
        unsafe {
            DrawModelWiresEx(
                *model.as_ref(),
                pos.into(),
                axis.into(),
                angle_deg,
                scale.into(),
                tint.into(),
            )
        }
    }

    #[inline]
    pub fn draw_model_points_ex(
        &mut self,
        model: impl AsRef<ffi::Model>,
        pos: impl Into<MintVec3>,
        axis: impl Into<MintVec3>,
        angle_deg: f32,
        scale: impl Into<MintVec3>,
        tint: impl Into<Color>,
    ) {
        unsafe {
            rlEnablePointMode();
            DrawModelEx(
                *model.as_ref(),
                pos.into(),
                axis.into(),
                angle_deg,
                scale.into(),
                tint.into(),
            );
            rlDisablePointMode();
            // DrawModelPointsEx(
            //     *model.as_ref(),
            //     pos.into(),
            //     axis.into(),
            //     angle_deg,
            //     scale.into(),
            //     tint.into(),
            // )
        }
    }
    #[inline]
    pub fn draw_triangle3d(
        &mut self,
        a: impl Into<MintVec3>,
        b: impl Into<MintVec3>,
        c: impl Into<MintVec3>,
        color: impl Into<Color>,
    ) {
        let a_wire = a.into();
        let b_wire = b.into();
        let c_wire = c.into();
        unsafe { DrawTriangle3D(a_wire, b_wire, c_wire, color.into()) }
        unsafe {
            rlSetLineWidth(1.0);
            rlEnableWireMode();
            DrawTriangle3D(a_wire, b_wire, c_wire, Color::WHITE);
            rlDisableWireMode();
        }
    }
}

pub unsafe fn with_immediate_mode3d<F>(observer: &Camera3D, viewport: Viewport, near: f32, far: f32, mut capture: F)
where
    F: FnMut(&mut Immediate3D),
{
    begin_immediate_mode3d(observer, viewport, near, far);
    let mut context = Immediate3D;
    capture(&mut context);
    end_immediate_mode3d();
}

pub unsafe fn begin_immediate_mode3d(observer: &Camera3D, viewport: Viewport, near: f32, far: f32) {
    rlDrawRenderBatchActive();
    rlViewport(viewport.x, viewport.y, viewport.w, viewport.h);
    let aspect = viewport.w as f32 / viewport.h as f32;
    rlMatrixMode(RL_PROJECTION as i32);
    rlPushMatrix();
    rlLoadIdentity();
    match observer.projection {
        CameraProjection::CAMERA_PERSPECTIVE => {
            // TODO is fovy truly VERTICAL angle?????? what?
            let top = (near as f64) * ((observer.fovy as f64 * 0.5).to_radians().tan());
            let right = top * aspect as f64;
            rlFrustum(-right, right, -top, top, near as f64, far as f64);
        },
        CameraProjection::CAMERA_ORTHOGRAPHIC => {
            // TODO Camera3D.fovy means HORIZONTAL width in world units?????? what?
            let width = observer.fovy as f64;
            let height = width / aspect as f64;
            let l = -0.5 * width;
            let r = 0.5 * width;
            let b = -0.5 * height;
            let t = 0.5 * height;
            rlOrtho(l, r, b, t, near as f64, far as f64);
        },
    }
    rlMatrixMode(RL_MODELVIEW as i32);
    rlPushMatrix();
    rlLoadIdentity();
    let view = Matrix::look_at(observer.position, observer.target, observer.up);
    rlMultMatrixf(view.to_array().as_ptr());
    rlEnableDepthTest();
}

pub unsafe fn end_immediate_mode3d() {
    rlMatrixMode(RL_MODELVIEW as i32);
    rlPopMatrix();
    rlMatrixMode(RL_PROJECTION as i32);
    rlPopMatrix();
    rlMatrixMode(RL_MODELVIEW as i32);
}
