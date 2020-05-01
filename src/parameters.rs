use crate::helpers::*;

// Using u16 instead of usize for IJK reduces the computation time by ~40%!
// u16 means 2^16 = 65_536 => flakes length of > 20µm allowed. 
//
// surprisingly, going down from u16 to u8 for "k" increases the time by ~40% again!
// the arragement (ijk vs kij vs kji) doesn't really make a difference
//

// Maximal (static) size of the flake 
pub const BITS: u16 = 2;
pub const DIV: u16 = 8 / BITS;
// pub const FLAKE_MAX: IJK = IJK{i:10, j:10, k:10};          // much larger values do not fit inside the stack during initialization -- see crystal.rs 
pub const FLAKE_MAX: IJK = IJK{i:6000, j:6000, k:300};          // much larger values do not fit inside the stack during initialization -- see crystal.rs 
// and resulting center location of the flake -- Don't change!
pub const CENTER: IJK = IJK{i: FLAKE_MAX.i/2, j: FLAKE_MAX.j/2, k: FLAKE_MAX.k/2};

// Stacking faults arrangement -- Shouldn't be larger than FLAKE.MAX.k
pub const STACKING_FAULTS: [u16; 0] = [];
// pub const STACKING_FAULTS: [u16; 1] = [CENTER.k];
// pub const STACKING_FAULTS: [u16; 2] = [CENTER.k-2, CENTER.k+2];
// pub const STACKING_FAULTS: [u16; 3] = [CENTER.k-3, CENTER.k, CENTER.k+3];
// pub const STACKING_FAULTS: [u16; 4] = [CENTER.k-3, CENTER.k, CENTER.k+6, CENTER.k+8];

// Number of used vacancy kinds
pub const VAC_LISTS: usize = 9;
// and associated probabilities of their interaction
pub const PROB_LIST_NUM: usize = 2;                                                
pub const PROB_LIST: [[usize; VAC_LISTS]; 4] = [[0, 0, 1, 1_000, 100_000, 1_000_000, 10_000_000, 100_000_000, 1000_000_000],
                                                [0, 0, 1, 1_000, 1_000_000, 1_000_000_000, 10_000_000_000, 100_000_000_000, 1_000_000_000_000],
                                                [0, 0, 1, 10_000, 100_000_000, 1_000_000_000_000, 1_000_000_000_000, 1_000_000_000_000, 1_000_000_000_000],
                                                [0, 0, 1, 100_000, 1_000_000_000, 1_000_000_000_000, 1_000_000_000_000, 1_000_000_000_000, 1_000_000_000_000]];

// and the colors
pub const GOLD: Color =  Color(1.6,1.3,0.0);
pub const DIRT: Color =  Color(0.8,1.0,0.3);
pub const ATOM_COLORS: [Color; 3] = [Color(1.0,1.0,0.0), Color(1.0,0.8,0.3), Color(0.8,0.8,0.8)];
pub const LAYER_COLORS: [Color; 3] = [Color(0.8,0.8,0.0), Color(0.8,0.6,0.2), Color(0.6,0.6,0.6)];
pub const VAC_COLORS: [Color; 9] = [Color(0.4,1.0,0.4), Color(0.4,0.4,1.0), Color(1.0,0.8,0.8), Color(1.2,0.6,0.6), Color(1.4,0.4,0.4), Color(1.6,0.2,0.2), Color(2.0,0.0,0.0), Color(4.0,0.0,0.0), Color(8.0,0.0,0.0)];

// Diameter of a gold atom in a fcc lattice in nm
pub const DIAMETER: f32 = 0.40782;             

// statistics mode
pub const FILENAME: &str = "test";
pub const NUMBER_OF_CYCLES: usize = 100;
// pub const ATOMS_LIST: [usize; 5] = [100, 1_000, 10_000, 100_000, 1_000_000];
// pub const STOP_MARKS: [usize; 10] = [100, 300, 1_000, 3_000, 10_000, 30_000, 100_000, 300_000, 1_000_000, 3_000_000];
// pub const STOP_MARKS: [usize; 21] = [100, 180, 320, 560, 1_000, 1_800, 3_200, 5_600, 10_000, 18_000, 32_000, 56_000, 100_000, 180_000, 320_000, 560_000, 1_000_000, 1_800_000, 3_200_000, 5_600_000, 10_000_000];
pub const STOP_MARKS: [usize; 51] = [100, 130, 160, 200, 250, 320, 400, 500, 630, 790, 1_000, 1_300, 1_600, 2_000, 2_500, 3_200, 4_000, 5_000, 6_300, 7_900, 10_000, 13_000, 16_000, 20_000, 25_000, 32_000, 40_000, 50_000, 63_000, 79_000, 100_000, 130_000, 160_000, 200_000, 250_000, 320_000, 400_000, 500_000, 630_000, 790_000, 1_000_000, 1_300_000, 1_600_000, 2_000_000, 2_500_000, 3_200_000, 4_000_000, 5_000_000, 6_300_000, 7_900_000, 10_000_000];
