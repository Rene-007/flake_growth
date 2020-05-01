use std::collections::BTreeSet;
use ndarray::{Array3};

use crate::helpers::*;
use crate::parameters::*;

pub enum State {
    Empty,
    Gold,
    Dirt,
}

pub struct Bulk { 
    storage: Array3<u8>,
    unit: u8,
    pub number_of_atoms: usize,
    pub i_min: u16, 
    pub i_max: u16, 
    pub j_min: u16, 
    pub j_max: u16, 
    pub k_min: u16, 
    pub k_max: u16
}


impl Bulk {
    pub fn new() -> Self {
        let storage = Array3::<u8>::zeros((FLAKE_MAX.i as usize, FLAKE_MAX.j as usize, (FLAKE_MAX.k/DIV + 1) as usize));   
        println!("Array shape {:?}", storage.shape());                     
        Bulk { 
            storage, 
            unit: (2u16.pow(BITS as u32) - 1) as u8,                         // needed for the bitmask further down and should only be calculated once
            number_of_atoms: 0, 
            i_min: CENTER.i, 
            i_max: CENTER.i, 
            j_min: CENTER.j, 
            j_max: CENTER.j, 
            k_min: CENTER.k, 
            k_max: CENTER.k 
        }
    }

    pub fn clear(&mut self) {
        for i in self.i_min..=self.i_max {
            for j in self.j_min..=self.j_max {
                for k in self.k_min..=self.k_max {
                    let ijk = IJK{i, j, k};
                    self.set(ijk, State::Empty);
                }
            }
        }
        self.number_of_atoms = 0;
    }

    pub fn set(&mut self, ijk: IJK, state: State) {
        // translate enum to value
        let value: u8 = match state {
            State::Empty    => {0},
            State::Gold     => {1},
            State::Dirt     => {2}
        };

        // save value
        // read data into a virtual register
        let mut register = self.storage[[ijk.i as usize, ijk.j as usize, (ijk.k/DIV) as usize]];
        // update the right bits in the byte/word/longword or whatever will be usec in the end
        let pos = ijk.k%DIV;                                                           // calculate the position
        let change = value.wrapping_shl((pos*BITS) as u32);                             // move bit to the right position
        let bitmask = !self.unit.wrapping_shl((pos*BITS) as u32);                       // construct a bitmask for the same position
        register = (register & bitmask) + change;                                           // update the register
        self.storage[[ijk.i as usize, ijk.j as usize, (ijk.k/DIV) as usize]] = register;    // write data back  
        
        // update extrema
       if value > 0 {   
            self.number_of_atoms += 1;      
            if ijk.i < self.i_min { self.i_min = ijk.i }
            if ijk.i > self.i_max { self.i_max = ijk.i }
            if ijk.j < self.j_min { self.j_min = ijk.j }
            if ijk.j > self.j_max { self.j_max = ijk.j }
            if ijk.k < self.k_min { self.k_min = ijk.k }
            if ijk.k > self.k_max { self.k_max = ijk.k }
        }
    }

    pub fn get(&self, ijk: IJK, state: State) -> bool {
        let value = self.storage[[ijk.i as usize, ijk.j as usize, (ijk.k/DIV)  as usize]]; 
        let pos = ijk.k%DIV;                                                           // calculate position in byte/word/longword or whatever we will use in the end
        let value = value.wrapping_shr((pos*BITS) as u32) & self.unit;                  // select the right bits

        match state {
            State::Empty    => {value == 0},
            State::Gold     => {value == 1},
            State::Dirt     => {value == 2},
        }
    }

}




// Since using BTreeSet this abstraction is actually not necessary anymore
pub struct SurfaceAtoms {
    pub list: BTreeSet<IJK>
}

impl SurfaceAtoms {
    pub fn new() -> Self {
        SurfaceAtoms{ list: BTreeSet::new() }
    }

    pub fn add(&mut self, ijk: IJK) {
        self.list.insert(ijk);
    }

    pub fn remove(&mut self, ijk: IJK) {
        self.list.take(&ijk);   
    }
}



pub struct Vacancies {
    pub list: Vec<BTreeSet<IJK>>,
}

impl Vacancies {
    pub fn new() -> Self {
        let mut list = Vec::with_capacity(VAC_LISTS);
        for _i in 0..VAC_LISTS {
            list.push(BTreeSet::new());
        }
        Vacancies{ list }
    }

    pub fn recursive_remove(&mut self, ijk: IJK, index: usize) {
        // Recursion over all lists from first to last
        match self.list[index].take(&ijk)  {
            Some(_value) => {
                //  println!("removal of {:?} in list {:?}", _value, index)
            }
            None => {
                if (index + 1) < VAC_LISTS {
                    self.recursive_remove(ijk, index + 1);
                }
            } 
        }
    } 
}
