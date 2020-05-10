use nalgebra::Translation3;
use crate::helpers::*;
use crate::parameters::*;


#[derive(Copy,Clone,Debug)]
pub struct XYZ {pub x: f32, pub y: f32, pub z: f32}

// some helpful values to get the fcc lattice right
const SIN60: f32 = 0.866_025_403_784;
const YPOS: f32 = 0.288_675_134_594;
const ZPOS: f32 = 0.816_496_580_927;
const X0: XYZ = XYZ{x:1.0, y:0.0, z:0.0};
const Y0: XYZ = XYZ{x:0.5, y:SIN60, z:0.0};
const Z0: XYZ = XYZ{x:0.5, y:YPOS, z:ZPOS};

#[derive(Clone)]
pub struct Lattice {
    pub stacking: Stackings,
    pub stacking_faults: Vec::<u16>,
    diameter: f32
}

impl Lattice {
    pub fn new(stacking_faults: Vec<u16>, diameter: f32) -> Self {
        Lattice{ 
            stacking: Stackings::new(&stacking_faults), 
            stacking_faults,
            diameter 
        }
    }
    
    pub fn get_xyz(&self, ijk: IJK) -> XYZ {
        // mapping from a memory location to a point in space using the fcc lattice
        let a = (ijk.i as f32 - CENTER.i as f32) * self.diameter;
        let b = (ijk.j as f32 - CENTER.j as f32) * self.diameter;
        let c = (ijk.k as f32 - CENTER.k as f32) * self.diameter; 
        let x = X0.x*a + Y0.x*b + Z0.x*c;
        let y = X0.y*a + Y0.y*b + Z0.y*(self.stacking.pos[ijk.k as usize] as f32) * self.diameter;
        let z = X0.z*a + Y0.z*b + Z0.z*c;
        XYZ{x, y, z}   
    }
    
    pub fn get_ijk(&self, xyz: XYZ ) -> IJK {
        // back mapping from the real world to the memory location
        let c = xyz.z/Z0.z;
        let b = (xyz.y - Z0.y/Z0.z*xyz.z)/Y0.y;
        let a = xyz.x - 0.5*b - 0.5*c;
        let k = (c/self.diameter + CENTER.k as f32) as u16; 
        let j = (b/self.diameter + CENTER.j as f32) as u16; 
        let i = (a/self.diameter + CENTER.i as f32) as u16; 
        IJK{i, j, k}   
    }
    
    pub fn position(&self, ijk: IJK) -> Translation3<f32> {
        // a small helper for the scene
        let xyz: XYZ = self.get_xyz(ijk);
        // Translation3::new(xyz.x, xyz.y, xyz.z)   // When changing that you have to adept the wireframe, too!
        Translation3::new(xyz.y, xyz.z, xyz.x)
    }

    pub fn next_neighbor(&self, ijk: IJK, neighbor: usize) -> IJK {  
        // Note, the stacking shift occurs between the layers.
        // So, the shift[k+1] refers to the shift from layer k to k+1
        // and the shift[k] from k to k-1.
        let IJK{i,j,k} = ijk;
        match neighbor {
            0 => IJK{i: i + 1 ,j: j + 0 ,k: k},
            1 => IJK{i: i + 0 ,j: j + 1 ,k: k},
            2 => IJK{i: i + 1 ,j: j - 1 ,k: k},
            3 => IJK{i: i - 1 ,j: j + 1 ,k: k},
            4 => IJK{i: i - 1 ,j: j + 0 ,k: k},
            5 => IJK{i: i + 0 ,j: j - 1 ,k: k},
            6 => IJK{i: i + 0 ,j: j + 0 ,k: k + 1},
            7 => IJK{i: i - self.stacking.shift_i[(k+1) as usize] as u16,j: (j as i16 - self.stacking.shift_j[(k+1) as usize]) as u16 ,k: k + 1},
            8 => IJK{i: i - 1 ,j: j + 0 ,k: k + 1},        
            9 => IJK{i: i + 0 ,j: j + 0 ,k: k - 1},  
            10 => IJK{i: i + self.stacking.shift_i[k as usize] as u16,j: (j as i16 + self.stacking.shift_j[k as usize]) as u16 ,k: k - 1}, 
            11 => IJK{i: i + 1 ,j: j + 0 ,k: k - 1}, 
            _ => IJK{i,j,k}
        }
    }
}

#[derive(Copy,Clone)]
pub struct Stackings {
    pub shift_i: [u16; FLAKE_MAX.k as usize],
    pub shift_j: [i16; FLAKE_MAX.k as usize],
    pub pos: [i16; FLAKE_MAX.k as usize]
}

impl Stackings {
    pub fn new(stacking_faults: &Vec::<u16>) -> Self {
        // init the three lists with 1s
        let mut shift_i: [u16; FLAKE_MAX.k as usize] = [1; FLAKE_MAX.k as usize]; 
        let mut shift_j: [i16; FLAKE_MAX.k as usize] = [1; FLAKE_MAX.k as usize]; 
        let mut pos: [i16; FLAKE_MAX.k as usize] = [1; FLAKE_MAX.k as usize]; 
        // set the positions with stacking fault to -1
        for i in stacking_faults.iter() {
            shift_j[*i as usize] = -1;
        }
        // magic: sum up with multiply of the element -> after each stacking fault it will count backwards
        for i in 1..FLAKE_MAX.k {
            shift_j[i as usize] = shift_j[(i-1) as usize]*shift_j[i as usize];
        }        
        // copy the basic behaviour to the other lists
        for i in 0..FLAKE_MAX.k {
            pos[i as usize] = shift_j[0..(i+1) as usize].iter().sum();
            if shift_j[i as usize] == 1 {shift_i[i as usize] = 0};
        }        
        // set the central position to zero
        let kzero = pos[CENTER.k as usize];
        pos.iter_mut().for_each(|el| *el -= kzero);
        // FLAKE_MAX.k must be <= 32 for the following dbg's to work
        // dbg!("Stacking pos     {:?}", pos);
        // dbg!("Stacking shift_i {:?}", shift_i);
        // dbg!("Stacking shift_j {:?}", shift_j);
        
        Stackings { 
            shift_i, 
            shift_j, 
            pos 
        }
    }
}
