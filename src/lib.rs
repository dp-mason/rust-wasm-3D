pub mod scene;
pub mod transformations;
pub mod renderer;

use nalgebra::{Matrix3x2, Matrix4};
use wasm_bindgen::prelude::*;
use num::clamp;

use scene::mesh::*;
use scene::camera::*;
use scene::*;
use renderer::*;
use std::mem;
//use transformations::*;
 
// Define the size of our canvas
// in the future will be loaded from config, min size 10x10
const CANVAS_WIDTH:usize = 480;
const CANVAS_HEIGHT:usize = 480;

const CANVAS_W_I16:i16 = CANVAS_WIDTH as i16;
const CANVAS_H_I16:i16 = CANVAS_HEIGHT as i16;

const CANVAS_W_F32:f32 = CANVAS_WIDTH as f32;
const CANVAS_H_F32:f32 = CANVAS_HEIGHT as f32;

const OUTPUT_BUFFER_SIZE: usize = CANVAS_HEIGHT * CANVAS_WIDTH * 4; // 4 u8 values for each pixel
static mut OUTPUT_BUFFER: [u8; OUTPUT_BUFFER_SIZE] = [255; OUTPUT_BUFFER_SIZE];

const Z_BUFFER_LEN:usize = CANVAS_WIDTH * CANVAS_HEIGHT;
static mut Z_BUFFER: [f32; Z_BUFFER_LEN] = [0.0; Z_BUFFER_LEN];

const AR:f32 = CANVAS_WIDTH as f32 / CANVAS_HEIGHT as f32; // aspect ratio of window (height over width)

const CANVAS_SLOPE:f32     = CANVAS_HEIGHT as f32 / CANVAS_WIDTH as f32;
const INV_CANVAS_SLOPE:f32 = 1.0 / CANVAS_SLOPE;

const PERCENT_OF_MINUTE:f32 = 1.0 / 60.0;

const DEBUG:bool = true;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Point2d_i( i16,  i16);
#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Point2d_u( usize,  usize);

pub struct Line2d_i(( i16,  i16), ( i16,  i16));
#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Line2d_u(( usize,  usize), ( usize,  usize));

fn lerp_f(percent:f32, min:f32, max:f32) -> Option<f32> {
    if percent >= 0.0 && percent <= 1.0 && min < max {
        Some( percent * (max - min) + min )
    }
    else {
        None
    }
}

// Function to return a pointer to our buffer
// in wasm memory
#[wasm_bindgen]
pub fn get_output_buffer_pointer() -> *const u8 {
    let pointer: *const u8;
    unsafe {
        pointer = OUTPUT_BUFFER.as_ptr();
    }
    
    return pointer;
}

// matrix transform to canvas space
const CANVAS_COORD_TRANSFORM:Matrix3x2<f32> = Matrix3x2::new(
         CANVAS_W_F32        ,            0.0            ,
              0.0            ,      -CANVAS_H_F32        ,
    CANVAS_WIDTH as f32 / 2.0, CANVAS_HEIGHT as f32 / 2.0
);

pub fn get_mist_factor(z_val:f32, min_z:f32, max_z:f32) -> f32 {
    let percent = (z_val - min_z) / (max_z - min_z); //TODO: pass in percent, do mult instead, faster
    percent / ((2.0 as f32).powf(-3.0) - 1.0 * (1.0 - percent) + 1.0)
}

pub fn mix_values(a:f32, b:f32, factor:f32) -> f32{
    (a * 1.0 - factor) + (b * factor)
}

//TODO: mist pass function
pub fn apply_mist_pass_from_z_buffer(cam:Camera) {
    for pixel in 0..Z_BUFFER_LEN {
        unsafe {
            if Z_BUFFER[pixel] < f32::MAX {
                let mistiness = get_mist_factor(Z_BUFFER[pixel], cam.znear, cam.zfar);
                
                OUTPUT_BUFFER[pixel * 4    ] = mix_values(OUTPUT_BUFFER[pixel * 4    ] as f32, 255.0, mistiness) as u8;
                OUTPUT_BUFFER[pixel * 4 + 1] = mix_values(OUTPUT_BUFFER[pixel * 4 + 1] as f32, 255.0, mistiness) as u8;
                OUTPUT_BUFFER[pixel * 4 + 2] = mix_values(OUTPUT_BUFFER[pixel * 4 + 2] as f32, 255.0, mistiness) as u8;
            }
        }
    }
}

