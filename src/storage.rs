/*!
 Storage back-end of the crystal struct

The **storage struct** provides:
* Bulk: A lean 3D storage for the atom (empty, gold or dirt) of each position of the crystal which e.g. can hold 10.8 billion positions within 2.5GB.
* SurfaceAtoms: A list which holds all atom positions of the surface atoms.
* Vacancies: A list of lists of the positions of all vacancies depending on their coordination number.

Bulk is implemented via an ndarray, its size is (so far) predefined at compile time but it actually does "dynammically" use the memory due to the advantegeous handling of zeroed pages by the OS :-)

SurfaceAtoms and Vacancies utilize a BTreeSet datastructure to quickly find (random) locations within them.
*/

use std::collections::BTreeSet;
use ndarray::{Array3};

use crate::helpers::*;
use crate::parameters::*;

#[cfg(target_arch = "wasm32")]
use crate::println;

pub enum Atom {
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
        println!("Array shape {:?} with DIV {:?}", storage.shape(), DIV);                     
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
        // To reset the storage we have two possibilities:
        // The quick one seems to need twice the memory size very briefly during allocation:
        self.storage = Array3::<u8>::zeros((0, 0, 0));                                  // This is the solution for freeing the memory, first!
        self.storage = Array3::<u8>::zeros((FLAKE_MAX.i as usize, FLAKE_MAX.j as usize, (FLAKE_MAX.k/DIV + 1) as usize)); 

        // And the second one is safer but takes longer:
        // for i in self.i_min..=self.i_max {
        //     for j in self.j_min..=self.j_max {
        //         for k in self.k_min..=self.k_max {
        //             let ijk = IJK{i, j, k};
        //             self.set(ijk, Atom::Empty);
        //         }
        //     }
        // }
        // However be careful with the second method, too: As some memory pages are already touched the real memory usage will be non-zero at the beginning.
        // And it seems to be a tiny bit slower to grow flakes afterwards. Possible explanation: As each flake has a slightly different shape it will also use a
        // slightly different memory region and together with the already touched (but now unused) pages the overall memory size is larger. And this makes
        // some memory handling slower.

        self.number_of_atoms = 0;
        self.i_min = CENTER.i;
        self.i_max = CENTER.i; 
        self.j_min = CENTER.j; 
        self.j_max = CENTER.j; 
        self.k_min = CENTER.k; 
        self.k_max = CENTER.k;
    }

    pub fn set(&mut self, ijk: IJK, atom: Atom) {
        // translate enum to value
        let value: u8 = match atom {
            Atom::Empty    => {0},
            Atom::Gold     => {1},
            Atom::Dirt     => {2}
        };

        // save value
        // read data into a virtual register
        let mut register = self.storage[[ijk.i as usize, ijk.j as usize, (ijk.k/DIV) as usize]];
        // update the right bits in the byte/word/longword or whatever will be used in the end
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

    pub fn get(&self, ijk: IJK, atom: Atom) -> bool {
        let value = self.storage[[ijk.i as usize, ijk.j as usize, (ijk.k/DIV)  as usize]]; 
        let pos = ijk.k%DIV;                                                           // calculate position in byte/word/longword or whatever we will use in the end
        let value = value.wrapping_shr((pos*BITS) as u32) & self.unit;                  // select the right bits

        match atom {
            Atom::Empty    => {value == 0},
            Atom::Gold     => {value == 1},
            Atom::Dirt     => {value == 2},
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
