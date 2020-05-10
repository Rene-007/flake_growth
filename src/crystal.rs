/*!
The buisness logic of the flake growth program.

It utilizes the Storage back end and provides basic methods in the Crystal struct for:
* adding single gold or dirt atoms
* randomly selecting vacancy which may be turned into a new surface atom
* and for adding many random atoms at once in an optimized way.

It furthermore provides some basic shapes (layers, spheres, cylinders, boxes and rounded boxes) prefilled with atoms as a starting point, a bunch of helpers (get extremas, hexagaon approximation, size) and a statistics "module".

*/


use rand::Rng;
use rand::seq::IteratorRandom;

use std::io::Write;                                                                                                                                                                  
use std::fs::File; 

use crate::helpers::*;
use crate::parameters::*;
use crate::lattice::*;
use crate::storage::*;

#[cfg(target_arch = "wasm32")]
use crate::println;

#[derive(Copy,Clone,Debug)]
pub struct Extrema {pub x_min: f32, pub x_max: f32, pub y_min: f32, pub y_max: f32, pub z_min: f32, pub z_max: f32 }
#[derive(Copy,Clone,Debug)]
pub struct ExtremaCoordinates{pub x_min: IJK, pub x_max: IJK, pub y_min: IJK, pub y_max: IJK, pub z_min: IJK, pub z_max: IJK }

pub struct Crystal {
    pub lattice: Lattice,
    pub prob_list_num: usize,
    pub prob_list: [u64; VAC_LISTS],
    pub prob_list_log: Vec<i8>,
    pub bulk: Bulk,
    pub surface: SurfaceAtoms,
    pub dirt: SurfaceAtoms,
    pub vacancies: Vacancies,
    pub extrema: Extrema,
    pub extrema_ijk: ExtremaCoordinates,
    pub substrate_pos: u16,
}

impl Crystal {
    pub fn new(lattice: Lattice) -> Self {
        let prob_list = PROB_LIST[PROB_LIST_NUM];
        Crystal{ 
            lattice, 
            prob_list_num:  PROB_LIST_NUM, 
            prob_list, 
            prob_list_log:  prob_list.iter().map(|&el| (10.0*el as f32).log10() as i8 -1).collect::<Vec<i8>>(), 
            bulk:   	    Bulk::new(), 
            surface:        SurfaceAtoms::new(), 
            dirt:           SurfaceAtoms::new(), 
            vacancies:      Vacancies::new(), 
            extrema:        Extrema{x_min: 0.0, x_max: 0.0, y_min: 0.0, y_max: 0.0, z_min: 0.0, z_max: 0.0 }, 
            extrema_ijk:    ExtremaCoordinates{x_min: CENTER, x_max: CENTER, y_min: CENTER, y_max: CENTER, z_min: CENTER, z_max: CENTER }, 
            substrate_pos:  1
        }       
    }

    /// Reset the crystal.
    pub fn clear(&mut self) {
        self.bulk.clear();
        self.surface = SurfaceAtoms::new();
        self.dirt = SurfaceAtoms::new();
        self.vacancies = Vacancies::new();
        self.extrema = Extrema{x_min: 0.0, x_max: 0.0, y_min: 0.0, y_max: 0.0, z_min: 0.0, z_max: 0.0 };
        self.extrema_ijk = ExtremaCoordinates{x_min: CENTER, x_max: CENTER, y_min: CENTER, y_max: CENTER, z_min: CENTER, z_max: CENTER };
    }

    /// Reconstruct the vacancy lists -- needed when lattice has changed.
    pub fn update_vacancies(&mut self) {
        self.vacancies = Vacancies::new();

        // iterate over all atoms in the bulk
        for i in self.bulk.i_min-1..=self.bulk.i_max+1 {
            for j in self.bulk.j_min-1..=self.bulk.j_max+1 {
                for k in self.bulk.k_min-1..=self.bulk.k_max+1 {
                    let ijk = IJK{i, j, k};
                    if self.bulk.get(ijk, Atom::Empty) && !self.hidden_atom(ijk) { 
                        // iterate over the 12 vacancies around an atom
                        for l in 0..12 {
                            let nn_ijk = self.lattice.next_neighbor(ijk,l);
                            if nn_ijk.k > self.substrate_pos && self.bulk.get(nn_ijk, Atom::Empty) { 
                                // calc coordiation number and write the position to the associated list
                                match self.number_of_neighbors(nn_ijk) {
                                    1 => { self.vacancies.list[0].insert(nn_ijk); },
                                    x if x>1 && x<9 => {
                                        self.vacancies.list[x-1].insert(nn_ijk);
                                        self.vacancies.list[x-2].take(&nn_ijk);
                                    },
                                    _=> { }
                                }
                            }
                        } 
                    }
                }
            }
        }
    }