pub fn put_z_buffer_pixel(x:usize, y:usize, z_val:f32) {
    let loc_within_buffer = (y * CANVAS_WIDTH) + x;
    // prioritize closer values if lines overlap
    unsafe {
        if z_val < Z_BUFFER[loc_within_buffer] {
            Z_BUFFER[loc_within_buffer] = z_val;
        }
    }
}

// Iterates through each pixel, fills the canvas with black, useful for clearing before each frame is drawn
pub fn clear_frame_buffer(){
    for pixel in 0..OUTPUT_BUFFER_SIZE/4{
        unsafe {
            OUTPUT_BUFFER[pixel*4    ] = 255; // Red
            OUTPUT_BUFFER[pixel*4 + 1] = 255; // Green
            OUTPUT_BUFFER[pixel*4 + 2] = 255; // Blue
            OUTPUT_BUFFER[pixel*4 + 3] = 255; // Alpha (Always Opaque)
        }
        unsafe {
            Z_BUFFER[pixel] = f32::MAX;
        }
    }
}

pub fn put_buffer_pixel(x:usize, y:usize, red: u8, green: u8, blue: u8, alpha: u8){    
    let loc_within_buffer = (y * CANVAS_WIDTH + x) * 4;
    unsafe {
        OUTPUT_BUFFER[loc_within_buffer    ] = red;
        OUTPUT_BUFFER[loc_within_buffer + 1] = green;
        OUTPUT_BUFFER[loc_within_buffer + 2] = blue;
        OUTPUT_BUFFER[loc_within_buffer + 3] = alpha;
    }
}

pub fn draw_clamped_line_to_buffer(x0: usize, y0: usize, x1:usize, y1:usize, start_z:f32, end_z:f32) {    
    let mut curr_z:f32 = start_z;
    let increment_z:f32 = 1.0 / (((x1 - x0) as f32).powf(2.0) + ((y1 as f32 - y0 as f32)).powf(2.0)).sqrt() * (end_z as f32 - start_z as f32);

    if x0 != x1 && y0 != y1{
        // if line is not horizontal or vertical
        //https://en.wikipedia.org//wiki/Bresenham's_line_algorithm#Derivation
        let abs_delta_x:i16 = (x1 - x0) as i16; // x1 guaranteed to be bigger than x0 since that is part of clamping
        let sign_of_x_delta:i16 = 1; // sign of x 

        let neg_delta_y:i16 = -1 * (y1 as i16 - y0 as i16).abs();
        let sign_of_y_delta:i16 = if y0 < y1 { 1 } else { -1 };

        let mut error = abs_delta_x + neg_delta_y;

        let mut curr_x:i16 = x0 as i16;
        let mut curr_y:i16 = y0 as i16;
        let x_end:i16 = x1 as i16;
        let y_end:i16 = y1 as i16;

        
        loop {
            put_buffer_pixel(curr_x as usize, curr_y as usize, 0, 0, 0, 255);
            put_z_buffer_pixel(curr_x as usize, curr_y as usize, curr_z);
            
            if curr_x == x_end && curr_y == y_end { break; }
            curr_z += increment_z;

            let e2 = 2 * error;
            
            if e2 >= neg_delta_y {
                if curr_x == x_end { break }
                error = error + neg_delta_y;
                curr_x = curr_x + sign_of_x_delta;
            }
            if e2 <= abs_delta_x {
                if curr_y == y_end { break }
                error = error + abs_delta_x;
                curr_y = curr_y + sign_of_y_delta;
            }
        }
    } else {
        match x0 == x1 {
            true => {
                // if vertical (or single point)
                let min_y = if y0 > y1 { y1 } else { y0 };
                let max_y = if y0 > y1 { y0 } else { y1 };
                for curr_y in min_y..max_y {
                    put_buffer_pixel(x0, curr_y, 0, 0, 0, 255);
                    put_z_buffer_pixel(x0, curr_y, curr_z);
                    curr_z += increment_z;
                }
            }
            false => {
                // we have x0 < x1 guarantee
                for curr_x in x0..x1 {
                    put_buffer_pixel(curr_x, y0, 0, 0, 0, 255);
                    put_z_buffer_pixel(curr_x, y0, curr_z);
                    curr_z += increment_z;
                }
            }
        }
    }
}

