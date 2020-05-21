/*!
Implementation of the fcc lattice with stacking faults.

In principle only this needs to be reimplemented for allowing other shapes with other geometries (e.g. 5 fold such as nanorods) to grow.

*/

use nalgebra::Translation3;
use crate::helpers::*;
use crate::parameters::*;


// some helpful values to get the fcc lattice right
const SIN60: f32 = 0.866_025_403_784;
const YPOS: f32 = 0.288_675_134_594;
const ZPOS: f32 = 0.816_496_580_927;
const X0: XYZ = XYZ{x:1.0, y:0.0, z:0.0};
const Y0: XYZ = XYZ{x:0.5, y:SIN60, z:0.0};
const Z0: XYZ = XYZ{x:0.5, y:YPOS, z:ZPOS};

/// Implenentation of the fcc lattice.
///
/// This basically does the mapping between memory locations and real world positions and also provides an iterator over a 3D box.
#[derive(Clone)]
pub struct Lattice {
    pub stacking: Stackings,
    pub stacking_faults: Vec::<u16>,
    diameter: f32,
    // the following parameters are only needed for the iterator
    min: XYZ,
    max: XYZ,
    min_ijk: IJK,
    max_ijk: IJK,
    layer_start: IJK,
    pub origin: IJK,
    curr: IJK,
    next: IJK,
}

impl Lattice {
    /// Initialization of the lattice for given stacking faults (as a vector) and an atom diameter.
    pub fn new(stacking_faults: Vec<u16>, diameter: f32) -> Self {
        Lattice{ 
            stacking: Stackings::new(&stacking_faults), 
            stacking_faults,
            diameter,
            min: XYZ{x: 0.0, y: 0.0, z:0.0}, 
            max: XYZ{x: 0.0, y: 0.0, z:0.0},
            min_ijk: CENTER,
            max_ijk: CENTER,
            layer_start: CENTER,
            origin: CENTER,
            curr: CENTER,
            next: CENTER,
        }
    }
    
    /// mapping from a memory location to a point in space using the fcc lattice
    pub fn get_xyz(&self, ijk: IJK) -> XYZ {
        let a = (ijk.i as f32 - CENTER.i as f32) * self.diameter;
        let b = (ijk.j as f32 - CENTER.j as f32) * self.diameter;
        let c = (ijk.k as f32 - CENTER.k as f32) * self.diameter; 
        let x = X0.x*a + Y0.x*b + Z0.x*c;
        let y = X0.y*a + Y0.y*b + Z0.y*(self.stacking.pos[ijk.k as usize] as f32) * self.diameter;
        let z = X0.z*a + Y0.z*b + Z0.z*c;
        XYZ{x, y, z}   
    }
    
    /// back mapping from the real world to the associated memory location
    pub fn get_ijk(&self, xyz: XYZ ) -> IJK {
        // for the correction further down the xyz needs to be made mutable (xyz is copy anyways)
        let mut xyz = xyz;

        // the k component is simple
        let c = xyz.z/Z0.z;
        let k = (c/self.diameter + CENTER.k as f32).round() as u16; 

        // j and subsequently i have to be compensated for stacking faults
        // this is possible by correcting the original y coordinate
        // note, this is basically just the difference between the stacking pos without and with stacking faults
        xyz.y -= YPOS*(CENTER.k as i32 - (k as i32 - self.stacking.pos[k as usize] as i32 )) as f32 * DIAMETER;

        // the rest is a straight forward back projection
        let b = (xyz.y - Z0.y*c)/Y0.y;                     
        let a = (xyz.x - Y0.x*b - Z0.x*c)/X0.x;
        let j = (b/self.diameter + CENTER.j as f32).round() as u16;   
        let i = (a/self.diameter + CENTER.i as f32).round() as u16; 
        IJK{i, j, k}   
    }
    

    /// a small helper function for the scene
    pub fn position(&self, ijk: IJK) -> Translation3<f32> {
        let xyz: XYZ = self.get_xyz(ijk);
        // Translation3::new(xyz.x, xyz.y, xyz.z)   // When changing that you have to adept the wireframe, too!
        Translation3::new(xyz.y, xyz.z, xyz.x)
    }

    /// give back the position of one of the twelve adjecent neighbors (number 0..11)
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

    /// initialize the box for the iterator
    pub fn init_box_iter(&mut self, min: XYZ, max: XYZ) {
        self.min = min;
        self.max = max;
        self.min_ijk = self.get_ijk(min);
        self.max_ijk = self.get_ijk(max);
        self.origin = self.min_ijk;
        self.layer_start = self.min_ijk;
        self.curr = self.min_ijk;
        self.next = self.min_ijk;
    }

