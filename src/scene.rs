use std::cell::RefCell;
use std::rc::Rc;

use kiss3d::window::Window;
use kiss3d::resource::Mesh;
use kiss3d::scene::SceneNode;
use nalgebra::{Translation3, Point3, Vector3};
// use nalgebra::{UnitQuaternion, Vector3};

use crate::helpers::*;
use crate::parameters::*;
use crate::lattice::*;
use crate::crystal::*;


pub struct ShowSceneNode {pub surface: bool, pub dirt: bool, pub vacancies: bool, pub current: bool, pub wireframe: bool, pub hexagon: bool, pub substrate: bool }


pub struct Scene {
    pub lattice: Lattice,
    pub surface: SceneNode,
    pub dirt: SceneNode,
    pub vacancies: Vec<SceneNode>,
    pub show_vacancy: Vec<bool>,
    pub current: SceneNode,
    pub wireframe: SceneNode,
    pub hexagon: SceneNode,
    pub hex_upper: Vec<Point3<f32>>,
    pub hex_lower: Vec<Point3<f32>>,
    pub substrate: SceneNode,
    pub show: ShowSceneNode,
    pub visual_layers: bool,
}

impl Scene {
    pub fn new(window: &mut Window,lattice: Lattice) -> Self {
        let mut surface = window.add_group();
        let mut dirt = window.add_group();
        let mut vacancies_all = window.add_group();
        let mut vacancies = Vec::new();
        let mut show_vacancy = Vec::new();
        for _index in 0..VAC_LISTS {
            vacancies.push(vacancies_all.add_group());
            show_vacancy.push(false);
        }
        let show = ShowSceneNode{surface: true, dirt: true, vacancies: false, current: true, wireframe: false, hexagon: false, substrate: false};
        surface.set_visible(show.surface);
        dirt.set_visible(show.dirt);
        Scene{ 
            lattice, 
            surface, 
            dirt, 
            vacancies, 
            show_vacancy, 
            current:        window.add_group(), 
            wireframe:      window.add_group(), 
            hexagon:        window.add_group(), 
            hex_upper:      Vec::new(), 
            hex_lower:      Vec::new(), 
            substrate:      window.add_group(), 
            show, 
            visual_layers:  false 
        }
    }

    pub fn update_surface(&mut self, window: &mut Window, crystal: &Crystal) {
        window.remove_node(&mut self.surface);
        self.surface = window.add_group();
        if self.visual_layers {
            crystal.surface.list.iter().for_each(|&ijk| {
                let layer = (self.lattice.stacking.pos[ijk.k as usize]).rem_euclid(3) as usize;     // a better modulo function
                add_atom_to_group(&mut self.surface, &crystal.lattice.position(ijk), ATOM_COLORS[layer]) 
            });
        }
        else {
            crystal.surface.list.iter().for_each(|&ijk| {
                // let dyn_color =  Color((ijk.i - CENTER.i) as f32 / 10.0, (ijk.j - CENTER.j) as f32 / 10.0, (ijk.k - CENTER.k) as f32 / 10.0);
                // add_atom_to_group(&mut self.surface, &crystal.lattice.position(ijk), dyn_color) 
                add_atom_to_group(&mut self.surface, &crystal.lattice.position(ijk), GOLD) 
            });
        }
        if !self.show.surface {
            &self.surface.set_visible(false);
        }
    }

    pub fn update_dirt(&mut self, window: &mut Window, crystal: &Crystal) {
        window.remove_node(&mut self.dirt);
        self.dirt = window.add_group();
        crystal.dirt.list.iter().for_each(|&ijk| {
            add_atom_to_group(&mut self.dirt, &crystal.lattice.position(ijk), DIRT) 
        });
        if !self.show.dirt {
            &self.dirt.set_visible(false);
        }
    }

    pub fn update_vacancies(&mut self, window: &mut Window, crystal: &Crystal, hide_all: bool) {
        &self.vacancies.iter_mut().for_each(|el|window.remove_node(el));
        &self.vacancies.iter_mut()
                       .enumerate()
                       .for_each(|(number,my_scene)| {
                                *my_scene = window.add_group();
                                crystal.vacancies.list[number].iter().for_each(|&ijk| 
                                    add_atom_to_group(my_scene, &crystal.lattice.position(ijk), VAC_COLORS[number]) 
                                    );
                                });
        for i in 0..VAC_LISTS {
            if hide_all {
                &self.vacancies[i].set_visible(false);
            }
            else {
                &self.vacancies[i].set_visible(self.show_vacancy[i]);

            }
        }
    }

    pub fn add_wireframe(&mut self, flake: &Crystal) {
        // println!("extrema: {:?}", flake.extrema);
        let mut c = self.wireframe.add_cube(flake.extrema.y_max - flake.extrema.y_min + DIAMETER, 
                                            flake.extrema.z_max - flake.extrema.z_min + DIAMETER, 
                                            flake.extrema.x_max - flake.extrema.x_min + DIAMETER);
        let center = &Translation3::new((flake.extrema.y_max + flake.extrema.y_min)/2.0, 
                                        (flake.extrema.z_max + flake.extrema.z_min)/2.0, 
                                        (flake.extrema.x_max + flake.extrema.x_min)/2.0,);
        c.append_translation(center);
        c.set_color(1.0, 0.0, 0.0);
        c.set_points_size(1.0);
        c.set_lines_width(1.0);
        c.set_surface_rendering_activation(false);
    }