#[derive(Debug)]
enum Border {
    Left,
    Top,
    Bottom,
    Right
}

const ALL_BORDERS:[Border; 4] = [Border::Top, Border::Left, Border::Right, Border::Bottom];

// TODO: add enum argument (top, right, bottom, left) for the border to be checked
// TODO: what border to the corner pixels belong to? corner inclusive arg?
fn find_border_intersect(slope:f32, y_intercept:f32, bord_select:Border) -> Option<Point2d_u> {
    // TODO: broken function
    match bord_select {
        Border::Top => {
            // calculate the top border intersect, equivalent to x intercept
            let x_intercept:i16 = ((-1.0 * y_intercept) / slope) as i16;
            if x_intercept < CANVAS_W_I16 - 1 && x_intercept > 0 {
                return Some(Point2d_u(x_intercept as usize, 0));
            }
        }
        Border::Right => {
            let right_border_intersect = (slope * (CANVAS_W_F32 - 1.0) + y_intercept) as i16;
            if right_border_intersect < CANVAS_H_I16 && right_border_intersect >= 0 {
                return Some(Point2d_u(CANVAS_WIDTH - 1, right_border_intersect as usize));
            }
        }
        Border::Bottom => {
            let bottom_border_intersect = ((CANVAS_H_F32 - y_intercept) / slope) as i16;
            if bottom_border_intersect < CANVAS_W_I16 && bottom_border_intersect >= 0 {
               return Some(Point2d_u(bottom_border_intersect as usize, CANVAS_HEIGHT - 1));
            }
        }
        Border::Left => {
            // "< CANVAS_W_I16 - 1" and "> 0" ensures corners are not counted twice
            if (y_intercept as i16) >= 0 && (y_intercept as i16) < CANVAS_H_I16 {
                return Some(Point2d_u(0, y_intercept as usize));
            }
        }
    }

    None
}

fn find_border_intersect_in_x_range(slope:f32, y_intercept:f32, xmin:i16, xmax:i16) -> Option<Point2d_u> {
    
    for curr_border in ALL_BORDERS.into_iter() {
        match find_border_intersect(slope, y_intercept, curr_border) {
            Some(pt) => {
                if (pt.0 as i16) >= xmin && (pt.0 as i16) <= xmax {
                    return Some(pt);
                }
            }
            None => {}
        }
    }

    None
}

// TODO: there has got to be a faster or smarter way to do line clamping
// find intersections with the canvas borders, 2 Max possible for straight line
fn find_canvas_intersections(slope:f32, y_intercept:f32) -> Option<( [Point2d_u; 2] )> {
    let mut isects_found:usize = 0;
    let mut intersects = [Point2d_u(0,0); 2];

    // test every border for intersection: top, right, bottom, left
    for curr_border in ALL_BORDERS.into_iter() {
        match find_border_intersect(slope, y_intercept, curr_border) {
            Some(x) => {
                intersects[isects_found] = x;
                isects_found += 1;
            }
            None => {}
        }
        if isects_found > 1 {break}
    }

    match isects_found {
        0 => {
            return None;
        }
        1 => {
            intersects[1] = intersects[0];
            return Some(intersects); //corner case
        }
        2 => {
            return Some(intersects)
        }
        _ => {
            return None; //TODO error messaging
        }
    }
}