    /// Switch to the next propability list.
    pub fn next_prob_list(&mut self) {     
        self.prob_list_num = (self.prob_list_num + 1).rem_euclid(PROB_LIST.len());
        self.prob_list = PROB_LIST[self.prob_list_num];
        self.prob_list_log = self.prob_list.iter().map(|&el| (10.0*el as f32).log10() as i8 -1).collect::<Vec<i8>>();
        // println!("new prob_list {:?}: 10^{:2?} = {:13?}", self.prob_list_num, self.prob_list_log, self.prob_list);
    }

    /// Add a gold atom to the crystal.
    pub fn add_atom(&mut self, ijk: IJK) -> bool {

        // check if anything is already at the position
        if self.bulk.get(ijk, Atom::Empty) {

            // update bulk an surface
            self.bulk.set(ijk,Atom::Gold);
            self.surface.add(ijk);
            self.update_extrema(ijk);

            // update vacancy lists
            // first remove the now occupied position
            self.vacancies.recursive_remove(ijk, 0);
            
            // iterate over each of the 12 position around the added atom
            for l in 0..12 {                            
                
                // check if it is a vacancy and within the boundaries
                let nn_ijk = self.lattice.next_neighbor(ijk,l);
                if nn_ijk.i > 1 && nn_ijk.i < FLAKE_MAX.i - 2 
                    && nn_ijk.j > 1 && nn_ijk.j < FLAKE_MAX.j - 2
                    && nn_ijk.k > self.substrate_pos && nn_ijk.k < FLAKE_MAX.k - 2
                    && self.bulk.get(nn_ijk, Atom::Empty) { 

                        // calc coordiation number of the vacancy and write the position to the associated list
                        match self.number_of_neighbors(nn_ijk) {
                            1 => { self.vacancies.list[0].insert(nn_ijk); },
                            x if x>1 && x<9 => {
                                self.vacancies.list[x-1].insert(nn_ijk);
                                self.vacancies.list[x-2].take(&nn_ijk);
                            },
                            _=> { }
                        }
                }
                
                // if at the position is not a vacancy but a now-hidden atom then remove it from the surface atoms list
                else {
                    if self.hidden_atom(nn_ijk) {
                        self.surface.remove(nn_ijk)
                    }
                }
            }
            true        // atom added
        } 
        else { 
            false       // no atom added
        }
    }

    /// Add a dirt atom to the crystal.
    pub fn add_dirt(&mut self, ijk: IJK) -> bool {

        // check if anything is already at the position
        if self.bulk.get(ijk, Atom::Empty) {

            // update bulk, surface and vacancy lists
            self.bulk.set(ijk,Atom::Dirt);
            self.dirt.add(ijk);
            self.update_extrema(ijk);
            self.vacancies.recursive_remove(ijk, 0);

            true        // atom added
        }
        else { 
            false       // no atom added
        }
    }

    /// Check if an atom is hidden inside the bulk, i.e. not at the surface.
    fn hidden_atom(&self, ijk: IJK) -> bool {
        self.number_of_neighbors(ijk) == 12
    }

    /// Calc the number of neigboring gold atoms for a given position.
    fn number_of_neighbors(&self, ijk: IJK) -> usize {
        let mut number = 0;
        for l in 0..12 {
            let nn_ijk = self.lattice.next_neighbor(ijk, l);
            if self.bulk.get(nn_ijk, Atom::Gold) {
                number += 1;
            }
        }
        number
    }

    /// Pick a random vacancy (which can then be added to the bulk via add_atom).
    pub fn random_vacancy(&self) -> IJK {

        // set up a weighted probability list (prob_sum)
        let mut probabilities = Vec::<u64>::new();
        let mut prob_sum = Vec::<u64>::new();
        for (index,list) in self.vacancies.list.iter().enumerate(){
            probabilities.push(self.prob_list[index]*list.len() as u64);
            prob_sum.push(probabilities.iter().sum());
        }

        // chose a random list
        let mut random_number: u64 = 0;
        if let Some(number) = prob_sum.last() { 
            if *number > 0 { random_number = rand::thread_rng().gen_range(0, *number) as u64 }
        };
        let mut chosen_list: usize = 0;
        if let Some(pos) = prob_sum.iter().position(|&x| x >= random_number) { chosen_list = pos; }

        // pick random atom from the chosen list
        match self.vacancies.list[chosen_list].iter().choose(&mut rand::thread_rng()) {
            Some(&ijk) => ijk,
            None => CENTER
        }   
        // old long version -- same speeed
        // match self.vacancies.list[chosen_list].iter().choose(&mut rand::thread_rng()) {
        //     Some(value) => *value,
        //     None => {
        //         println!("Error: Atom not in list!");
        //         CENTER
        //     }
        // }       
    }


