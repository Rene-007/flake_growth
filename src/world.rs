/*!
Init of the world, event loop and associated functions

In order to get the whole program to work in wasm, too, an AppState had to be implemented and that's the reason for a seperate module instead of just putting everything in the main function.

Events are handle in State::step and all the associated functions are implemented in World where basically the whole program is initiallized.
*/

// use std::time::Instant;  // doesn't work with wasm
use instant::Instant;
use separator::Separatable;

use crate::helpers::*;
use crate::parameters::*;
use crate::lattice::*;
use crate::storage::*;
use crate::crystal::*;
use crate::scene::*;
use crate::planar_scene::*;

use kiss3d::camera::{Camera, ArcBall};
use kiss3d::planar_camera::PlanarCamera;
use kiss3d::post_processing::PostProcessingEffect;
use kiss3d::event::{Action, Key, WindowEvent};
use kiss3d::window::{State, Window};
// use nalgebra::{Vector3,Point3};
use nalgebra::Point3;

#[cfg(target_arch = "wasm32")]
use crate::println;

pub struct World {
    overlay: PlanarScene,
    scene: Scene,
    camera: ArcBall,
    lattice: Lattice,
    flake: Crystal,
    i: u16,
    j: u16,
    k: u16,
}


impl State for World {
    fn cameras_and_effect( &mut self,) -> (
        Option<&mut dyn Camera>,
        Option<&mut dyn PlanarCamera>,
        Option<&mut dyn PostProcessingEffect>,
    ) {
        (Some(&mut self.camera), None, None)
    }