// if any part of the line is in the canvas, return the clamped coords
// else return None
pub fn clamp_line_to_canvas(line:&Line2d_i) -> Option<Line2d_u> {

    // ensure x0 < x1, simplifies things
    let mut x0 = line.0.0;
    let mut y0 = line.0.1;
    let mut x1 = line.1.0;
    let mut y1 = line.1.1;
    if x0 > x1 {
        let xtemp = x1;
        let ytemp = y1;
        x1 = x0;
        y1 = y0;
        x0 = xtemp;
        y0 = ytemp;  
    }

    let first_point_inside:bool = x0 >= 0 && x0 < CANVAS_W_I16 && y0 >= 0 && y0 < CANVAS_H_I16;
    let second_point_inside:bool = x1 >= 0 && x1 < CANVAS_W_I16 && y1 >= 0 && y1 < CANVAS_H_I16;
    
    if x0 == x1 { // if vertical line (or single point provided)
        // vertical
        if x0 < 0 || x0 >= CANVAS_W_I16{ // cull line outside canvas
            return None;
        }
        let x = num::clamp(x0, 0, CANVAS_W_I16 - 1) as usize;
        return Some(Line2d_u(
            (x, num::clamp(y0, 0, CANVAS_H_I16 - 1) as usize),
            (x, num::clamp(y1, 0, CANVAS_H_I16 - 1) as usize)
        ));
    } else if y0 == y1 {
        // horizontal
        if y0 < 0 || y0 >= CANVAS_H_I16{ // cull line outside canvas
            return None;
        }
        let y = num::clamp(y0, 0, CANVAS_H_I16 - 1) as usize;
        return Some(Line2d_u(
            (num::clamp(x0, 0, CANVAS_W_I16 - 1) as usize, y), 
            (num::clamp(x1, 0, CANVAS_W_I16 - 1) as usize, y)
        ));
    } else if first_point_inside && second_point_inside { // if already inside canvas, return line back (now reordered)
        return Some(Line2d_u((x0 as usize,y0 as usize),(x1 as usize,y1 as usize)));   
    } else if first_point_inside || second_point_inside {
        // find line equation
        let slope:f32 = (y1 - y0) as f32 / (x1 - x0) as f32;
        let y_intercept:f32 = y0 as f32 - (slope * x0 as f32);

        // if either point is in the canvas, there can be at most one intersect
        if first_point_inside {
            let clamped_point = find_border_intersect_in_x_range(slope, y_intercept, x0, x1);
            match clamped_point {
                Some(pt) => {
                    return Some(Line2d_u( (x0 as usize, y0 as usize), (pt.0, pt.1) ));
                }
                None => {None}
            }         
        } else {
            let clamped_point = find_border_intersect_in_x_range(slope, y_intercept, x0, x1);
            match clamped_point {
                Some(pt) => {
                    return Some(Line2d_u( (pt.0, pt.1), (x1 as usize, y1 as usize) ));
                }
                None => {None}
            }
        }
    } else {
        // find if there are any intersects, both points are outside canvas
        let mut returned_line = Line2d_u((0,0),(0,0));

        // find line equation
        let slope:f32 = (y1 - y0) as f32 / (x1 - x0) as f32;
        let y_intercept:f32 = y0 as f32 - (slope * x0 as f32);
        let maybe_intercepts = find_canvas_intersections(slope, y_intercept);
        match maybe_intercepts {
            Some(intercepts) => {
                // ensure line is drawn left to right
                if intercepts[0].0 < intercepts[1].0 {
                    return Some( Line2d_u((intercepts[0].0, intercepts[0].1), (intercepts[1].0, intercepts[1].1)) );
                } else {
                    return Some( Line2d_u((intercepts[1].0, intercepts[1].1), (intercepts[0].0, intercepts[0].1)) );
                }
            },
            None => {return None} // no intersects found, if reached line must be outside the canvas completely
        }
    }
}

// draw a line on the canvas buffer, this is in canvas space coords, 2d pixel coords
//  0 . . . Width
//  . . . . 
//  . . . .
// Height
pub fn draw_line(x0: i16, y0: i16, x1:i16, y1:i16, start_z:f32, end_z:f32) -> bool {
    
    let mut z_range = (start_z, end_z);
    
    if x0 > x1 {
        z_range.0 = end_z;
        z_range.1 = start_z;
    }

    let clamped_line = clamp_line_to_canvas(  &Line2d_i( (x0,y0),(x1,y1) )  );

    match clamped_line {
        Some(line) => {
            draw_clamped_line_to_buffer(line.0.0, line.0.1, line.1.0, line.1.1, z_range.0, z_range.1);
            return true;
        }   
        None => { return false; }
    }
}