    /// A slightly faster version of random_vacancy + add_atom
    /// -- useful for adding large amounts of atoms at once.
    pub fn random_add(&mut self, number_of_atoms: usize) {

        // init some often used variables
        let mut random_number: u64 = 0;
        let mut probabilities = Vec::<u64>::new();
        let mut prob_sum = Vec::<u64>::new();
        // let mut small_rng = rand::rngs::SmallRng::from_rng(&mut rand::thread_rng()).unwrap();    // is not faster than thread_rng

        // loop to add multiple atoms
        for _index in 0..number_of_atoms {

            // set up a weighted probability list (prob_sum)
            probabilities.clear();
            prob_sum.clear();
            for (index, list) in self.vacancies.list.iter().enumerate(){
                probabilities.push(self.prob_list[index]*list.len() as u64);
                prob_sum.push(probabilities.iter().sum());
            }
            
            // get random position in probability list -- again, SmallRng was not faster here
            if let Some(number) = prob_sum.last() { 
                if *number > 0 { random_number = rand::thread_rng().gen_range(0, *number) as u64 }
            };

            // chose a list
            if let Some(chosen_list) = prob_sum.iter().position(|&x| x >= random_number) { 
                
                // pick random atom from the chosen list
                if let Some(&ijk) = self.vacancies.list[chosen_list].iter().choose(&mut rand::thread_rng()) {
                    self.vacancies.list[chosen_list].take(&ijk);
                    
                    // at to bulk and upgrade numbers
                    self.bulk.set(ijk,Atom::Gold);
                    self.update_extrema(ijk);
                    
                    // update vacancies lists
                    for l in 0..12 {
                        let nn_ijk = self.lattice.next_neighbor(ijk,l);
                        if nn_ijk.i > 1 && nn_ijk.i < FLAKE_MAX.i - 2 
                            && nn_ijk.j > 1 && nn_ijk.j < FLAKE_MAX.j - 2
                            && nn_ijk.k > self.substrate_pos && nn_ijk.k < FLAKE_MAX.k - 2
                            && self.bulk.get(nn_ijk, Atom::Empty) { 
                                match self.number_of_neighbors(nn_ijk) {
                                    1 => { self.vacancies.list[0].insert(nn_ijk); },
                                    x if x>1 && x<9 => {
                                        self.vacancies.list[x-1].insert(nn_ijk);
                                        self.vacancies.list[x-2].take(&nn_ijk);
                                    },
                                    _=> { }
                                }
                        }
                    }
                }
            }
        }   
        
        // reconstruct the surface list -- it is much faster to do this only once at the end
        self.surface.list.clear();
        for i in self.bulk.i_min..=self.bulk.i_max {
            for j in self.bulk.j_min..=self.bulk.j_max {
                for k in self.bulk.k_min..=self.bulk.k_max {
                    let ijk = IJK{i, j, k};
                    if self.bulk.get(ijk, Atom::Gold) && !self.hidden_atom(ijk) {             
                        self.surface.add(ijk)
                    }
                    
                }
            }
        }
    }

    
    /// Update the extrama positions and coordinates of the crystal.
    pub fn update_extrema(&mut self, ijk: IJK) {
        let xyz = self.lattice.get_xyz(ijk);
        if xyz.x < self.extrema.x_min {
            self.extrema.x_min = xyz.x;
            self.extrema_ijk.x_min = ijk;
        }
        if xyz.x > self.extrema.x_max {
            self.extrema.x_max = xyz.x;
            self.extrema_ijk.x_max = ijk;
        }
        if xyz.y < self.extrema.y_min {
            self.extrema.y_min = xyz.y;
            self.extrema_ijk.y_min = ijk;
        }
        if xyz.y > self.extrema.y_max {
            self.extrema.y_max = xyz.y;
            self.extrema_ijk.y_max = ijk;
        }
        if xyz.z < self.extrema.z_min {
            self.extrema.z_min = xyz.z;
            self.extrema_ijk.z_min = ijk;
        }
        if xyz.z > self.extrema.z_max {
            self.extrema.z_max = xyz.z;
            self.extrema_ijk.z_max = ijk;
        }
        // println!("Extrema: {:?}", self.extrema_ijk);
    }


    /// Calculate height, width, depth and aspect ratio of the crystal.
    pub fn get_size(&self) -> [f32;4] {
        let h = (self.extrema.z_max-self.extrema.z_min) + DIAMETER;
        let w = (self.extrema.x_max-self.extrema.x_min) + DIAMETER;
        let d = (self.extrema.y_max-self.extrema.y_min) + DIAMETER;
        let r = ( w  * d ).sqrt() / h;
        [h,w,d,r]
    }


