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

use kiss3d::light::Light;
use kiss3d::camera::{Camera, ArcBall};
use kiss3d::planar_camera::PlanarCamera;
use kiss3d::post_processing::PostProcessingEffect;
use kiss3d::event::{Action, Key, WindowEvent};
use kiss3d::window::{State, Window};
// use nalgebra::{Vector3,Point3};
use nalgebra::Point3;

#[cfg(target_arch = "wasm32")]
use crate::println;

pub struct AppState {
    overlay: PlanarScene,
    scene: Scene,
    camera: ArcBall,
    lattice: Lattice,
    flake: Crystal,
    i: u16,
    j: u16,
    k: u16,
    new_atom: bool,
    add_random_atoms: usize,
    show_hide_vacancy: usize,
    show_process: bool
}

impl AppState {
    pub fn init(window: &mut Window) -> Self {
        // init lattice and flake  
        let lattice = Lattice::new(STACKING_FAULTS.to_vec(), DIAMETER);
        let flake = Crystal::new(lattice.clone());

        println!("Stacking faults {:?}", STACKING_FAULTS);

        // init OpenGL scene
        // let mut window = Window::new_with_size("Flake Growth", 1600, 900);
        let scene = Scene::new(window, lattice.clone());
        let overlay = PlanarScene::new(window);
        let camera = ArcBall::new(Point3::new(-15.0, 7.5, 0.0), Point3::origin());
        window.set_light(Light::Absolute(Point3::new(-300.0, 300.0, 300.0)));
        window.set_background_color(1.0, 1.0, 1.0);
    
        // start with a single atom in the middle
        let IJK { i, j, k } = CENTER;
        let new_atom = true;                               

        // helpers for simplifying the event handling of the keys
        let add_random_atoms: usize = 100;
        let show_hide_vacancy: usize = 0;
        let show_process = true;
        AppState{
            overlay,
            scene,
            camera,
            lattice,
            flake,
            i,
            j,
            k,
            new_atom,
            add_random_atoms,
            show_hide_vacancy,
            show_process,
        }
    }
}

impl State for AppState {

    fn cameras_and_effect( &mut self,) -> (
        Option<&mut dyn Camera>,
        Option<&mut dyn PlanarCamera>,
        Option<&mut dyn PostProcessingEffect>,
    ) {
        (Some(&mut self.camera), None, None)
    }