// takes a triangle with origin at 0 coords and projects the coords to canvas space coords
// then draws triangle on the canvas with draw_line
pub fn draw_projected_triangle(a:(f32, f32), b:(f32, f32), c:(f32, f32), z_a:f32, z_b:f32, z_c:f32) {
    // transform to pixel canvas bitmap coordinates
    let ax = (a.0 * CANVAS_COORD_TRANSFORM.m11 + a.1 * CANVAS_COORD_TRANSFORM.m21 + CANVAS_COORD_TRANSFORM.m31) as i16;
    let ay = (a.0 * CANVAS_COORD_TRANSFORM.m12 + a.1 * CANVAS_COORD_TRANSFORM.m22 + CANVAS_COORD_TRANSFORM.m32) as i16;

    let bx = (b.0 * CANVAS_COORD_TRANSFORM.m11 + b.1 * CANVAS_COORD_TRANSFORM.m21 + CANVAS_COORD_TRANSFORM.m31) as i16;
    let by = (b.0 * CANVAS_COORD_TRANSFORM.m12 + b.1 * CANVAS_COORD_TRANSFORM.m22 + CANVAS_COORD_TRANSFORM.m32) as i16;
    
    let cx = (c.0 * CANVAS_COORD_TRANSFORM.m11 + c.1 * CANVAS_COORD_TRANSFORM.m21 + CANVAS_COORD_TRANSFORM.m31) as i16;
    let cy = (c.0 * CANVAS_COORD_TRANSFORM.m12 + c.1 * CANVAS_COORD_TRANSFORM.m22 + CANVAS_COORD_TRANSFORM.m32) as i16;

    draw_line(ax, ay, bx, by, z_a, z_b); // z value is for mist pass
    draw_line(bx, by, cx, cy, z_b, z_c);
    draw_line(cx, cy, ax, ay, z_c, z_a);
}

// TODO: return None if both verts are outside the frustum
pub fn persp_project_vert(vert:Vert3, perspective_matx:Matrix4<f32>) -> (f32, f32){
    let mut xtemp = vert.x * perspective_matx.m11 + vert.y * perspective_matx.m21 + vert.z * perspective_matx.m31 + perspective_matx.m41;
    let mut ytemp = vert.x * perspective_matx.m12 + vert.y * perspective_matx.m22 + vert.z * perspective_matx.m32 + perspective_matx.m42;
    let mut ztemp = vert.x * perspective_matx.m13 + vert.y * perspective_matx.m23 + vert.z * perspective_matx.m33 + perspective_matx.m43;
    let w         = vert.z * perspective_matx.m34;
    
    if w != 1.0 {
       xtemp /= w;
       ytemp /= w;
    }

    (xtemp, ytemp)
}

fn draw_mesh(mesh:&Mesh, camera:&Camera) {
    let perspective_matx = camera.pers_tranfm_matx;

    let mut projected_verts:Vec<(f32, f32)> = vec![(0.0, 0.0); mesh.verts.len()];
    
    // project all verts
    for index in 0..mesh.verts.len(){
        projected_verts[index] = persp_project_vert(mesh.verts[index], camera.pers_tranfm_matx);
    }
    
    // draw triangles between projected points
    let numtriindeces = mesh.tris.len();
    for index in 0..mesh.tris.len() / 3{
        draw_projected_triangle(
            projected_verts[mesh.tris[index * 3    ]], // projected coords
            projected_verts[mesh.tris[index * 3 + 1]],
            projected_verts[mesh.tris[index * 3 + 2]],
            mesh.verts[mesh.tris[index * 3    ]].z, // z values of points in 3D space
            mesh.verts[mesh.tris[index * 3 + 1]].z,
            mesh.verts[mesh.tris[index * 3 + 2]].z
        );
    }
}