    /// Calculate the coordinates of a hexagon spanned by the extrema.
    pub fn get_hexagon(&self) -> [f32;12] {

        // left side
        let x_min = self.extrema_ijk.x_min;
        let delta1 = self.extrema_ijk.y_max.j as isize - self.extrema_ijk.x_min.j as isize;
        let delta2 = self.extrema_ijk.x_min.j as isize - self.extrema_ijk.y_min.j as isize;
        let a = self.lattice.get_xyz(IJK{i: x_min.i, j: (x_min.j as isize + delta1) as u16, k: x_min.k});
        let b = self.lattice.get_xyz(self.extrema_ijk.x_min);
        let c = self.lattice.get_xyz(IJK{i: (x_min.i as isize + delta2) as u16, j: (x_min.j as isize - delta2) as u16, k: x_min.k});

        //right side
        let x_max = self.extrema_ijk.x_max;
        let delta3 = self.extrema_ijk.y_min.j as isize - self.extrema_ijk.x_max.j as isize;
        let delta4 = self.extrema_ijk.x_max.j as isize - self.extrema_ijk.y_max.j as isize;
        let d = self.lattice.get_xyz(IJK{i: x_max.i, j: (x_max.j as isize + delta3) as u16, k: x_max.k});
        let e = self.lattice.get_xyz(self.extrema_ijk.x_max);
        let f = self.lattice.get_xyz(IJK{i: (x_max.i as isize + delta4) as u16, j: (x_max.j as isize - delta4) as u16, k: x_max.k});

        // output of the xy coordinates
        [a.x,a.y, b.x,b.y, c.x,c.y, d.x,d.y, e.x,e.y, f.x,f.y]
    }


    /// Add a gold/dirt layer to the crystal.
    pub fn add_layer(&mut self, ijk: IJK, layer_size: u16, atom: Atom) {
        
        // iterate over all positions in the range
        let i_min: u16 = ijk.i - layer_size +1;
        let i_max: u16 = ijk.i + layer_size;
        let j_min: u16 = ijk.j - layer_size +1;
        let j_max: u16 = ijk.j + layer_size;
        for is in i_min..i_max {
            for js in j_min..j_max {

                // check if in hexagonal shape
                if is+js >= (i_min + ijk.j) && is+js < (i_max + ijk.j) {
                    let iter_ijk = IJK{i: is, j: js, k: ijk.k};

                    // add depending on the atom
                    match atom {
                        Atom::Empty => {}
                        Atom::Gold  => {self.add_atom(iter_ijk);},
                        Atom::Dirt  => {self.add_dirt(iter_ijk);},
                    }
                }
    
            }
        }
    }


    /// Add a cuboid filled with gold/dirt.
    pub fn add_box(&mut self, center: XYZ, width: f32, depth: f32, height: f32, atom: Atom) {

        // calc some parameters first
        let min = XYZ{x: center.x - width/2.0, y: center.y - depth/2.0, z: center.z - height/2.0};
        let max = XYZ{x: center.x + width/2.0, y: center.y + depth/2.0, z: center.z + height/2.0};
        let min_ijk = self.lattice.get_ijk(min);
        let max_ijk = self.lattice.get_ijk(max);

        // some heuristics to always cover all positions (The fcc lattic is not cartesian!)
        let delta_i = (max_ijk.i as isize - min_ijk.i as isize).abs();
        let delta_j = (max_ijk.j as isize - min_ijk.j as isize).abs();
        let delta_k = (max_ijk.k as isize - min_ijk.k as isize).abs();
        let delta = (delta_k/2* std::cmp::max(delta_i, delta_j)) as u16;                    

        // iter over all possible positions
        for i in (min_ijk.i - delta)..(max_ijk.i + delta) {
            for j in  (min_ijk.j - delta)..(max_ijk.j + delta) {
                for k in  (min_ijk.k - 1)..(max_ijk.k + 1) {

                    // check if within box
                    let iter_ijk = IJK{i,j,k};
                    let pos = self.lattice.get_xyz(iter_ijk);
                    if min.x < pos.x && pos.x < max.x       && min.y < pos.y && pos.y < max.y       && min.z < pos.z && pos.z < max.z {

                        // add depending on the atom
                        match atom {
                            Atom::Empty => {}
                            Atom::Gold  => {self.add_atom(iter_ijk);},
                            Atom::Dirt  => {self.add_dirt(iter_ijk);},
                        }
                    }
    
                }
            }
        }
    }