    fn step(&mut self, window: &mut Window) {
        if self.scene.show.hexagon {
            self.scene.draw_hexgon_outline(window);             // lines have to be redrawn every time   
        }
        if self.overlay.show {
            self.overlay.draw_scene(window, &self.flake);            // text has to be redrawn every time, too
            self.overlay.update_indicators(window, &self.flake);     // update length of indicators
        }
        if self.overlay.show_help {
            self.overlay.show_help(window);
        }
        for mut event in window.events().iter() {
            match event.value {
                WindowEvent::Key(key, Action::Press, _) => {     
                    match key { 
                        // special actions
                        Key::Back => {     // back to start
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
                        
                        // interact with (planar) scene
                        Key::Space => {     // show/hide help
                            self.overlay.show_help = !self.overlay.show_help;
                            self.overlay.help.set_visible(self.overlay.show_help);
                            event.inhibited = true // override the default keyboard handler
                        }
                        Key::R => {         // show/hide indicators
                            self.overlay.show = !self.overlay.show;
                            self.overlay.scene.set_visible(self.overlay.show);
                            self.overlay.layers.set_visible(self.overlay.show);
                            event.inhibited = true // override the default keyboard handler
                        }
                        Key::S => {         // highlight current atom
                            if self.scene.show.current {
                                window.remove_node(&mut self.scene.current);
                            }
                            else
                            {
                                self.scene.current = window.add_group();
                                add_atom_to_group(&mut self.scene.current, &self.lattice.position(IJK{i: self.i, j: self.j, k:self.k}), Color(1.0, 0.0, 0.0));

                            }
                            self.scene.show.current = !self.scene.show.current;
                            event.inhibited = true // override the default keyboard handler
                        }
                        Key::G => {         // show/hide gold atoms
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
                            event.inhibited = true // override the default keyboard handler
                        }  
                        Key::T => {         // show/hide dirt atoms
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
                            event.inhibited = true // override the default keyboard handler
                        }  
                        Key::V => {         // visualize layers on/off
                            self.scene.visual_layers = !self.scene.visual_layers;
                            self.scene.update_surface(window, &self.flake);
                            event.inhibited = true // override the default keyboard handler
                        }  
                        Key::B => {         // show/hide wireframe (box)
                            self.scene.show.wireframe = !self.scene.show.wireframe;
                            self.scene.wireframe.set_visible(self.scene.show.wireframe);
                            event.inhibited = true // override the default keyboard handler
                        }  
                        Key::H => {         // show/hide hexagon
                            self.scene.show.hexagon = !self.scene.show.hexagon;
                            self.scene.hexagon.set_visible(self.scene.show.hexagon);
                            event.inhibited = true // override the default keyboard handler
                        }  
                        
                        // add some special geometries
                        Key::L => {         // add layer
                            self.flake.add_layer(IJK{i: self.i, j: self.j, k:self.k}, 7, Atom::Gold);
                            self.scene.update_surface(window, &self.flake);
                            self.scene.update_vacancies(window, &self.flake, false);
                            event.inhibited = true // override the default keyboard handler
                        } 
                        Key::O => {         // add dirt layer on top
                            self.flake.add_layer(IJK{i: self.i, j: self.j, k: self.flake.extrema_ijk.z_max.k+1}, 7, Atom::Dirt);
                            self.scene.update_dirt(window, &self.flake);
                            self.scene.update_vacancies(window, &self.flake, false);
                            event.inhibited = true // override the default keyboard handler
                        } 
                        Key::Home => {      // add sphere
                            self.flake.clear();
                            // sphere
                            let pos = XYZ{x: 0.0, y: 0.0, z: 0.0};
                            self.flake.add_sphere(pos, 5.0);
                            self.camera = ArcBall::new(Point3::new(-45.0, 22.5, 0.0), Point3::origin());
                            // window.render_with_camera(&mut self.camera);
                            self.scene.update_surface(window, &self.flake);
                            self.scene.update_vacancies(window, &self.flake, false);
                            event.inhibited = true // override the default keyboard handler  
                        }     
                        Key::End => {       // add cylinder
                            self.flake.clear();
                            let pos = XYZ{x: 0.0, y: 0.0, z: 0.0};
                            self.flake.add_cylinder(pos, 20.0, 5.0);
                            self.camera = ArcBall::new(Point3::new(-45.0, 22.5, 0.0), Point3::origin());
                            // window.render_with_camera(&mut self.camera);
                            self.scene.update_surface(window, &self.flake);
                            self.scene.update_vacancies(window, &self.flake, false);
                            event.inhibited = true // override the default keyboard handler
                        }                          
                        Key::Delete => {    // add rounded monomer antenna
                            self.flake.clear();
                            // antenna arm
                            let pos = XYZ{x: 0.0, y: 0.0, z: -3.0};
                            self.flake.add_rounded_box(pos, 20.0, 10.0, 6.0, 3.0);
                            // other stuff
                            self.camera = ArcBall::new(Point3::new(-45.0, 22.5, 0.0), Point3::origin());
                            // window.render_with_camera(&mut self.camera);
                            self.scene.update_surface(window, &self.flake);
                            self.scene.update_vacancies(window, &self.flake, false);
                            event.inhibited = true // override the default keyboard handler
                        }
                        Key::I => {         // add rounded Dipole antenna
                            self.flake.clear();
                            // left arm
                            let pos = XYZ{x: -15.0, y: 0.0, z: -3.0};
                            self.flake.add_rounded_box(pos, 20.0, 10.0, 6.0, 3.0);
                            // right arm
                             let pos = XYZ{x: 15.0, y: 0.0, z: -3.0};
                            self.flake.add_rounded_box(pos, 20.0, 10.0, 7.0, 3.5);
                            // other stuff
                            self.camera = ArcBall::new(Point3::new(-45.0, 22.5, 0.0), Point3::origin());
                            // window.render_with_camera(&mut self.camera);
                            self.scene.update_surface(window, &self.flake);
                            self.scene.update_vacancies(window, &self.flake, false);
                            event.inhibited = true // override the default keyboard handler
                        }                        
                        Key::K => {         // add Dipole antenna
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
                            // window.render_with_camera(&mut self.camera);
                            self.scene.update_surface(window, &self.flake);
                            self.scene.update_vacancies(window, &self.flake, false);
                            event.inhibited = true // override the default keyboard handler
                        }                        
                        Key::U => {         // add rounded Jord antenna
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
                            // window.render_with_camera(&mut self.camera);
                            self.scene.update_surface(window, &self.flake);
                            self.scene.update_dirt(window, &self.flake);
                            self.scene.update_vacancies(window, &self.flake, false);
                            event.inhibited = true // override the default keyboard handler
                        }
                        Key::J => {         // add Jord antenna
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
                            // window.render_with_camera(&mut self.camera);
                            self.scene.update_surface(window, &self.flake);
                            self.scene.update_vacancies(window, &self.flake, false);
                            event.inhibited = true // override the default keyboard handler
                        }
                        Key::M => {      // add substrat below lowest vacancies layer
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
                            event.inhibited = true // override the default keyboard handler
                        }    
                        Key::Minus => {
                            self.scene.show.substrate = !self.scene.show.substrate;
                            self.scene.substrate.set_visible(self.scene.show.substrate);
                        }                   
                        
                        // tweak stacking
                        Key::Up => {        // reset stacking
                            self.lattice = Lattice::new(STACKING_FAULTS.to_vec(), DIAMETER);
                            self.flake.lattice = self.lattice.clone();
                            self.flake.update_vacancies();
                            self.scene.lattice = self.lattice.clone();
                            self.scene.update_vacancies(window, &self.flake, false);
                            println!("Stacking faults {:?}", self.lattice.stacking_faults);
                            event.inhibited = true // override the default keyboard handler
                        }
                        Key::PageUp => {    // add stacking faults on top
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
                            event.inhibited = true // override the default keyboard handler
                        } 
                        Key::PageDown => {  // add stacking faults on bottom
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
                            event.inhibited = true // override the default keyboard handler
                        } 

                        // change probabilities
                        Key::P => {  
                             self.flake.next_prob_list();
                             event.inhibited = true // override the default keyboard handler
                            }                         

                        // show/hide vacancies
                        Key::F1 => {   self.show_hide_vacancy = 1; }                         
                        Key::F2 => {   self.show_hide_vacancy = 2; }                          
                        Key::F3 => {   self.show_hide_vacancy = 3; }                                            
                        Key::F4 => {   self.show_hide_vacancy = 4; }                                            
                        Key::F5 => {   self.show_hide_vacancy = 5; }                                               
                        Key::F6 => {   self.show_hide_vacancy = 6; }                                            
                        Key::F7 => {   self.show_hide_vacancy = 7; }                                             
                        Key::F8 => {   self.show_hide_vacancy = 8; } 
                        Key::F9 => {   self.show_hide_vacancy = 9; }            
                        Key::F  => {   self.show_hide_vacancy = 99;}   // show-hide all

                        // initiating atoms adding at random positions
                        Key::Key1 => { self.add_random_atoms = 1;      self.show_process = true;}   
                        Key::Key2 => { self.add_random_atoms = 10;     self.show_process = true;}   
                        Key::Key3 => { self.add_random_atoms = 100;    self.show_process = true;}   
                        Key::Key4 => { self.add_random_atoms = 1_000;  self.show_process = true;}   
                        Key::Key5 => { self.add_random_atoms = 10_000; self.show_process = true;}   
                        Key::Key6 => { self.add_random_atoms = 100_000;     }  
                        Key::Key7 => { self.add_random_atoms = 1_000_000;   }   
                        Key::Key8 => { self.add_random_atoms = 10_000_000;  }   
                        Key::Key9 => { self.add_random_atoms = 100_000_000; }  

                        // go to position and request a new atom there
                        Key::X => { self.new_atom = true; self.k -=1; }
                        Key::W => { self.new_atom = true; self.k +=1; }                
                        Key::D => { self.new_atom = true; self.i +=1; }                       
                        Key::A => { self.new_atom = true; self.i -=1; }                
                        Key::E => { self.new_atom = true; self.j +=1; }                
                        Key::Y => { self.new_atom = true; self.j -=1; }   
                        Key::C => { self.new_atom = true; self.i +=1; self.j -=1; }              
                        Key::Q => { self.new_atom = true; self.i -=1; self.j +=1; }   
                        
                        // some statistics
                        Key::Comma => {
                            // start timer and call statistics module
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
                            // window.render_with_camera(&mut self.camera);
                            event.inhibited = true; // override the default keyboard handler
                           }

                        // other keys
                        _=> {}
                    }
                }
            // other events
            _=> {}
            }

            // pseudo functions for common changes (to clear up the code)
            if self.new_atom {
                self.new_atom = false;
                if self.flake.add_atom(IJK{i: self.i, j: self.j, k:self.k}) { 
                    self.scene.update_surface(window, &self.flake);
                    self.scene.update_vacancies(window, &self.flake, false);
                } 
                self.scene.update_boundaries(window, &self.flake);
                event.inhibited = true; // override the default keyboard handler
            }
            if self.add_random_atoms > 0 {
                let start = Instant::now();
                let mut ijk = IJK{i: self.i, j: self.j, k:self.k};
                if self.show_process {
                    for _index in 0..self.add_random_atoms {
                        ijk = self.flake.random_vacancy();
                        self.flake.add_atom(ijk);
                    }                   
                    self.scene.update_surface(window, &self.flake);
                    self.scene.update_vacancies(window, &self.flake, false);
                } 
                else {
                    println!("Calculation {} atoms... ", self.add_random_atoms.separated_string());
                    self.flake.random_add(self.add_random_atoms);
                    println!(" ...finished");
                }
                self.overlay.added_atoms = self.add_random_atoms;
                self.overlay.duration = start.elapsed();
                self.scene.update_boundaries(window, &self.flake);
                self.i = ijk.i;              // with "let IJK{i,j,k} = ijk" the scope just here
                self.j = ijk.j;  
                self.k = ijk.k;
                self.add_random_atoms = 0;
                self.show_process = false;
                event.inhibited = true; // override the default keyboard handler
            }
            if self.show_hide_vacancy > 0 {
                if self.show_hide_vacancy == 99 {
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
                else {
                    self.scene.show_vacancy[self.show_hide_vacancy-1] = !self.scene.show_vacancy[self.show_hide_vacancy-1];
                    if self.scene.show.vacancies {
                        self.scene.vacancies[self.show_hide_vacancy-1].set_visible(self.scene.show_vacancy[self.show_hide_vacancy-1]);
                    }
                    else {
                        self.scene.update_vacancies(window, &self.flake, true);
                        self.scene.vacancies[self.show_hide_vacancy-1].set_visible(true);
                        self.scene.show.vacancies = true;
                    }
                }
                self.show_hide_vacancy = 0;
                event.inhibited = true // override the default keyboard handler
            }
            if self.scene.show.current {
                window.remove_node(&mut self.scene.current);
                self.scene.current = window.add_group();
                add_atom_to_group(&mut self.scene.current, &self.lattice.position(IJK{i: self.i, j: self.j, k:self.k}), Color(1.0, 0.0, 0.0));
            }
        }
    }

}