    fn step(&mut self, window: &mut Window) {
        if self.scene.show.hexagon {
            self.scene.draw_hexgon_outline(window);                                 // lines have to be redrawn every time   
        }
        if self.overlay.show {
            self.overlay.draw_scene(window, &self.flake);                           // text has to be redrawn every time, too
            self.overlay.update_indicators(window, &self.flake);                    // update length of indicators
        }
        if self.overlay.show_help {                                                  
            self.overlay.show_help(window);                                         // draw help overlay
        }
        if self.scene.show.current {                                                
            window.remove_node(&mut self.scene.current);                        // highlight the last atom by drawing it again in red 
            self.scene.current = window.add_group();                                
            add_atom_to_group(&mut self.scene.current, &self.lattice.position(IJK{i: self.i, j: self.j, k:self.k}), Color(1.0, 0.0, 0.0));
        }
        for event in window.events().iter() {
            match event.value {
                WindowEvent::Key(key, Action::Press, _) => {     
                    match key { 
                        // special actions
                        Key::Back =>    self.back(window),                          // back to start
                        Key::Period =>  self.back(window),                          // back to start for wasm
                        Key::P =>       self.flake.next_prob_list(),                // change probabilities
                        
                        #[cfg(not(target_arch = "wasm32"))]
                        Key::Comma =>   self.statistics(window),                    // some statistics
                     
                        // interact with (planar) scene
                        Key::Space =>   self.show_hide_help(),                      // show/hide help
                        Key::R =>       self.show_hide_indicators(),                // show/hide indicators
                        Key::S =>       self.highlight_current_atom(window),        // highlight current atom
                        Key::G =>       self.show_hide_gold(window),                // show/hide gold atoms
                        Key::T =>       self.show_hide_dirt(window),                // show/hide dirt atoms
                        Key::V =>       self.visualize_stacking(window),            // visualize layers on/off 
                        Key::B =>       self.show_hide_wireframe(),                 // show/hide wireframe (box)
                        Key::H =>       self.show_hide_hexagon(),                   // show/hide hexagon
                        Key::Minus =>   self.show_hide_substrate(),                 // show/hide substrate         

                        // add some special geometries
                        Key::L =>       self.add_gold_layer(window),                // add gold layer
                        Key::O =>       self.add_dirt_layer(window),                // add dirt layer on top
                        // Key::Z =>       self.add_box(window),                       // add box     
                        Key::Home =>    self.add_sphere(window),                    // add sphere     
                        Key::End =>     self.add_cylinder(window),                  // add cylinder                        
                        Key::Delete =>  self.add_rounded_monomer_antenna(window),   // add rounded monomer antenna
                        Key::I =>       self.add_rounded_dipole_antenna(window),    // add rounded Dipole antenna  
                        Key::K =>       self.add_dipole_antenna(window),            // add Dipole antenna
                        Key::U =>       self.add_rounded_jord_antenna(window),      // add rounded Jord antenna
                        Key::J =>       self.add_jord_antenna(window),              // add Jord antenna
                        Key::Down =>    self.add_remove_substrate(window),          // add/remove substrat below lowest vacancies layer    
                        // Key::N =>       self.add_column(window),                    // add column step by step    
                        
                        // tweak stacking
                        Key::Up =>      self.reset_stacking(window),                // reset stacking
                        Key::PageUp =>  self.add_stacking_fault_top(window),        // add stacking faults on top
                        Key::PageDown =>self.add_stacking_fault_bottom(window),     // add stacking faults on bottom

                        // show/hide vacancies
                        Key::F1 =>      self.show_hide_vacancy(window, 1),                         
                        Key::F2 =>      self.show_hide_vacancy(window, 2),                          
                        Key::F3 =>      self.show_hide_vacancy(window, 3),                                            
                        Key::F4 =>      self.show_hide_vacancy(window, 4),                                            
                        Key::F5 =>      self.show_hide_vacancy(window, 5),                                               
                        Key::F6 =>      self.show_hide_vacancy(window, 6),                                            
                        Key::F7 =>      self.show_hide_vacancy(window, 7),                                             
                        Key::F8 =>      self.show_hide_vacancy(window, 8), 
                        Key::F9 =>      self.show_hide_vacancy(window, 9),            
                        Key::F  =>      self.show_hide_all_vacancies(window),       // show-hide all

                        // initiating atoms adding at random positions
                        Key::Key1 =>    self.add_random_atoms(window, true , 1),
                        Key::Key2 =>    self.add_random_atoms(window, true , 10),
                        Key::Key3 =>    self.add_random_atoms(window, true , 100),
                        Key::Key4 =>    self.add_random_atoms(window, true , 1_000),
                        Key::Key5 =>    self.add_random_atoms(window, true , 10_000),
                        Key::Key6 =>    self.add_random_atoms(window, false, 100_000),
                        Key::Key7 =>    self.add_random_atoms(window, false, 1_000_000),
                        Key::Key8 =>    self.add_random_atoms(window, false, 10_000_000),
                        Key::Key9 =>    self.add_random_atoms(window, false, 100_000_000),  

                        // go to position and request a new atom there
                        Key::X =>       self.add_atom(window, IJK{i: self.i,     j: self.j,     k: self.k - 1}),    
                        Key::W =>       self.add_atom(window, IJK{i: self.i,     j: self.j,     k: self.k + 1}),                    
                        Key::D =>       self.add_atom(window, IJK{i: self.i + 1, j: self.j,     k: self.k    }),                           
                        Key::A =>       self.add_atom(window, IJK{i: self.i - 1, j: self.j,     k: self.k    }),                    
                        Key::E =>       self.add_atom(window, IJK{i: self.i,     j: self.j + 1, k: self.k    }),                    
                        Key::Y =>       self.add_atom(window, IJK{i: self.i,     j: self.j - 1, k: self.k    }),       
                        Key::C =>       self.add_atom(window, IJK{i: self.i + 1, j: self.j - 1, k: self.k    }),             
                        Key::Q =>       self.add_atom(window, IJK{i: self.i - 1, j: self.j + 1, k: self.k    }),  
                        
                        _=> {}          // remaining keys
                    }
                }
            _=> {}                      // remaining events
            }
        }
    }

}


impl World {
    pub fn new(window: &mut Window) -> Self {
        // init lattice and flake  
        let lattice = Lattice::new(STACKING_FAULTS.to_vec(), DIAMETER);
        let flake = Crystal::new(lattice.clone());

        // init OpenGL scene
        let scene = Scene::new(window, lattice.clone());
        let overlay = PlanarScene::new(window);
        let camera = ArcBall::new(Point3::new(-15.0, 7.5, 0.0), Point3::origin());
    
        // start with a single atom in the middle
        let IJK { i, j, k } = CENTER;

        World{
            overlay,
            scene,
            camera,
            lattice,
            flake,
            i,
            j,
            k,
        }
    }

    fn back(&mut self, window: &mut Window) {
        self.flake.clear();
        self.i = CENTER.i;           // with "let IJK{i,j,k} = ijk" the scope just here
        self.j = CENTER.j;  
        self.k = CENTER.k;
        if self.flake.add_atom(IJK{i: self.i, j: self.j, k:self.k}) { 
            self.scene.update_surface(window, &self.flake);
            self.scene.update_dirt(window, &self.flake);
            self.scene.update_vacancies(window, &self.flake, false);
        }
    }