    /// Add a sphere filled with gold.
    pub fn add_sphere(&mut self, center: XYZ, radius: f32) {

        // calc some parameters first
        let min = XYZ{x: center.x - radius, y: center.y - radius, z: center.z - radius}; 
        let max = XYZ{x: center.x + radius, y: center.y + radius, z: center.z + radius}; 
        let min_ijk = self.lattice.get_ijk(min);
        let max_ijk = self.lattice.get_ijk(max);

        // some heuristics again
        let delta = 2* std::cmp::max((max_ijk.i as isize - min_ijk.i as isize).abs(), (max_ijk.j as isize - min_ijk.j as isize).abs()) as u16;

        // iter over all possible positions
        for i in (min_ijk.i - delta)..(max_ijk.i + delta) {
            for j in  (min_ijk.j - delta)..(max_ijk.j + delta) {
                for k in  (min_ijk.k - 1)..(max_ijk.k + 1) {

                    // check if within sphere and add
                    let iter_ijk = IJK{i,j,k};
                    let pos = self.lattice.get_xyz(iter_ijk);
                    let distance = ( (pos.x - center.x).powi(2) + (pos.y - center.y).powi(2) + (pos.z - center.z).powi(2) ).sqrt();
                    if distance < radius {
                        self.add_atom(iter_ijk);
                    }   
                }
            }
        }
    }

    /// Add an octant (1/8th) of sphere filled with gold.
    pub fn add_sphere_corner(&mut self, center: XYZ, radius: f32, octant: u8) {
        
        // calc some parameters first
        let min = XYZ{x: center.x - radius, y: center.y - radius, z: center.z - radius}; 
        let max = XYZ{x: center.x + radius, y: center.y + radius, z: center.z + radius}; 
        let min_ijk = self.lattice.get_ijk(min);
        let max_ijk = self.lattice.get_ijk(max);

        // some heuristics again
        let delta = 2* std::cmp::max((max_ijk.i as isize - min_ijk.i as isize).abs(), (max_ijk.j as isize - min_ijk.j as isize).abs()) as u16;

        // iter over all possible positions
        for i in (min_ijk.i - delta)..(max_ijk.i + delta) {
            for j in  (min_ijk.j - delta)..(max_ijk.j + delta) {
                for k in  (min_ijk.k - 1)..(max_ijk.k + 1) {
                    
                    // check if within sphere
                    let ijk = IJK{i,j,k};
                    let pos = self.lattice.get_xyz(ijk);
                    let pos = XYZ{x: pos.x - center.x, y: pos.y - center.y, z: pos.z - center.z};
                    let distance = ( (pos.x).powi(2) + (pos.y).powi(2) + (pos.z).powi(2) ).sqrt();

                    // check if in chosen octant and add
                    match octant {
                        0 => if pos.x >= 0.0 && pos.y >= 0.0 && pos.z >= 0.0 && distance < radius {
                            self.add_atom(ijk);
                            },
                        1 => if pos.x <  0.0 && pos.y >= 0.0 && pos.z >= 0.0 && distance < radius {
                            self.add_atom(ijk);
                            },
                        2 => if pos.x >= 0.0 && pos.y <  0.0 && pos.z >= 0.0 && distance < radius {
                            self.add_atom(ijk);
                            },
                        3 => if pos.x <  0.0 && pos.y <  0.0 && pos.z >= 0.0 && distance < radius {
                            self.add_atom(ijk);
                            },
                        4 => if pos.x >= 0.0 && pos.y >= 0.0 && pos.z <  0.0 && distance < radius {
                            self.add_atom(ijk);
                            },
                        5 => if pos.x <  0.0 && pos.y >= 0.0 && pos.z <  0.0 && distance < radius {
                            self.add_atom(ijk);
                            },
                        6 => if pos.x >= 0.0 && pos.y <  0.0 && pos.z <  0.0 && distance < radius {
                            self.add_atom(ijk);
                            },
                        7 => if pos.x <  0.0 && pos.y <  0.0 && pos.z <  0.0 && distance < radius {
                            self.add_atom(ijk);
                            },
                        _ => {}

                    }   
                }
            }
        }
    }

    
    /// Add a cylinder filled with gold.
    pub fn add_cylinder(&mut self, center: XYZ, length: f32, radius: f32) {
        
        // calc some parameters first
        let min = XYZ{x: center.x - length/2.0, y: center.y - radius, z: center.z - radius}; 
        let max = XYZ{x: center.x + length/2.0, y: center.y + radius, z: center.z + radius}; 
        let min_ijk = self.lattice.get_ijk(min);
        let max_ijk = self.lattice.get_ijk(max);

        // some heuristics again
        let delta = 2* std::cmp::max((max_ijk.i as isize - min_ijk.i as isize).abs(), (max_ijk.j as isize - min_ijk.j as isize).abs()) as u16;
        
        // iter over all possible positions
        for i in (min_ijk.i - delta)..(max_ijk.i + delta) {
            for j in  (min_ijk.j - delta)..(max_ijk.j + delta) {
                for k in  (min_ijk.k - 1)..(max_ijk.k + 1) {
                    
                    // check if within cylinder and add
                    let ijk = IJK{i,j,k};
                    let pos = self.lattice.get_xyz(ijk);
                    let pos = XYZ{x: pos.x - center.x, y: pos.y - center.y, z: pos.z - center.z};
                    let distance = ( (pos.y).powi(2) + (pos.z).powi(2) ).sqrt();
                    if -length/2.0 < pos.x && pos.x < length/2.0 && distance < radius {
                        self.add_atom(ijk);
                    }   
                }
            }
        }
    }

