use kiss3d::window::Window;

mod state;          pub use state::*;
mod helpers;        pub use helpers::*;
mod parameters;     
mod lattice;        
mod storage;        
mod crystal;         
mod scene;          
mod planar_scene; 


fn main() {
    let mut window = Window::new_with_size("Flake Growth", 1600, 900);

    let state = AppState::init(&mut window);
    window.render_loop(state)
}
