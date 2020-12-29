use kiss3d::window::Window;
use kiss3d::light::Light;
use nalgebra::Point3;

mod world;          pub use world::*;
mod helpers;        pub use helpers::*;
mod parameters;     pub use parameters::*;
mod lattice;        
mod storage;        
mod crystal;         
mod scene;          
mod planar_scene; 

#[cfg(feature = "sidebar")]
mod sidebar;        
#[cfg(feature = "sidebar")]
pub use sidebar::*;

fn main() {
    let mut window = Window::new_with_size("Flake Growth", 1600, 900);
    window.set_light(Light::Absolute(Point3::new(-300.0, 300.0, 300.0)));
    window.set_background_color(1.0, 1.0, 1.0);
    
    let mut world = World::new(&mut window);
    println!("Stacking faults {:?}", STACKING_FAULTS);
    world.add_random_atoms(&mut window, true, 1);
    // world.add_random_atoms(&mut window, true, 2_000);
    window.render_loop(world)
}
