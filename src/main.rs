use std::time::Instant;
use separator::Separatable;

use kiss3d::event::{Action, Key, WindowEvent};
use kiss3d::light::Light;
use kiss3d::window::Window;
use kiss3d::camera::ArcBall;

use nalgebra::{Point3};

mod helpers;        use helpers::*;
mod parameters;     use parameters::*;
mod lattice;        use lattice::*;
mod storage;        use storage::*;
mod crystal;        use crystal::*; 
mod scene;          use scene::*;
mod planar_scene;   use planar_scene::*;

fn main() {
    // init lattice and flake  
    let mut lattice = Lattice::new(STACKING_FAULTS.to_vec(), DIAMETER);
    let mut flake = Crystal::new(lattice.clone());
    println!("Stacking faults {:?}", STACKING_FAULTS);

    // init OpenGL scene
    let mut window = Window::new_with_size("Flake Growth", 1600, 900);
    let mut overlay = PlanarScene::new(&mut window);
    let mut scene = Scene::new(&mut window, lattice.clone());
    let mut camera = ArcBall::new(Point3::new(-15.0, 7.5, 0.0), Point3::origin());
    window.set_light(Light::Absolute(Point3::new(-300.0, 300.0, 300.0)));
    window.set_background_color(1.0, 1.0, 1.0);
  
    // start with a single atom in the middle
    let IJK { mut i, mut j, mut k } = CENTER;
    let mut new_atom = true;                               

    // helpers for simplifying the event handling of the keys
    let mut add_random_atoms: usize = 0;
    let mut show_hide_vacancy: usize = 0;
    let mut show_process = true;

    // main rendering and event loop
    while window.render_with_camera(&mut camera) {
        if scene.show.hexagon {
            scene.draw_hexgon_outline(&mut window);             // lines have to be redrawn every time   
        }
        if overlay.show {
            overlay.draw_scene(&mut window, &flake);            // text has to be redrawn every time, too
            overlay.update_indicators(&mut window, &flake);     // update length of indicators
        }
        if overlay.show_help {
            overlay.show_help(&mut window);
        }
        for mut event in window.events().iter() {
            match event.value {
                WindowEvent::Key(key, Action::Press, _) => {     
                    match key { 
                        // special actions
                        Key::Back => {     // back to start
                            flake.clear();
                            i = CENTER.i;           // with "let IJK{i,j,k} = ijk" the scope just here
                            j = CENTER.j;  
                            k = CENTER.k;
                            if flake.add_atom(IJK{i,j,k}) { 
                                scene.update_surface(&mut window, &flake);
                                scene.update_dirt(&mut window, &flake);
                                scene.update_vacancies(&mut window, &flake, false);
                            }
                        }                      
                        
                        // interact with (planar) scene
                        Key::Space => {     // show/hide help
                            overlay.show_help = !overlay.show_help;
                            overlay.help.set_visible(overlay.show_help);
                            event.inhibited = true // override the default keyboard handler
                        }
                        Key::R => {         // show/hide indicators
                            overlay.show = ! overlay.show;
                            overlay.scene.set_visible(overlay.show);
                            overlay.layers.set_visible(overlay.show);
                            event.inhibited = true // override the default keyboard handler
                        }
                        Key::S => {         // highlight current atom
                            if scene.show.current {
                                window.remove_node(&mut scene.current);
                            }
                            else
                            {
                                scene.current = window.add_group();
                                add_atom_to_group(&mut scene.current, &lattice.position(IJK{i,j,k}), Color(1.0, 0.0, 0.0));

                            }
                            scene.show.current = !scene.show.current;
                            event.inhibited = true // override the default keyboard handler
                        }
                        Key::G => {         // show/hide gold atoms
                            if scene.show.surface {
                                // window.remove(&mut scene.surface);
                                window.remove_node(&mut scene.surface);
                                scene.surface.set_visible(false);
                            }
                            else {   
                                scene.surface = window.add_group();
                                if scene.visual_layers {
                                    flake.surface.list.iter().for_each(|&ijk| add_atom_to_group(
                                        &mut scene.surface, &lattice.position(ijk), ATOM_COLORS[(lattice.stacking.pos[ijk.k as usize].rem_euclid(3)) as usize])
                                    );   
                                }
                                else {
                                    flake.surface.list.iter().for_each(|&ijk| add_atom_to_group(&mut scene.surface, &lattice.position(ijk), GOLD) );

                                }
                                scene.surface.set_visible(true);
                            }
                            scene.show.surface = !scene.show.surface;
                            event.inhibited = true // override the default keyboard handler
                        }  
                        Key::T => {         // show/hide dirt atoms
                            if scene.show.dirt {
                                window.remove_node(&mut scene.dirt);
                                scene.dirt.set_visible(false);
                            }
                            else {   
                                scene.dirt = window.add_group();
                                flake.dirt.list.iter().for_each(|&ijk| add_atom_to_group(&mut scene.dirt, &lattice.position(ijk), DIRT) );
                                scene.dirt.set_visible(true);
                            }
                            scene.show.dirt = !scene.show.dirt;
                            event.inhibited = true // override the default keyboard handler
                        }  
                        Key::V => {         // visualize layers on/off
                            scene.visual_layers = !scene.visual_layers;
                            scene.update_surface(&mut window, &flake);
                            event.inhibited = true // override the default keyboard handler
                        }  
                        Key::B => {         // show/hide wireframe (box)
                            scene.show.wireframe = !scene.show.wireframe;
                            scene.wireframe.set_visible(scene.show.wireframe);
                            event.inhibited = true // override the default keyboard handler
                        }  
                        Key::H => {         // show/hide hexagon
                            scene.show.hexagon = !scene.show.hexagon;
                            scene.hexagon.set_visible(scene.show.hexagon);
                            event.inhibited = true // override the default keyboard handler
                        }  
                        
                        // add some special geometries
                        Key::L => {         // add layer
                            flake.add_layer(IJK{i,j,k}, 7, State::Gold);
                            scene.update_surface(&mut window, &flake);
                            scene.update_vacancies(&mut window, &flake, false);
                            event.inhibited = true // override the default keyboard handler
                        } 
                        Key::O => {         // add dirt layer on top
                            flake.add_layer(IJK{i,j,k: flake.extrema_ijk.z_max.k+1}, 7, State::Dirt);
                            scene.update_dirt(&mut window, &flake);
                            scene.update_vacancies(&mut window, &flake, false);
                            event.inhibited = true // override the default keyboard handler
                        } 
                        Key::Home => {      // add sphere
                            flake.clear();
                            // sphere
                            let pos = XYZ{x: 0.0, y: 0.0, z: 0.0};
                            flake.add_sphere(pos, 5.0);
                            camera = ArcBall::new(Point3::new(-45.0, 22.5, 0.0), Point3::origin());
                            window.render_with_camera(&mut camera);
                            scene.update_surface(&mut window, &flake);
                            scene.update_vacancies(&mut window, &flake, false);
                            event.inhibited = true // override the default keyboard handler  
                        }     
                        Key::End => {       // add cylinder
                            flake.clear();
                            let pos = XYZ{x: 0.0, y: 0.0, z: 0.0};
                            flake.add_cylinder(pos, 20.0, 5.0);
                            camera = ArcBall::new(Point3::new(-45.0, 22.5, 0.0), Point3::origin());
                            window.render_with_camera(&mut camera);
                            scene.update_surface(&mut window, &flake);
                            scene.update_vacancies(&mut window, &flake, false);
                            event.inhibited = true // override the default keyboard handler
                        }                          
                        Key::Delete => {    // add rounded monomer antenna
                            flake.clear();
                            // antenna arm
                            let pos = XYZ{x: 0.0, y: 0.0, z: -3.0};
                            flake.add_rounded_box(pos, 20.0, 10.0, 6.0, 3.0);
                            // other stuff
                            camera = ArcBall::new(Point3::new(-45.0, 22.5, 0.0), Point3::origin());
                            window.render_with_camera(&mut camera);
                            scene.update_surface(&mut window, &flake);
                            scene.update_vacancies(&mut window, &flake, false);
                            event.inhibited = true // override the default keyboard handler
                        }
                        Key::I => {         // add rounded Dipole antenna
                            flake.clear();
                            // left arm
                            let pos = XYZ{x: -15.0, y: 0.0, z: -3.0};
                            flake.add_rounded_box(pos, 20.0, 10.0, 6.0, 3.0);
                            // right arm
                             let pos = XYZ{x: 15.0, y: 0.0, z: -3.0};
                            flake.add_rounded_box(pos, 20.0, 10.0, 7.0, 3.5);
                            // other stuff
                            camera = ArcBall::new(Point3::new(-45.0, 22.5, 0.0), Point3::origin());
                            window.render_with_camera(&mut camera);
                            scene.update_surface(&mut window, &flake);
                            scene.update_vacancies(&mut window, &flake, false);
                            event.inhibited = true // override the default keyboard handler
                        }                        
                        Key::K => {         // add Dipole antenna
                            flake.clear();
                            // left arm
                            let pos = XYZ{x: -15.0, y: 0.0, z: 0.0};
                            flake.add_box(pos, 20.0, 10.0, 6.0, State::Gold);
                            // left arm scond level
                            let pos = XYZ{x: -15.0, y: 0.0, z: 3.5};
                            flake.add_box(pos, 18.0, 8.0, 1.0, State::Gold);
                            // right arm
                            let pos = XYZ{x: 15.0, y: 0.0, z: 0.0};
                            flake.add_box(pos, 20.0, 10.0, 6.0, State::Gold);
                            // other stuff
                            camera = ArcBall::new(Point3::new(-45.0, 22.5, 0.0), Point3::origin());
                            window.render_with_camera(&mut camera);
                            scene.update_surface(&mut window, &flake);
                            scene.update_vacancies(&mut window, &flake, false);
                            event.inhibited = true // override the default keyboard handler
                        }                        
                        Key::U => {         // add rounded Jord antenna
                            flake.clear();
                            // left arm
                            let pos = XYZ{x: -14.0, y: 0.0, z: -3.0};
                            flake.add_rounded_box(pos, 20.0, 10.0, 6.0, 3.0);
                            // right arm
                            let pos = XYZ{x: 14.0, y: 0.0, z: -3.0};
                            flake.add_rounded_box(pos, 20.0, 10.0, 6.0, 3.0);
                            // left connector
                            let pos = XYZ{x: -14.0, y: 25.0, z: -3.0};
                            flake.add_rounded_box(pos, 5.0, 42.0, 3.0, 2.0);
                            // right connector
                            let pos = XYZ{x: 14.0, y: -25.0, z: -3.0};
                            flake.add_rounded_box(pos, 5.0, 42.0, 3.0, 2.0);
                            // left protector
                            let pos = XYZ{x: -14.0, y: 25.0, z: -1.5};
                            flake.add_box(pos, 6.0, 44.0, 4.0, State::Dirt);
                            // right protector
                            let pos = XYZ{x: 14.0, y: -25.0, z: -1.5};
                            flake.add_box(pos, 6.0, 44.0, 4.0, State::Dirt);
                            // waist for the left arm
                            let pos = XYZ{x: -14.0, y: 0.0, z: 0.0};
                            flake.add_box(pos, 10.0, 12.0, 7.0, State::Dirt);
                            // other stuff
                            camera = ArcBall::new(Point3::new(-60.0, 60.0, 0.0), Point3::origin());
                            window.render_with_camera(&mut camera);
                            scene.update_surface(&mut window, &flake);
                            scene.update_dirt(&mut window, &flake);
                            scene.update_vacancies(&mut window, &flake, false);
                            event.inhibited = true // override the default keyboard handler
                        }
                        Key::J => {         // add Jord antenna
                            flake.clear();
                            // left arm
                            let pos = XYZ{x: -15.0, y: 0.0, z: 0.0};
                            flake.add_box(pos, 20.0, 10.0, 6.0, State::Gold);
                            // right arm
                            let pos = XYZ{x: 15.0, y: 0.0, z: 0.0};
                            flake.add_box(pos, 20.0, 10.0, 6.0, State::Gold);
                            // left connector;
                            let pos = XYZ{x: -15.0, y: 25.0, z: -1.5};
                            flake.add_box(pos, 5.0, 40.0, 3.0, State::Gold);
                            // right connector;
                            let pos = XYZ{x: 15.0, y: -25.0, z: -1.5};
                            flake.add_box(pos, 5.0, 40.0, 3.0, State::Gold);
                            // other stuff
                            camera = ArcBall::new(Point3::new(-60.0, 60.0, 0.0), Point3::origin());
                            window.render_with_camera(&mut camera);
                            scene.update_surface(&mut window, &flake);
                            scene.update_vacancies(&mut window, &flake, false);
                            event.inhibited = true // override the default keyboard handler
                        }
                        Key::Down => {      // add substrat below lowest vacancies layer
                            if flake.substrate_pos == 1 {
                                flake.substrate_pos = flake.extrema_ijk.z_min.k - 1;
                                flake.update_vacancies();
                                scene.update_vacancies(&mut window, &flake, false);
                                scene.add_substrate(&mut flake);
                                scene.show.substrate = true;
                                scene.substrate.set_visible(scene.show.substrate);
                                println!("Substrate added at: {:?}", flake.substrate_pos);
                            }
                            else  {
                                flake.substrate_pos = 1;
                                flake.update_vacancies();
                                scene.show.substrate = false;
                                scene.substrate.set_visible(scene.show.substrate);
                            }
                            event.inhibited = true // override the default keyboard handler
                        }    
                        Key::Minus => {
                            scene.show.substrate = !scene.show.substrate;
                            scene.substrate.set_visible(scene.show.substrate);
                        }                   
                        
                        // tweak stacking
                        Key::Up => {        // reset stacking
                            lattice = Lattice::new(STACKING_FAULTS.to_vec(), DIAMETER);
                            flake.lattice = lattice.clone();
                            flake.update_vacancies();
                            scene.lattice = lattice.clone();
                            scene.update_vacancies(&mut window, &flake, false);
                            println!("Stacking faults {:?}", lattice.stacking_faults);
                            event.inhibited = true // override the default keyboard handler
                        }
                        Key::PageUp => {    // add stacking faults on top
                            let mut stacking = flake.lattice.stacking_faults.clone();
                            let new_fault = flake.extrema_ijk.z_max.k + 1;
                            match stacking.binary_search(&(new_fault)) {
                                Ok(_pos) => {} // element already in vector @ `pos` 
                                Err(pos) => {
                                    stacking.insert(pos, new_fault);
                                    lattice = Lattice::new(stacking, DIAMETER);
                                    flake.lattice = lattice.clone();
                                    flake.update_vacancies();
                                    scene.lattice = lattice.clone();
                                    scene.update_vacancies(&mut window, &flake, false);
                                },
                            }
                            println!("Stacking faults {:?}", lattice.stacking_faults);
                            event.inhibited = true // override the default keyboard handler
                        } 
                        Key::PageDown => {  // add stacking faults on bottom
                            let mut stacking = flake.lattice.stacking_faults.clone();
                            let new_fault = flake.extrema_ijk.z_min.k + 1;
                            match stacking.binary_search(&(new_fault)) {
                                Ok(_pos) => {} // element already in vector @ `pos` 
                                Err(pos) => {
                                    stacking.insert(pos, new_fault);
                                    // insert/remove a stacking fault down at the bottom to not mess up with the representation
                                    match stacking.binary_search(&(0)) {
                                        Ok(_pos) => {stacking.remove(0);} 
                                        Err(pos) => {stacking.insert(pos, 0)}
                                    }
                                    lattice = Lattice::new(stacking, DIAMETER);
                                    flake.lattice = lattice.clone();
                                    flake.update_vacancies();
                                    scene.lattice = lattice.clone();
                                    scene.update_vacancies(&mut window, &flake, false);
                                },
                            }
                            println!("Stacking faults {:?}", lattice.stacking_faults);
                            event.inhibited = true // override the default keyboard handler
                        } 

                        // change probabilities
                        Key::P => {  
                             flake.next_prob_list();
                             event.inhibited = true // override the default keyboard handler
                            }                         

                        // show/hide vacancies
                        Key::F1 => {   show_hide_vacancy = 1; }                         
                        Key::F2 => {   show_hide_vacancy = 2; }                          
                        Key::F3 => {   show_hide_vacancy = 3; }                                            
                        Key::F4 => {   show_hide_vacancy = 4; }                                            
                        Key::F5 => {   show_hide_vacancy = 5; }                                               
                        Key::F6 => {   show_hide_vacancy = 6; }                                            
                        Key::F7 => {   show_hide_vacancy = 7; }                                             
                        Key::F8 => {   show_hide_vacancy = 8; } 
                        Key::F9 => {   show_hide_vacancy = 9; }            
                        Key::F  => {   show_hide_vacancy = 99;}   // show-hide all

                        // initiating atoms adding at random positions
                        Key::Key1 => { add_random_atoms = 1;      show_process = true;}   
                        Key::Key2 => { add_random_atoms = 10;     show_process = true;}   
                        Key::Key3 => { add_random_atoms = 100;    show_process = true;}   
                        Key::Key4 => { add_random_atoms = 1_000;  show_process = true;}   
                        Key::Key5 => { add_random_atoms = 10_000; show_process = true;}   
                        Key::Key6 => { add_random_atoms = 100_000;     }  
                        Key::Key7 => { add_random_atoms = 1_000_000;   }   
                        Key::Key8 => { add_random_atoms = 10_000_000;  }   
                        Key::Key9 => { add_random_atoms = 100_000_000; }  

                        // go to position and request a new atom there
                        Key::X => { new_atom = true; k -=1; }
                        Key::W => { new_atom = true; k +=1; }                
                        Key::D => { new_atom = true; i +=1; }                       
                        Key::A => { new_atom = true; i -=1; }                
                        Key::E => { new_atom = true; j +=1; }                
                        Key::Y => { new_atom = true; j -=1; }   
                        Key::C => { new_atom = true; i +=1; j -=1; }              
                        Key::Q => { new_atom = true; i -=1; j +=1; }   
                        
                        // some statistics
                        Key::Comma => {
                            // start timer and call statistics module
                            let start = Instant::now();
                            let added_atoms = flake.statistics();
                            overlay.added_atoms = added_atoms;
                            overlay.duration = start.elapsed();
                            // switch representation to hexagon (to not end in a freeze)
                            window.remove_node(&mut scene.surface);
                            scene.surface.set_visible(false);
                            scene.update_boundaries(&mut window, &flake);
                            scene.show.hexagon = true;
                            scene.hexagon.set_visible(scene.show.hexagon);
                            // place camera somewhere sensible
                            camera = ArcBall::new(Point3::new(-200.0, 200.0, 0.0), Point3::origin());
                            window.render_with_camera(&mut camera);
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
            if new_atom {
                new_atom = false;
                if flake.add_atom(IJK{i,j,k}) { 
                    scene.update_surface(&mut window, &flake);
                    scene.update_vacancies(&mut window, &flake, false);
                } 
                scene.update_boundaries(&mut window, &flake);
                event.inhibited = true; // override the default keyboard handler
            }
            if add_random_atoms > 0 {
                let start = Instant::now();
                let mut ijk = IJK{i,j,k};
                if show_process {
                    for _index in 0..add_random_atoms {
                        ijk = flake.random_vacancy();
                        flake.add_atom(ijk);
                    }                   
                    scene.update_surface(&mut window, &flake);
                    scene.update_vacancies(&mut window, &flake, false);
                } 
                else {
                    println!("Calculation {} atoms... ", add_random_atoms.separated_string());
                    flake.random_add(add_random_atoms);
                    println!(" ...finished");
                }
                overlay.added_atoms = add_random_atoms;
                overlay.duration = start.elapsed();
                scene.update_boundaries(&mut window, &flake);
                i = ijk.i;              // with "let IJK{i,j,k} = ijk" the scope just here
                j = ijk.j;  
                k = ijk.k;
                add_random_atoms = 0;
                show_process = false;
                event.inhibited = true; // override the default keyboard handler
            }
            if show_hide_vacancy > 0 {
                if show_hide_vacancy == 99 {
                    if scene.show.vacancies {
                        scene.show_vacancy.iter_mut().for_each(|el| *el = false);
                        scene.vacancies.iter_mut().for_each(|my_scene| my_scene.set_visible(false) );
                    } 
                    else { 
                        scene.show_vacancy.iter_mut().for_each(|el| *el = true);
                        scene.update_vacancies(&mut window, &flake, false);
                    }
                    scene.show.vacancies = !scene.show.vacancies;
                }
                else {
                    scene.show_vacancy[show_hide_vacancy-1] = !scene.show_vacancy[show_hide_vacancy-1];
                    if scene.show.vacancies {
                        scene.vacancies[show_hide_vacancy-1].set_visible(scene.show_vacancy[show_hide_vacancy-1]);
                    }
                    else {
                        scene.update_vacancies(&mut window, &flake, true);
                        scene.vacancies[show_hide_vacancy-1].set_visible(true);
                        scene.show.vacancies = true;
                    }
                }
                show_hide_vacancy = 0;
                event.inhibited = true // override the default keyboard handler
            }
            if scene.show.current {
                window.remove_node(&mut scene.current);
                scene.current = window.add_group();
                add_atom_to_group(&mut scene.current, &lattice.position(IJK{i,j,k}), Color(1.0, 0.0, 0.0));
            }
        }
    }
}