    pub fn add_hexagon(&mut self, flake: &Crystal) {
        // println!("extrema: {:?}", flake.extrema);
        let z_max = flake.extrema.z_max + DIAMETER/2.0;
        let z_min = flake.extrema.z_min - DIAMETER/2.0;
        let [ax,ay,bx,by,cx,cy,dx,dy,ex,ey,fx,fy] = flake.get_hexagon();

        self.hex_upper.clear();
        self.hex_upper.push(Point3::new(ay, z_max, ax));
        self.hex_upper.push(Point3::new(by, z_max, bx));
        self.hex_upper.push(Point3::new(cy, z_max, cx));
        self.hex_upper.push(Point3::new(dy, z_max, dx));
        self.hex_upper.push(Point3::new(ey, z_max, ex));
        self.hex_upper.push(Point3::new(fy, z_max, fx));
 
        self.hex_lower.clear();
        for i in 0..6 {
            self.hex_lower.push(self.hex_upper[i] - Vector3::new(0.0, z_max-z_min, 0.0));
        }

        let mut vertices = Vec::new();
        vertices.extend(&self.hex_upper);
        vertices.extend(&self.hex_lower);
        let indices = vec![Point3::new(0, 1, 2),  Point3::new(0, 2, 3),   Point3::new(0, 3, 5),  Point3::new(5, 3, 4),          // upper facet
                           Point3::new(6, 7, 8),  Point3::new(8, 9, 6),   Point3::new(9, 11, 6), Point3::new(9, 10, 11),        // lower facet
                           Point3::new(0, 6, 1),  Point3::new(6, 7, 1),   Point3::new(1, 7, 2),  Point3::new(7, 8, 2),          // side facets, simplified
                           Point3::new(2, 8, 3),  Point3::new(8, 9, 3),   Point3::new(3, 9, 4),  Point3::new(9, 10, 4),      
                           Point3::new(4, 10, 5), Point3::new(10, 11, 5), Point3::new(5, 11, 0), Point3::new(11, 6, 0)];       
        // let indices = vec![Point3::new(0u16, 1, 2), Point3::new(1, 2,3)];
    
        let mesh = Rc::new(RefCell::new(Mesh::new(
            vertices, indices, None, None, true,
        )));
        let mut c = self.hexagon.add_mesh(mesh, Vector3::new(1.0, 1.0, 1.0));
        // c.set_lines_width(3.0);
        // c.set_lines_color(Some(Point3::new(0.0, 0.0, 0.0)));
        // c.set_surface_rendering_activation(false);
        c.set_color(GOLD.0, GOLD.1, GOLD.2);
        c.enable_backface_culling(false);
        c.recompute_normals();
    }

    pub fn draw_hexgon_outline(&self, window: &mut Window) {
        for i in 0..self.hex_upper.len() {
            window.draw_line(&self.hex_upper[i], &self.hex_upper[(i+1)%6], &Point3::new(0.0, 0.0, 0.0));
            window.draw_line(&self.hex_lower[i], &self.hex_lower[(i+1)%6], &Point3::new(0.0, 0.0, 0.0));
            window.draw_line(&self.hex_upper[i], &self.hex_lower[i], &Point3::new(0.0, 0.0, 0.0));
        }
    }

    pub fn add_substrate(&mut self, flake: &Crystal) {
        // println!("substrate position: {:?}", flake.substrate_pos);
        let thickness = 1.0;
        let mut c = self.substrate.add_cube((flake.extrema.y_max - flake.extrema.y_min)*2.0, 
                                            thickness, 
                                            (flake.extrema.x_max - flake.extrema.x_min)*2.0);
        let z_pos = self.lattice.get_xyz(IJK{i: CENTER.i, j: CENTER.j, k: flake.substrate_pos}).z - thickness/2.0 + DIAMETER/3.0;
        let center = &Translation3::new(0.0, z_pos, 0.0,);
        c.append_translation(center);
        c.set_color( 1.5, 1.7, 2.0 );       // colors coordinates above 1.0 like the planar colors minus one

    }

    pub fn update_boundaries(&mut self, window: &mut Window, flake: &Crystal) {
        // update the representation of the wireframe, hexagon and substrate
        window.remove_node(&mut self.wireframe);
        self.wireframe = window.add_group();
        self.add_wireframe(&flake);
        self.wireframe.set_visible(self.show.wireframe);
        window.remove_node(&mut self.hexagon);
        self.hexagon = window.add_group();
        self.add_hexagon(&flake);
        self.hexagon.set_visible(self.show.hexagon);
        if flake.substrate_pos > 0 {
            window.remove_node(&mut self.substrate);
            self.substrate = window.add_group();
            self.add_substrate(&flake);
            self.substrate.set_visible(self.show.substrate);
        }
    }
}

// one could also get &Translation3<f32> directly via scene.lattice.position(IJK) but then you would borrow twice -- immutable and mutable -- which is not possible
// in order to get that work one had to implement add_atom_to_group for every group specifically
pub fn add_atom_to_group(group: &mut SceneNode, trans: &Translation3<f32>, color: Color) {
    let mut s = group.add_sphere(DIAMETER/2.);

    // cubes do not make a difference
    // let mut s = group.add_cube(DIAMETER, DIAMETER, DIAMETER);
    // let rot = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.7);
    // s.append_rotation_wrt_center(&rot);

    s.set_color(color.0, color.1, color.2);
    s.append_translation(trans);
}