    fn show_hide_help(&mut self) {
        self.overlay.show_help = !self.overlay.show_help;
        self.overlay.help.set_visible(self.overlay.show_help);
    }

    fn show_hide_indicators(&mut self) {
        self.overlay.show = !self.overlay.show;
        self.overlay.scene.set_visible(self.overlay.show);
        self.overlay.layers.set_visible(self.overlay.show);
    }

    fn highlight_current_atom(&mut self, window: &mut Window) {
        if self.scene.show.current {
            window.remove_node(&mut self.scene.current);
        }
        else
        {
            self.scene.current = window.add_group();
            add_atom_to_group(&mut self.scene.current, &self.lattice.position(IJK{i: self.i, j: self.j, k:self.k}), Color(1.0, 0.0, 0.0));

        }
        self.scene.show.current = !self.scene.show.current;
    }

    fn show_hide_gold(&mut self, window: &mut Window) {
        if self.scene.show.surface {
            // window.remove(&mut self.scene.surface);
            window.remove_node(&mut self.scene.surface);
            self.scene.surface.set_visible(false);
        }
        else {   
            self.scene.surface = window.add_group();
            if self.scene.visual_layers {
                // for ijk in self.flake.surface.list.clone() {
                //     add_atom_to_group(
                //         &mut self.scene.surface, &self.lattice.position(ijk), ATOM_COLORS[(self.lattice.stacking.pos[ijk.k as usize].rem_euclid(3)) as usize])    
                // }
                self.flake.surface.list.clone().iter().for_each(|&ijk| add_atom_to_group(
                    &mut self.scene.surface, &self.lattice.position(ijk), ATOM_COLORS[(self.lattice.stacking.pos[ijk.k as usize].rem_euclid(3)) as usize])
                );   
            }
            else {
                self.flake.surface.list.clone().iter().for_each(|&ijk| add_atom_to_group(&mut self.scene.surface, &self.lattice.position(ijk), GOLD) );

            }
            self.scene.surface.set_visible(true);
        }
        self.scene.show.surface = !self.scene.show.surface;
    }

    fn show_hide_dirt(&mut self, window: &mut Window) {
        if self.scene.show.dirt {
            window.remove_node(&mut self.scene.dirt);
            self.scene.dirt.set_visible(false);
        }
        else {   
            self.scene.dirt = window.add_group();
            self.flake.dirt.list.clone().iter().for_each(|&ijk| add_atom_to_group(&mut self.scene.dirt, &self.lattice.position(ijk), DIRT) );
            self.scene.dirt.set_visible(true);
        }
        self.scene.show.dirt = !self.scene.show.dirt;
    }

    fn visualize_stacking(&mut self, window: &mut Window) {
        self.scene.visual_layers = !self.scene.visual_layers;
        self.scene.update_surface(window, &self.flake);
    }

    fn show_hide_wireframe(&mut self) {
        self.scene.show.wireframe = !self.scene.show.wireframe;
        self.scene.wireframe.set_visible(self.scene.show.wireframe);
    }

    fn show_hide_hexagon(&mut self) {
        self.scene.show.hexagon = !self.scene.show.hexagon;
        self.scene.hexagon.set_visible(self.scene.show.hexagon);
    }

    fn show_hide_substrate(&mut self) {
        self.scene.show.substrate = !self.scene.show.substrate;
        self.scene.substrate.set_visible(self.scene.show.substrate);
    }

    fn add_gold_layer(&mut self, window: &mut Window) {
        self.flake.add_layer(IJK{i: self.i, j: self.j, k:self.k}, 7, Atom::Gold);
        self.scene.update_surface(window, &self.flake);
        self.scene.update_vacancies(window, &self.flake, false);
    }

    fn add_dirt_layer(&mut self, window: &mut Window) {
        self.flake.add_layer(IJK{i: self.i, j: self.j, k: self.flake.extrema_ijk.z_max.k+1}, 7, Atom::Dirt);
        self.scene.update_dirt(window, &self.flake);
        self.scene.update_vacancies(window, &self.flake, false);
    }