pub fn render_scene_to_buffer(scene:&Scene){
    clear_frame_buffer();
    for mesh in &scene.meshes{
        draw_mesh(mesh, &scene.camera);
    }
}


// TODO: move scene info to some sort of static memory so it is not regenerated every time

#[wasm_bindgen]
pub fn cube_anim(seconds:f32) -> *const u8{    
    let mut cube:Mesh = Mesh::cube(10.0);
    
    let yrot_mtx:Matrix4<f32> = transformations::make_y_rotation_matrix( (seconds) % 360.0 );
    cube.transform(yrot_mtx);
    
    let translt_mtx:Matrix4<f32> = transformations::make_translation_matrix( (seconds).cos() * 20.0, 0.0, 100.0 - (100.0/3.0) + ((seconds).sin() * 20.0));
    cube.transform(translt_mtx);

    let cam:Camera = Camera::new_default();
    let the_scene:Scene = Scene{meshes:vec![cube], camera:cam};
    render_scene_to_buffer(&the_scene);
    apply_mist_pass_from_z_buffer(the_scene.camera);

    get_output_buffer_pointer()
}

#[wasm_bindgen]
pub fn ico_anim(seconds:f32) -> *const u8{
    let mut ico_sphere:Mesh = Mesh::ico_sphere(10.0, 0);
    
    let yrot_mtx:Matrix4<f32> = transformations::make_y_rotation_matrix( (seconds * 10.0) % 360.0 );
    ico_sphere.transform(yrot_mtx);
    
    let xrot_mtx:Matrix4<f32> = transformations::make_x_rotation_matrix( (seconds * 5.0) % 360.0 );
    ico_sphere.transform(xrot_mtx);
    
    let translt_mtx:Matrix4<f32> = transformations::make_translation_matrix( (seconds).cos() * 30.0, 0.0, 80.0 + ((seconds).sin() * 50.0));
    ico_sphere.transform(translt_mtx);

    let cam:Camera = Camera::new(120.0, 0.1, 120.0);
    let the_scene:Scene = Scene{meshes:vec![ico_sphere], camera:cam};
    render_scene_to_buffer(&the_scene);
    apply_mist_pass_from_z_buffer(the_scene.camera);

    get_output_buffer_pointer()
}

// draws line from top left corner to bottom right
fn line_test_func(input:f32, addend:f32) -> i16 {
    ( (input * CANVAS_SLOPE) + addend ) as i16
}

fn test_clamp(unclamped_line:&Line2d_i, clamped_line:&Line2d_u){   
    assert_eq!(&clamp_line_to_canvas(unclamped_line).unwrap(), clamped_line);
}

// debug various draw functions assuming that canvas is at minimum 10x10 pixel
#[test]
pub fn test_buffer_draw() {

    // Horizontal Line, both ends out
    let mut temp_line    = Line2d_i((-10, CANVAS_H_I16 / 2), (CANVAS_W_I16 + 10, CANVAS_H_I16 / 2));
    let mut temp_clamped = Line2d_u((0, CANVAS_HEIGHT / 2), (CANVAS_WIDTH - 1, CANVAS_HEIGHT / 2));
    test_clamp(&temp_line, &temp_clamped);
    assert!( draw_line(temp_line.0.0, temp_line.0.1, temp_line.1.0, temp_line.1.1, 0.0, 0.0) );
    
    // Vertical Line, both ends out
    temp_line    = Line2d_i((CANVAS_W_I16 / 2, -10), (CANVAS_W_I16 / 2, CANVAS_H_I16 + 10));
    temp_clamped = Line2d_u((CANVAS_WIDTH / 2,   0), (CANVAS_WIDTH / 2, CANVAS_HEIGHT - 1));
    test_clamp(&temp_line, &temp_clamped);
    assert!( draw_line(temp_line.0.0, temp_line.0.1, temp_line.1.0, temp_line.1.1, 0.0, 0.0) );

    // Test corner intersects line extends out of canvas on both sides both intersects are on corner pixels
    let mut addend = 0.0;
    let longest_diag = Line2d_i( (-1, line_test_func(-1.0, 0.0)), (CANVAS_W_I16, CANVAS_H_I16) );
    let longest_diag_clamped = Line2d_u((0, 0), (CANVAS_WIDTH - 1, CANVAS_HEIGHT - 1));
    test_clamp(&longest_diag, &longest_diag_clamped);
    assert!( draw_line(longest_diag.0.0, longest_diag.0.1, longest_diag.1.0, longest_diag.1.1, 0.0, 0.0) );
}

