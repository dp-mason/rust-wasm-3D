extern crate nalgebra as na;
use na::{Matrix4};

pub fn make_x_rotation_matrix(angle_degrees:f32) -> Matrix4<f32>{
    let angle_radians = angle_degrees.to_radians();
    let x_rot_matrix:Matrix4<f32> = Matrix4::new(
        1.0,         0.0        ,            0.0      , 0.0,
        0.0, angle_radians.cos(), -angle_radians.sin(), 0.0,
        0.0, angle_radians.sin(),  angle_radians.cos(), 0.0,
        0.0,         0.0        ,            0.0      , 1.0);
    x_rot_matrix
}
    
pub fn make_y_rotation_matrix(angle_degrees:f32) -> Matrix4<f32>{
    let angle_radians = angle_degrees.to_radians();
    let y_rot_matrix:Matrix4<f32> = Matrix4::new(
        angle_radians.cos(), 0.0, -1.0 * angle_radians.sin(), 0.0,
                0.0        , 1.0,            0.0            , 0.0,
        angle_radians.sin(), 0.0,    angle_radians.cos()    , 0.0,
                0.0        , 0.0,            0.0            , 0.0
    );
    y_rot_matrix
}

pub fn make_z_rotation_matrix(angle_degrees:f32) -> Matrix4<f32>{
    let angle_radians = angle_degrees.to_radians();
    let z_rot_matrix:Matrix4<f32> = Matrix4::new(
        angle_radians.cos(), -1.0 * angle_radians.sin(), 0.0, 0.0,
        angle_radians.sin(),     angle_radians.cos()   , 0.0, 0.0,
                0.0        ,             0.0           , 1.0, 0.0,
                0.0        ,             0.0           , 0.0, 0.0
    );
    z_rot_matrix
}

pub fn make_translation_matrix(x_delta:f32, y_delta:f32, z_delta:f32) -> Matrix4<f32>{
    let translate_matrix:Matrix4<f32> = Matrix4::new(
          1.0  ,   0.0  ,   0.0  , 0.0,
          0.0  ,   1.0  ,   0.0  , 0.0,
          0.0  ,   0.0  ,   1.0  , 0.0,
        x_delta, y_delta, z_delta, 0.0
    );
    translate_matrix
}