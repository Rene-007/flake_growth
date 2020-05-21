// Using u16 instead of usize for IJK reduces the computation time by ~40%!
// u16 means 2^16 = 65_536 => flakes length of > 20µm allowed. 
//
// surprisingly, going down from u16 to u8 for "k" increases the time by ~40% again!
// the arragement (ijk vs kij vs kji) doesn't really make a difference

// First a small struct definition for holding the constants
#[derive(Copy,Clone,Debug,PartialEq, Eq, PartialOrd, Ord)]
pub struct IJK {pub i: u16, pub j: u16, pub k: u16}
// #[repr(C, align(8))]
// pub struct IJK {pub k: u8, pub j: u16, pub i: u16}         //  doesn't make a speed difference

#[derive(Copy,Clone,Debug)]
pub struct XYZ {pub x: f32, pub y: f32, pub z: f32}

// A compact struct for the colors
#[derive(Copy,Clone,Debug)]
pub struct Color(pub f32, pub f32, pub f32);



#[cfg(target_arch = "wasm32")]
extern crate stdweb;

// The stdweb docs say one should always try to use println! but somehow this doesn't work for us.
// So, here is a macro which replaces println! with a console.log output when compiled for wasm.
// The indirection over format! is for things such as "{:?}" which the console.log cannot handle.
#[cfg(target_arch = "wasm32")]
#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => { stdweb::console!(log, "{}", format!($($arg)*)); }
}
