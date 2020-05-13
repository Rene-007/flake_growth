use kiss3d::window::Window;

mod world;          pub use world::*;
mod helpers;        pub use helpers::*;
mod parameters;     
mod lattice;        
mod storage;        
mod crystal;         
mod scene;          
mod planar_scene; 


fn main() {
    let mut window = Window::new_with_size("Flake Growth", 1600, 900);
    
    let mut world = World::new(&mut window);
    world.add_random_atoms(&mut window, true, 2_000);
    window.render_loop(world)
}