    /// Add a duodecant (1/12th) of a cylinder filled with gold.
    pub fn add_cylinder_corner(&mut self, center: XYZ, length: f32, radius: f32, duodecant: u8) {
        
        // calc some parameters first
        let dist = if  length/2.0 > radius {length/2.0}
                   else {radius};
        let min = XYZ{x: center.x - dist, y: center.y - dist, z: center.z - dist}; 
        let max = XYZ{x: center.x + dist, y: center.y + dist, z: center.z + dist}; 
        let min_ijk = self.lattice.get_ijk(min);
        let max_ijk = self.lattice.get_ijk(max);

        // some heuristics again
        let delta_i = (max_ijk.i as isize - min_ijk.i as isize).abs();
        let delta_j = (max_ijk.j as isize - min_ijk.j as isize).abs();
        let delta_k = (max_ijk.k as isize - min_ijk.k as isize).abs();
        let delta = (delta_k/2* std::cmp::max(delta_i, delta_j)) as u16; 

        // iter over all possible positions
        for i in (min_ijk.i - delta)..(max_ijk.i + delta) {
            for j in  (min_ijk.j - delta)..(max_ijk.j + delta) {
                for k in  (min_ijk.k - 1)..(max_ijk.k + 1) {

                    // check if within cylinder
                    let ijk = IJK{i,j,k};
                    let pos = self.lattice.get_xyz(ijk);
                    let pos = XYZ{x: pos.x - center.x, y: pos.y - center.y, z: pos.z - center.z};
                    
                    // check if in duodecant 0-3; 8-11 and add -- duodecants 4-7 are handled separately to due speed issues
                    match duodecant {
                        0 => {
                            let distance = ( (pos.y).powi(2) + (pos.z).powi(2) ).sqrt();
                            if pos.x.abs() < length/2.0 && pos.y >= 0.0 && pos.z >= 0.0 && distance < radius {
                                self.add_atom(ijk);
                            }
                        },
                        1 => {
                            let distance = ( (pos.y).powi(2) + (pos.z).powi(2) ).sqrt();
                            if pos.x.abs() < length/2.0 && pos.y <  0.0 && pos.z >= 0.0 && distance < radius {
                                self.add_atom(ijk);
                            }
                        },
                        2 => {
                            let distance = ( (pos.y).powi(2) + (pos.z).powi(2) ).sqrt();
                            if pos.x.abs() < length/2.0 && pos.y >= 0.0 && pos.z <  0.0 && distance < radius {
                                self.add_atom(ijk);
                            }
                        },
                        3 => {
                            let distance = ( (pos.y).powi(2) + (pos.z).powi(2) ).sqrt();
                            if pos.x.abs() < length/2.0 && pos.y <  0.0 && pos.z <  0.0 && distance < radius {
                                self.add_atom(ijk);
                            }
                        },
                        8 => {
                            let distance = ( (pos.x).powi(2) + (pos.y).powi(2) ).sqrt();
                            if pos.z.abs() < length/2.0 && pos.x >= 0.0 && pos.y >= 0.0 && distance < radius {
                                self.add_atom(ijk);
                            }
                        },
                        9 => {
                            let distance = ( (pos.x).powi(2) + (pos.y).powi(2) ).sqrt();
                            if pos.z.abs() < length/2.0 && pos.x <  0.0 && pos.y >= 0.0 && distance < radius {
                                self.add_atom(ijk);
                            }
                        },
                        10 => {
                            let distance = ( (pos.x).powi(2) + (pos.y).powi(2) ).sqrt();
                            if pos.z.abs() < length/2.0 && pos.x >= 0.0 && pos.y <  0.0 && distance < radius {
                                self.add_atom(ijk);
                            }
                        },
                        11 => {
                            let distance = ( (pos.x).powi(2) + (pos.y).powi(2) ).sqrt();
                            if pos.z.abs() < length/2.0 && pos.x <  0.0 && pos.y <  0.0 && distance < radius {
                                self.add_atom(ijk);
                            }
                        },
                        _ => {}
                    
                    }
                }
            }
        }

        // different heuristics for duodecant 4-7
        let delta = (delta_k/10* std::cmp::max(delta_i, delta_j)) as u16; 
        
        // iter over all possible positions
        for i in (min_ijk.i - delta)..(max_ijk.i + delta) {
            for j in  (min_ijk.j - delta)..(max_ijk.j + delta) {
                for k in  (min_ijk.k - 1)..(max_ijk.k + 1) {
                    
                    // check if within cylinder
                    let ijk = IJK{i,j,k};
                    let pos = self.lattice.get_xyz(ijk);
                    let pos = XYZ{x: pos.x - center.x, y: pos.y - center.y, z: pos.z - center.z};
                    
                    // check if in duodecant 4-7 and add
                    match duodecant {
                        4 => {
                            let distance = ( (pos.z).powi(2) + (pos.x).powi(2) ).sqrt();
                            if pos.y.abs() < length/2.0 && pos.z >= 0.0 && pos.x >= 0.0 && distance < radius {
                                self.add_atom(ijk);
                            }
                        },
                        5 => {
                            let distance = ( (pos.z).powi(2) + (pos.x).powi(2) ).sqrt();
                            if pos.y.abs() < length/2.0 && pos.z <  0.0 && pos.x >= 0.0 && distance < radius {
                                self.add_atom(ijk);
                            }
                        },
                        6 => {
                            let distance = ( (pos.z).powi(2) + (pos.x).powi(2) ).sqrt();
                            if pos.y.abs() < length/2.0 && pos.z >= 0.0 && pos.x <  0.0 && distance < radius {
                                self.add_atom(ijk);
                            }
                        },
                        7 => {
                            let distance = ( (pos.z).powi(2) + (pos.x).powi(2) ).sqrt();
                            if pos.y.abs() < length/2.0 && pos.z <  0.0 && pos.x <  0.0 && distance < radius {
                                self.add_atom(ijk);
                            }
                        },
                        _ => {}
                    
                    }
                }
            }
        }
    }

    
    /// Add a at the top rounded box filled with gold. 
    pub fn add_rounded_box(&mut self, center: XYZ, width: f32, depth: f32, height: f32, radius: f32) {

        // some helper variable
        let inner_x = width/2.0 - radius;
        let inner_y = depth/2.0 - radius;
        let inner_z = height - radius;
        let outer_x = inner_x + radius/2.0;
        let outer_y = inner_y + radius/2.0;

        // upper corners
        let pos = XYZ{x: center.x - inner_x, y: center.y + inner_y, z: center.z + inner_z};
        self.add_sphere_corner(pos, radius, 1);
        let pos = XYZ{x: center.x - inner_x, y: center.y - inner_y, z: center.z + inner_z};
        self.add_sphere_corner(pos, radius, 3);
        let pos = XYZ{x: center.x + inner_x, y: center.y + inner_y, z: center.z + inner_z};
        self.add_sphere_corner(pos, radius, 0);
        let pos = XYZ{x: center.x + inner_x, y: center.y - inner_y, z: center.z + inner_z};
        self.add_sphere_corner(pos, radius, 2);  
        
        // upper rounded edges
        let pos = XYZ{x: center.x - inner_x, y: center.y + 0.0,     z: center.z + inner_z};
        self.add_cylinder_corner(pos, 2.0*inner_y, radius, 6);       
        let pos = XYZ{x: center.x + 0.0    , y: center.y + inner_y, z: center.z + inner_z};
        self.add_cylinder_corner(pos, 2.0*inner_x, radius, 0);
        let pos = XYZ{x: center.x + 0.0    , y: center.y - inner_y, z: center.z + inner_z};
        self.add_cylinder_corner(pos, 2.0*inner_x, radius, 1);
        let pos = XYZ{x: center.x + inner_x, y: center.y + 0.0,     z: center.z + inner_z};
        self.add_cylinder_corner(pos, 2.0*inner_y, radius, 4); 

        // central box
        let pos = XYZ{x: center.x + 0.0,     y: center.y + 0.0,     z: center.z + height/2.0};
        self.add_box(pos, 2.0*inner_x, 2.0*inner_y, height, Atom::Gold);

        // side boxes
        let pos = XYZ{x: center.x + 0.0,     y: center.y + outer_y, z: center.z + inner_z/2.0};
        self.add_box(pos, 2.0*inner_x, radius, inner_z, Atom::Gold);
        let pos = XYZ{x: center.x + 0.0,     y: center.y - outer_y, z: center.z + inner_z/2.0};
        self.add_box(pos, 2.0*inner_x, radius, inner_z, Atom::Gold);
        let pos = XYZ{x: center.x + outer_x, y: center.y + 0.0,     z: center.z + inner_z/2.0};
        self.add_box(pos, radius, 2.0*inner_y, inner_z, Atom::Gold);
        let pos = XYZ{x: center.x - outer_x, y: center.y + 0.0,     z: center.z + inner_z/2.0};
        self.add_box(pos, radius, 2.0*inner_y, inner_z, Atom::Gold);

        // side rounded edges
        let pos = XYZ{x: center.x + inner_x, y: center.y + inner_y, z: center.z + inner_z/2.0};
        self.add_cylinder_corner(pos, inner_z, radius, 8);  
        let pos = XYZ{x: center.x + inner_x, y: center.y - inner_y, z: center.z + inner_z/2.0};
        self.add_cylinder_corner(pos, inner_z, radius, 10);  
        let pos = XYZ{x: center.x - inner_x, y: center.y + inner_y, z: center.z + inner_z/2.0};
        self.add_cylinder_corner(pos, inner_z, radius, 9);  
        let pos = XYZ{x: center.x - inner_x, y: center.y - inner_y, z: center.z + inner_z/2.0};
        self.add_cylinder_corner(pos, inner_z, radius, 11); 
    }