    // check if new starting point is inside the box
    fn is_inside(&self, ijk: IJK) -> bool {
        let center_xyz = self.get_xyz(self.origin);
        let c_x = center_xyz.x;
        let c_y = center_xyz.y;
        // self.get_xyz(ijk).x > (c_x - DIAMETER) && self.get_xyz(ijk).y > (c_y - DIAMETER)
        self.get_xyz(ijk).x >= (c_x - 0.01*DIAMETER) && self.get_xyz(ijk).y >= (c_y - 0.01*DIAMETER)
    } 

    // determine starting point of next layer depending on the stacking
    fn kplus(&mut self, ijk: IJK) -> IJK {
        let mut ijk = ijk.clone();
        ijk.k += 1;
        if self.stacking.shift_i[ijk.k as usize ] == 0  {
            if self.is_inside(IJK{i: ijk.i - 1, j: ijk.j, k: ijk.k}) { 
                ijk.i -= 1;
                // print!("wa") 
            }
            else if self.is_inside(IJK{i: ijk.i, j: ijk.j - 1, k: ijk.k}) {
                ijk.j -= 1;
                //  print!("wy") 
                }
            else if self.is_inside(IJK{i: ijk.i, j: ijk.j, k: ijk.k}) { 
                // print!("w") 
            }
        }
        else {
            if self.is_inside(IJK{i: ijk.i - 1, j: ijk.j, k: ijk.k}) { 
                ijk.i -= 1;
                // print!("wa") 
            }
            else if self.is_inside(IJK{i: ijk.i, j: ijk.j, k: ijk.k}) { 
                // print!("w") 
            }
            else if self.is_inside(IJK{i: ijk.i - 1, j: ijk.j + 1, k: ijk.k}) { 
                ijk.i -= 1;
                ijk.j += 1;
                // print!("ewa") 
            }
        }
        // println!("");
        ijk
    }

}

/// iterator over a box defined by init_box_iter
impl Iterator for Lattice {
    type Item = IJK;
    
    // self.curr and self.next is used for the sequence
    // The following is returned:
    //     -> When the itereator is finished: None
    //     -> Otherwise: the next value in Some
    fn next(&mut self) -> Option<IJK> {

        // check if we are already at top layer
        if self.next.k == self.max_ijk.k + 1 {
            // reset box to zero    
            // self.init_box_iter(XYZ{x: 0.0, y: 0.0, z: 0.0}, XYZ{x: 0.0, y: 0.0, z: 0.0});       
            None
        }
        // still at least a layer to go
        else {
            self.curr = self.next;
            // iterating along the line until the line end -> new line
            if self.get_xyz(IJK{i: self.curr.i + 1, j: self.curr.j, k: self.curr.k}).x <= self.max.x {
                self.next = IJK{i: self.curr.i + 1, j: self.curr.j, k: self.curr.k};
            }
            // iterating over the lines until end of lines -> new layer
            else if self.get_xyz(IJK{i: self.curr.i, j: self.curr.j + 1, k: self.curr.k}).y <= (self.max.y) {
                let i_min = self.layer_start.i - (self.curr.j + 1 - self.layer_start.j)/2;
                self.next = IJK{i: i_min, j: self.curr.j + 1, k: self.curr.k};
            }
            // iterating over the layers
            else {
                self.layer_start = self.kplus(self.layer_start);
                self.next = self.layer_start;
            }
            Some(self.curr)
        }
    }
}


/// Structure to handle the stacking faults
#[derive(Copy,Clone)]
pub struct Stackings {
    pub shift_i: [u16; FLAKE_MAX.k as usize],
    pub shift_j: [i16; FLAKE_MAX.k as usize],
    pub pos: [i16; FLAKE_MAX.k as usize]
}

impl Stackings {
    /// Init the stacking via a vector with the stacking fault positions
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
        // FLAKE_MAX.k must be <= 32 for the following println!'s to work
        // println!("Stacking pos     {:?}", pos);
        // println!("Stacking shift_i {:?}", shift_i);
        // println!("Stacking shift_j {:?}", shift_j);
        // println!("Stacking shift_j sum {:?}", shift_j[7..14 as usize].iter().sum::<i16>());
        Stackings { 
            shift_i, 
            shift_j, 
            pos 
        }
    }
}