    // fn add_box(&mut self, window: &mut Window) {
    //     self.flake.clear();
    //     let pos = XYZ{x: 0.0, y: 5.0, z: 0.0};
    //     self.flake.add_box(pos, 10.0, 5.0, 3.0, Atom::Gold);
    //     self.camera = ArcBall::new(Point3::new(-45.0, 22.5, 0.0), Point3::origin());
    //     self.scene.update_surface(window, &self.flake);
    //     self.scene.update_vacancies(window, &self.flake, false);
    // }

    // fn add_column(&mut self, window: &mut Window) {
    //     // if self.k == CENTER.k {self.lattice.origin = IJK{i: self.i, j: self.j , k: self.k}}
    //     if self.lattice.origin == CENTER {self.lattice.origin = IJK{i: self.i, j: self.j , k: self.k}}
    //     let ijk = self.lattice.kplus(IJK{i: self.i, j: self.j , k: self.k});
    //     self.i = ijk.i;
    //     self.j = ijk.j;
    //     self.k = ijk.k;

    //     if self.flake.add_atom(ijk) { 
    //         self.scene.update_surface(window, &self.flake);
    //         self.scene.update_vacancies(window, &self.flake, false);
    //     } 
    //     self.scene.update_boundaries(window, &self.flake);
    // }

    fn add_sphere(&mut self, window: &mut Window) {
        self.flake.clear();
        let pos = XYZ{x: 0.0, y: 0.0, z: 0.0};
        self.flake.add_sphere(pos, 5.0);
        self.camera = ArcBall::new(Point3::new(-45.0, 22.5, 0.0), Point3::origin());
        self.scene.update_surface(window, &self.flake);
        self.scene.update_vacancies(window, &self.flake, false);
    }

    fn add_cylinder(&mut self, window: &mut Window) {
        self.flake.clear();
        let pos = XYZ{x: 0.0, y: 0.0, z: 0.0};
        self.flake.add_cylinder(pos, 20.0, 5.0);
        self.camera = ArcBall::new(Point3::new(-45.0, 22.5, 0.0), Point3::origin());
        self.scene.update_surface(window, &self.flake);
        self.scene.update_vacancies(window, &self.flake, false);
    }

    fn add_rounded_monomer_antenna(&mut self, window: &mut Window) {
        self.flake.clear();
        // antenna arm
        let pos = XYZ{x: 0.0, y: 0.0, z: -3.0};
        self.flake.add_rounded_box(pos, 20.0, 10.0, 6.0, 3.0);
        // other stuff
        self.camera = ArcBall::new(Point3::new(-45.0, 22.5, 0.0), Point3::origin());
        self.scene.update_surface(window, &self.flake);
        self.scene.update_vacancies(window, &self.flake, false);
    }

    fn add_dipole_antenna(&mut self, window: &mut Window) {
        self.flake.clear();
        // left arm
        let pos = XYZ{x: -15.0, y: 0.0, z: 0.0};
        self.flake.add_box(pos, 20.0, 10.0, 6.0, Atom::Gold);
        // left arm scond level
        let pos = XYZ{x: -15.0, y: 0.0, z: 3.5};
        self.flake.add_box(pos, 18.0, 8.0, 1.0, Atom::Gold);
        // right arm
        let pos = XYZ{x: 15.0, y: 0.0, z: 0.0};
        self.flake.add_box(pos, 20.0, 10.0, 6.0, Atom::Gold);
        // other stuff
        self.camera = ArcBall::new(Point3::new(-45.0, 22.5, 0.0), Point3::origin());
        self.scene.update_surface(window, &self.flake);
        self.scene.update_vacancies(window, &self.flake, false);
    }

    fn add_rounded_dipole_antenna(&mut self, window: &mut Window) {
        self.flake.clear();
        // left arm
        let pos = XYZ{x: -15.0, y: 0.0, z: -3.0};
        self.flake.add_rounded_box(pos, 20.0, 10.0, 6.0, 3.0);
        // right arm
         let pos = XYZ{x: 15.0, y: 0.0, z: -3.0};
        self.flake.add_rounded_box(pos, 20.0, 10.0, 7.0, 3.5);
        // other stuff
        self.camera = ArcBall::new(Point3::new(-45.0, 22.5, 0.0), Point3::origin());
        self.scene.update_surface(window, &self.flake);
        self.scene.update_vacancies(window, &self.flake, false);
    }