    /// This is the statistics module.
    /// Parameters like the stacking positions, substrate + probabilities can be change at runtime,
    /// and most of the other important ones at compile time via the paramters.rs.
    pub fn statistics(&mut self) -> usize {

        // create file and write header
        let filename = format!("{}_sub{}_stack{:?}_cycles{}_steps{}_p{}.csv", FILENAME, self.substrate_pos ,self.lattice.stacking_faults, NUMBER_OF_CYCLES, STOP_MARKS.len(), self.prob_list_num + 1);
        let mut f = File::create(filename).expect("Unable to create file"); 
        write!(f, "substrate: {} stacking: {:?} cycles: {} steps {} prob_list_log p{} -- {:?}\n", self.substrate_pos ,self.lattice.stacking_faults, NUMBER_OF_CYCLES, STOP_MARKS.len(), self.prob_list_num + 1, self.prob_list_log).expect("Unable to write in file");
        write!(f, "atoms, aspect ratio, k_min, k_max, len1, len2, len ratio\n").expect("Unable to write in file");
        // write!(f, "atoms, aspect ratio, k_min, k_max, a.x, a.y, b.x, b.y, c.x, c.y, d.x, d.y, e.x, e.y, f.x, f.y\n").expect("Unable to write in file");
        
        // create atoms-to-add-list from the defined STOP_MARKS
        let mut atoms_to_add: Vec<usize> = vec![STOP_MARKS[0]];
        for i in 1..STOP_MARKS.len() {
            atoms_to_add.push(STOP_MARKS[i]-STOP_MARKS[i-1]) 
        }
        
        // repeate the flake growth for several cycles
        let mut added_atoms: usize = 0;
        for i in 0..NUMBER_OF_CYCLES {

            // prepare the same zero conditions for every cycle
            added_atoms = 0;
            self.clear();
            self.add_atom(CENTER);

            // start the growth
            println!("{}/{}", i+1 , NUMBER_OF_CYCLES);
            for add_atoms in atoms_to_add.iter() {

                // the growth
                self.random_add(*add_atoms);
                
                // update the variables and print them / write them to the file
                added_atoms += add_atoms;
                let [..,r] = &self.get_size();
                let [ax,ay,bx,by,cx,cy,..] = self.get_hexagon();
                let len1 = ((ax-bx).powi(2) + (ay-by).powi(2)).sqrt();
                let len2 = ((cx-bx).powi(2) + (cy-by).powi(2)).sqrt();
                println!("{:>15} atoms -- aspect ratio: {:>4.1}, length ratio: {:>5.3}", added_atoms, r, len1/(len1+len2));
                write!(f, "{}, {}, {}, {}, {}, {}, {}", added_atoms, r, self.bulk.k_min, self.bulk.k_max, len1, len2, len1/(len1+len2)).unwrap();
                // let hexagon =  self.get_hexagon();
                // for element in hexagon.iter() { 
                //     write!(f, ", {}", element).unwrap() 
                // };
                 write!(f, "\n").unwrap();
            }
        }

        println!(" ...finished");

        // for presentation purposes: return the number of atoms of the last iteration 
        added_atoms
    }

}
