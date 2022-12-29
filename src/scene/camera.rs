extern crate nalgebra;
use nalgebra::{Matrix4};

// Define the size of our canvas
// in the future will be loaded from config, min size 10x10
const CANVAS_WIDTH:usize = 480;
const CANVAS_HEIGHT:usize = 480;

const AR:f32 = CANVAS_WIDTH as f32 / CANVAS_HEIGHT as f32; // aspect ratio of window (height over width)

const CANVAS_SLOPE:f32     = CANVAS_HEIGHT as f32 / CANVAS_WIDTH as f32;
const INV_CANVAS_SLOPE:f32 = 1.0 / CANVAS_SLOPE;

pub fn make_perspective_matrix(fov_angle_degrees:f32, znear:f32, zfar:f32) -> Matrix4<f32>{
    let t = fov_angle_degrees.to_radians();
    let scale = 1.0 / (t/2.0).tan();
    // this is only an ortho proj right now
    let persp_proj_matrix:Matrix4<f32> = Matrix4::new(
        AR * scale   , 0.0  ,             0.0             , 0.0,
            0.0      , scale,             0.0             , 0.0,
            0.0      , 0.0  ,     zfar / (zfar-znear)     , 1.0,
            0.0      , 0.0  , -((zfar*znear)/(zfar-znear)), 0.0
    );
    persp_proj_matrix
}

pub struct Camera {
    pub fov_angle_degrees:f32, 
    pub znear:f32, 
    pub zfar:f32,
    pub pers_tranfm_matx:Matrix4<f32>
}
impl Camera {
    pub fn new_default() -> Self {
        Self {
            fov_angle_degrees: 90.0, 
            znear: 0.1, 
            zfar: 100.0,
            pers_tranfm_matx: { make_perspective_matrix(90.0, 0.1, 100.0) }
        }
    }
    pub fn new(fov_degrees:f32, z_near:f32, z_far:f32) -> Self {
        Self {
            fov_angle_degrees: fov_degrees, 
            znear: z_near, 
            zfar: z_far,
            pers_tranfm_matx: { make_perspective_matrix(fov_degrees, z_near, z_far) }
        }
    }
}