fn draw_radial_line_to_buffer(center_x:i16, center_y:i16, segment_len:f32, angle_deg:f32) -> bool {
    let angle_rad = angle_deg * (3.14159 / 180.0); //.to_radians();
    let end_x = (angle_deg.sin() * segment_len) as i16 + center_x;
    let end_y = (angle_deg.cos() * segment_len) as i16 + center_y;
    return draw_line(center_x as i16, center_y as i16, end_x, end_y, 0.0, 0.0);
}

#[wasm_bindgen]
pub fn line_test_animation(time_since_start_sc:f32) -> *const u8 {
    clear_frame_buffer();

    let anim_len:f32 = 100.0;
    let progress:f32 = (time_since_start_sc % anim_len) / anim_len; // creates value 0..1 for anim progress
    let max_len:f32 = CANVAS_W_F32 + CANVAS_H_F32; // this length guarantees intersection in many cases 
    
    let offset:i16 = 60; //* (progress * (2.0 * 3.14159)).sin() as i16;
    // draw 10 radial lines covering most if not all cases
    // center
    assert!( draw_radial_line_to_buffer(CANVAS_W_I16 / 2, CANVAS_H_I16 / 2, 30.0   , lerp_f(progress, 0.0, 360.0).unwrap()) );
    assert!( draw_radial_line_to_buffer(CANVAS_W_I16 / 2, CANVAS_H_I16 / 2, max_len, lerp_f(progress, 5.0, 365.0).unwrap()) );
    // top
    draw_radial_line_to_buffer(CANVAS_W_I16 / 2, -1 * offset, max_len, lerp_f(progress, 90.0, 270.0).unwrap());
    draw_radial_line_to_buffer(CANVAS_W_I16 / 2, -1 * offset, 30.0   , lerp_f(progress*0.95, 90.0, 270.0).unwrap());
    // right
    draw_radial_line_to_buffer(CANVAS_W_I16 + offset, CANVAS_H_I16 / 2, max_len, lerp_f(progress, 180.0, 360.0).unwrap());
    draw_radial_line_to_buffer(CANVAS_W_I16 + offset, CANVAS_H_I16 / 2, CANVAS_W_F32 * 0.5, lerp_f(progress*0.95, 180.0, 360.0).unwrap());
    // bottom
    draw_radial_line_to_buffer(CANVAS_W_I16 / 2, CANVAS_H_I16 + offset, max_len, lerp_f(progress, 270.0, 270.0 + 180.0).unwrap());
    draw_radial_line_to_buffer(CANVAS_W_I16 / 2, CANVAS_H_I16 + offset, CANVAS_H_F32 * 0.5, lerp_f(progress*0.95, 270.0, 270.0 + 180.0).unwrap());
    // left
    draw_radial_line_to_buffer(-1 * offset, CANVAS_H_I16 / 2, max_len, lerp_f(progress, 0.0, 180.0).unwrap());
    draw_radial_line_to_buffer(-1 * offset, CANVAS_H_I16 / 2, CANVAS_W_F32 * 0.5, lerp_f(progress*0.95, 0.0, 180.0).unwrap());

    //draw_radial_line_to_buffer(CANVAS_W_I16 / 2, -5, max_len, lerp_f(progress, 90.0, 270.0).unwrap());

    get_output_buffer_pointer()
}

#[test]
fn border_intersect_test_anim(){
    test_buffer_draw();
    for sec in 0..720 {
        line_test_animation(sec as f32);
    }
}

#[test]
fn ico_anim_test(){
    for sec in 0..720 {
        ico_anim(sec as f32);
    }
}