    fn add_jord_antenna(&mut self, window: &mut Window) {
        self.flake.clear();
        // left arm
        let pos = XYZ{x: -15.0, y: 0.0, z: 0.0};
        self.flake.add_box(pos, 20.0, 10.0, 6.0, Atom::Gold);
        // right arm
        let pos = XYZ{x: 15.0, y: 0.0, z: 0.0};
        self.flake.add_box(pos, 20.0, 10.0, 6.0, Atom::Gold);
        // left connector;
        let pos = XYZ{x: -15.0, y: 25.0, z: -1.5};
        self.flake.add_box(pos, 5.0, 40.0, 3.0, Atom::Gold);
        // right connector;
        let pos = XYZ{x: 15.0, y: -25.0, z: -1.5};
        self.flake.add_box(pos, 5.0, 40.0, 3.0, Atom::Gold);
        // other stuff
        self.camera = ArcBall::new(Point3::new(-60.0, 60.0, 0.0), Point3::origin());
        self.scene.update_surface(window, &self.flake);
        self.scene.update_vacancies(window, &self.flake, false);
    }

    fn add_rounded_jord_antenna(&mut self, window: &mut Window) {
        self.flake.clear();
        // left arm
        let pos = XYZ{x: -14.0, y: 0.0, z: -3.0};
        self.flake.add_rounded_box(pos, 20.0, 10.0, 6.0, 3.0);
        // right arm
        let pos = XYZ{x: 14.0, y: 0.0, z: -3.0};
        self.flake.add_rounded_box(pos, 20.0, 10.0, 6.0, 3.0);
        // left connector
        let pos = XYZ{x: -14.0, y: 25.0, z: -3.0};
        self.flake.add_rounded_box(pos, 5.0, 42.0, 3.0, 2.0);
        // right connector
        let pos = XYZ{x: 14.0, y: -25.0, z: -3.0};
        self.flake.add_rounded_box(pos, 5.0, 42.0, 3.0, 2.0);
        // left protector
        let pos = XYZ{x: -14.0, y: 25.0, z: -1.5};
        self.flake.add_box(pos, 6.0, 44.0, 4.0, Atom::Dirt);
        // right protector
        let pos = XYZ{x: 14.0, y: -25.0, z: -1.5};
        self.flake.add_box(pos, 6.0, 44.0, 4.0, Atom::Dirt);
        // waist for the left arm
        let pos = XYZ{x: -14.0, y: 0.0, z: 0.0};
        self.flake.add_box(pos, 10.0, 12.0, 7.0, Atom::Dirt);
        // other stuff
        self.camera = ArcBall::new(Point3::new(-60.0, 60.0, 0.0), Point3::origin());
        self.scene.update_surface(window, &self.flake);
        self.scene.update_dirt(window, &self.flake);
        self.scene.update_vacancies(window, &self.flake, false);
    }

    fn add_remove_substrate(&mut self, window: &mut Window) {
        if self.flake.substrate_pos == 1 {
            self.flake.substrate_pos = self.flake.extrema_ijk.z_min.k - 1;
            self.flake.update_vacancies();
            self.scene.update_vacancies(window, &self.flake, false);
            self.scene.add_substrate(&mut self.flake);
            self.scene.show.substrate = true;
            self.scene.substrate.set_visible(self.scene.show.substrate);
            println!("Substrate added at: {:?}", self.flake.substrate_pos);
        }
        else  {
            self.flake.substrate_pos = 1;
            self.flake.update_vacancies();
            self.scene.show.substrate = false;
            self.scene.substrate.set_visible(self.scene.show.substrate);
        }
    }

    fn reset_stacking(&mut self, window: &mut Window) {
        self.lattice = Lattice::new([].to_vec(), DIAMETER);
        self.flake.lattice = self.lattice.clone();
        self.flake.update_vacancies();
        self.scene.lattice = self.lattice.clone();
        self.scene.update_surface(window, &self.flake);
        self.scene.update_vacancies(window, &self.flake, false);
        println!("Stacking faults {:?}", self.lattice.stacking_faults);
    }
    
    fn add_stacking_fault_top(&mut self, window: &mut Window) {
        let mut stacking = self.flake.lattice.stacking_faults.clone();
        let new_fault = self.flake.extrema_ijk.z_max.k + 1;
        match stacking.binary_search(&(new_fault)) {
            Ok(_pos) => {} // element already in vector @ `pos` 
            Err(pos) => {
                stacking.insert(pos, new_fault);
                self.lattice = Lattice::new(stacking, DIAMETER);
                self.flake.lattice = self.lattice.clone();
                self.flake.update_vacancies();
                self.scene.lattice = self.lattice.clone();
                self.scene.update_vacancies(window, &self.flake, false);
            },
        }
        println!("Stacking faults {:?}", self.lattice.stacking_faults);
    }

