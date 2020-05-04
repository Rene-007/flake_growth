// First a small struct definition for holding the constants
#[derive(Copy,Clone,Debug,PartialEq, Eq, PartialOrd, Ord)]
pub struct IJK {pub i: u16, pub j: u16, pub k: u16}
// #[repr(C, align(8))]
// pub struct IJK {pub k: u8, pub j: u16, pub i: u16}         //  doesn't make a speed difference



// A compact struct for the colors
#[derive(Copy,Clone,Debug)]
pub struct Color(pub f32, pub f32, pub f32);