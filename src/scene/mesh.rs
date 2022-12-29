extern crate nalgebra as na;
use na::{Matrix4};

#[derive(Copy, Clone)]
pub struct Vert3 {
    pub x: f32,
    pub y: f32,
    pub z: f32
}
impl Vert3{
    fn default_vert() -> Option<Self>{
        return Some(Self{x:0.0, y:0.0, z:0.0});
    }
    fn from_vec(coords:Vec<f32>) -> Option<Self>{
        if coords.len() == 3 {
            return Some(Self{x:coords[0], y:coords[1], z:coords[2]});
        }
        None
    }
}

pub struct Mesh {
    pub verts: Vec<Vert3>,
    pub tris: Vec<usize> // groups of 3, indeces into "points" vector
}
// constructors for common mesh shapes and mesh operations
impl Mesh {
    fn add_verts(&mut self, new_verts:&mut Vec<Vert3>){
        self.verts.append(new_verts);
    }
    fn add_tris(&mut self, new_tris:&mut Vec<usize>){
        match new_tris.len() % 3{
            0 => self.tris.append(new_tris),
            _ => {
                //println!("ERROR: triangles are added in groups of three. They are indexes into the \"verts\" vector\n");
            }
        }
    }
    pub fn primitive_triangle(size:f32) -> Self{
        // isosoles triangle, not perfectly centered
        let vert_one = Vert3{
            x:0.0,
            y:0.35 * size,
            z:0.1 * size // TODO: test
        };
        let vert_two = Vert3{
            x:0.25 * size,
            y:-0.25 * size,
            z:0.0
        };
        let vert_three = Vert3{
            x:vert_two.x * -1.0,
            y:-0.25 * size,
            z:0.0
        };
        
        Self{ verts:vec![vert_one, vert_two, vert_three], tris:vec![0,1,2] } // drawing the triangle clockwise    
    }
    pub fn ico_sphere(size:f32, level:i32) -> Self{
        // adapted from https://schneide.blog/2016/07/15/generating-an-icosphere-in-c/
        let a:f32 = 0.525731112119133606 * size;
        let b:f32 = 0.850650808352039932 * size;
        let c:f32 = 0.0;
        
        let vert_list = vec![
            Vert3{x:-a, y:c, z:b}, Vert3{x: a, y:c, z: b}, Vert3{x:-a, y: c, z:-b}, Vert3{x: a, y: c, z:-b},
            Vert3{x: c, y:b, z:a}, Vert3{x: c, y:b, z:-a}, Vert3{x: c, y:-b, z: a}, Vert3{x: c, y:-b, z:-a},
            Vert3{x: b, y:a, z:c}, Vert3{x:-b, y:a, z: c}, Vert3{x: b, y:-a, z: c}, Vert3{x:-b, y:-a, z: c}
        ];
        
        let tri_list = vec![
            0, 4, 1,  0,9, 4,  9, 5,4,   4,5,8,  4,8, 1,
            8,10, 1,  8,3,10,  5, 3,8,   5,2,3,  2,7, 3,
            7,10, 3,  7,6,10,  7,11,6,  11,0,6,  0,1, 6,
            6, 1,10,  9,0,11,  9,11,2,   9,2,5,  7,2,11
        ];
        
        Self{verts:vert_list, tris:tri_list}
    }
    pub fn cube(size:f32) -> Self {
        let vert_list = vec![
           Vert3{x: size, y: size, z:-size},
           Vert3{x: size, y:-size, z:-size},
           Vert3{x: size, y: size, z: size},
           Vert3{x: size, y:-size, z: size},
           Vert3{x:-size, y: size, z:-size},
           Vert3{x:-size, y:-size, z:-size},
           Vert3{x:-size, y: size, z: size},
           Vert3{x:-size, y:-size, z: size},
        ];

        let tri_list = vec![
            4, 2, 0,
            2, 7, 3,
            6, 5, 7,
            1, 7, 5,
            0, 3, 1,
            4, 1, 5,
            4, 6, 2,
            2, 6, 7,
            6, 4, 5,
            1, 3, 7,
            0, 2, 3,
            4, 0, 1
        ];

        Self{verts:vert_list, tris:tri_list}
    }
    pub fn transform(&mut self, transfm:Matrix4<f32>){
        for vert_index in 0..self.verts.len(){
            let mut temp_vert:Vert3  = self.verts[vert_index];
            self.verts[vert_index].x = temp_vert.x * transfm.m11 + temp_vert.y * transfm.m21 + temp_vert.z * transfm.m31 + transfm.m41;
            self.verts[vert_index].y = temp_vert.x * transfm.m12 + temp_vert.y * transfm.m22 + temp_vert.z * transfm.m32 + transfm.m42;
            self.verts[vert_index].z = temp_vert.x * transfm.m13 + temp_vert.y * transfm.m23 + temp_vert.z * transfm.m33 + transfm.m43;
            //let w:f32                = temp_vert.x * transfm.m14 + temp_vert.y * transfm.m24 + temp_vert.z * transfm.m34 + transfm.m44;

        }
    }
}