    fn add_stacking_fault_bottom(&mut self, window: &mut Window) {
        let mut stacking = self.flake.lattice.stacking_faults.clone();
        let new_fault = self.flake.extrema_ijk.z_min.k + 1;
        match stacking.binary_search(&(new_fault)) {
            Ok(_pos) => {} // element already in vector @ `pos` 
            Err(pos) => {
                stacking.insert(pos, new_fault);
                // insert/remove a stacking fault down at the bottom to not mess up with the representation
                match stacking.binary_search(&(0)) {
                    Ok(_pos) => {stacking.remove(0);} 
                    Err(pos) => {stacking.insert(pos, 0)}
                }
                self.lattice = Lattice::new(stacking, DIAMETER);
                self.flake.lattice = self.lattice.clone();
                self.flake.update_vacancies();
                self.scene.lattice = self.lattice.clone();
                self.scene.update_vacancies(window, &self.flake, false);
            },
        }
        println!("Stacking faults {:?}", self.lattice.stacking_faults);
    }

    fn show_hide_vacancy(&mut self, window: &mut Window, number: usize) {
        self.scene.show_vacancy[number-1] = !self.scene.show_vacancy[number-1];
        if self.scene.show.vacancies {
            self.scene.vacancies[number-1].set_visible(self.scene.show_vacancy[number-1]);
        }
        else {
            self.scene.update_vacancies(window, &self.flake, true);
            self.scene.vacancies[number-1].set_visible(true);
            self.scene.show.vacancies = true;
        }
    }

    fn show_hide_all_vacancies(&mut self, window: &mut Window) {
        if self.scene.show.vacancies {
            self.scene.show_vacancy.iter_mut().for_each(|el| *el = false);
            self.scene.vacancies.iter_mut().for_each(|my_scene| my_scene.set_visible(false) );
        } 
        else { 
            self.scene.show_vacancy.iter_mut().for_each(|el| *el = true);
            self.scene.update_vacancies(window, &self.flake, false);
        }
        self.scene.show.vacancies = !self.scene.show.vacancies;
    }

    fn add_atom(&mut self, window: &mut Window, ijk: IJK) {
        self.i = ijk.i;
        self.j = ijk.j;
        self.k = ijk.k;
        if self.flake.add_atom(ijk) { 
            self.scene.update_surface(window, &self.flake);
            self.scene.update_vacancies(window, &self.flake, false);
        } 
        self.scene.update_boundaries(window, &self.flake);
    }

    pub fn add_random_atoms(&mut self, window: &mut Window, show_process: bool, number: usize) {
        let start = Instant::now();
        let mut ijk = IJK{i: self.i, j: self.j, k:self.k};
        if show_process {
            for _index in 0..number {
                ijk = self.flake.random_vacancy();
                self.flake.add_atom(ijk);
            }                   
            self.scene.update_surface(window, &self.flake);
            self.scene.update_vacancies(window, &self.flake, false);
        } 
        else {
            println!("Calculation {} atoms... ", number.separated_string());
            self.flake.random_add(number);
            println!(" ...finished");
        }
        self.overlay.added_atoms = number;
        self.overlay.duration = start.elapsed();
        self.scene.update_boundaries(window, &self.flake);
        self.i = ijk.i;              // with "let IJK{i,j,k} = ijk" the scope just here
        self.j = ijk.j;  
        self.k = ijk.k;
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn statistics(&mut self, window: &mut Window) {
        let start = Instant::now();
        let added_atoms = self.flake.statistics();
        self.overlay.added_atoms = added_atoms;
        self.overlay.duration = start.elapsed();
        // switch representation to hexagon (to not end in a freeze)
        window.remove_node(&mut self.scene.surface);
        self.scene.surface.set_visible(false);
        self.scene.update_boundaries(window, &self.flake);
        self.scene.show.hexagon = true;
        self.scene.hexagon.set_visible(self.scene.show.hexagon);
        // place camera somewhere sensible
        self.camera = ArcBall::new(Point3::new(-200.0, 200.0, 0.0), Point3::origin());
    }
}

