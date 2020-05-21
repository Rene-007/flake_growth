use std::time::Duration;
use separator::Separatable;
use std::rc::Rc;

use kiss3d::window::Window;
use kiss3d::scene::PlanarSceneNode;
use kiss3d::text::Font;
// use nalgebra::{UnitQuaternion, Vector3};

use nalgebra::Translation2;
use nalgebra::{Point2, Point3};

use crate::parameters::*;
use crate::crystal::*;


const INDIX: [f32; 6] = [40.0, 40.0, 40.0, 420.0, 50.0, 40.0];                              // indicator pos x
const INDIY: [f32; 6] = [20.0, 80.0, 140.0, 200.0, 225.0, 40.0];                            // indicator pos y
const INDI_STEP: [f32; 2] = [60.0, 50.0];
const INDI_HEIGHT: f32 = 15.0;
const INDI_LENGTH: f32 = 150.0;

pub struct PlanarScene {
    pub scene: PlanarSceneNode,
    pub layers: PlanarSceneNode,
    pub help: PlanarSceneNode,
    pub show_help: bool,
    pub indicators: Vec<PlanarSceneNode>,
    pub show: bool,
    font: Rc<Font>,
    pub added_atoms: usize,
    pub duration: Duration,
}

impl PlanarScene {
    pub fn new(window: &mut Window) -> Self {
        let mut scene = window.add_planar_group();
        let mut indicators = Vec::new();
        for _index in 0..VAC_LISTS {
            indicators.push(scene.add_rectangle(1.0, INDI_HEIGHT));
        }
        let show = true;
        scene.set_visible(show);
        PlanarScene{ 
            scene, 
            layers:         window.add_planar_group(), 
            help:           window.add_planar_group(), 
            show_help:      true,
            indicators, 
            show, 
            font:           Font::default(), 
            added_atoms:    0, 
            duration:       Duration::new(0,0)
        }
    }


    pub fn draw_scene(&mut self, window: &mut Window, flake: &Crystal) {
        // top left -- atom numbers
        let atoms = format!("Atoms:");
        window.draw_text(&atoms[..], &Point2::new(INDIX[0], INDIY[0]), 50.0, &self.font, &Point3::new(0.0, 0.0, 0.0));
        let usage = format!("{}",flake.bulk.number_of_atoms.separated_string());
        let x_pos = INDIX[3]- 20.0*(usage.len() as f32);
        window.draw_text(&usage[..], &Point2::new(x_pos, INDIY[0]), 50.0, &self.font, &Point3::new(0.0, 0.0, 0.0));

        let surface_atoms = format!("Surface:");
        window.draw_text(&surface_atoms[..], &Point2::new(INDIX[1], INDIY[1]), 50.0, &self.font, &Point3::new(0.0, 0.0, 0.0));
        let usage = format!("{}", flake.surface.list.len().separated_string());
        let x_pos = INDIX[3] - 20.0*(usage.len() as f32);
        window.draw_text(&usage[..], &Point2::new(x_pos, INDIY[1]), 50.0, &self.font, &Point3::new(0.0, 0.0, 0.0));
        
        window.draw_text("Vacancies:", &Point2::new(INDIX[2], INDIY[2]), 50.0, &self.font, &Point3::new(0.0, 0.0, 0.0));
        let prob = format!("p {:2?}", flake.prob_list_num +1);
        window.draw_text(&prob[..], &Point2::new(INDIX[3] + 50.0, INDIY[2] + 5.0), 38.0, &self.font, &Point3::new(0.5, 0.5, 0.5));
        for (index,list) in flake.vacancies.list.iter().enumerate() {
            let number = format!("{}", index + 1);
            let usage = format!("{}", list.len().separated_string());
            let prob = format!("{:2}", flake.prob_list_log[index]);
            
            let x_pos = INDIX[3] - 20.0*(usage.len() as f32);
            let y_pos = INDIY[3] +(index as f32)*INDI_STEP[1];
            window.draw_text(&number[..], &Point2::new(INDIX[2], y_pos + 5.0), 40.0, &self.font, &Point3::new(0.5, 0.5, 0.5));
            window.draw_text(&usage[..], &Point2::new(x_pos, y_pos), 50.0, &self.font, &Point3::new(0.0, 0.0, 0.0));
            window.draw_text("10", &Point2::new(INDIX[3] + 50.0, y_pos + 5.0), 40.0, &self.font, &Point3::new(0.5, 0.5, 0.5));
            window.draw_text(&prob[..], &Point2::new(INDIX[3] + 85.0, y_pos), 30.0, &self.font, &Point3::new(0.5, 0.5, 0.5));
        }
        
        // bottom left -- timing      
        let mut added_atoms_string = format!("");
        let mut duration_string = format!("Start");
        if self.added_atoms > 0 {
            added_atoms_string = format!("Added atoms: {}", self.added_atoms.separated_string());
            duration_string = format!("Duration: {:#?}", self.duration);
        }
        window.draw_text(&added_atoms_string[..], &Point2::new(INDIX[0], (2*window.height()-140) as f32), 50.0, &self.font, &Point3::new(0.0, 0.0, 0.0));
        window.draw_text(&duration_string[..], &Point2::new(INDIX[0], (2*window.height()-80) as f32), 50.0, &self.font, &Point3::new(0.0, 0.0, 0.0));   
    
        // bottom right -- flake size
        let [h,w,d,r] = &flake.get_size();
        let [ax,ay,bx,by,cx,cy,..] = &flake.get_hexagon();
        let len1 = ((ax-bx).powi(2) + (ay-by).powi(2)).sqrt();
        let len2 = ((cx-bx).powi(2) + (cy-by).powi(2)).sqrt();
        let x_pos = (2*window.width()-350) as f32;
        let height = format!("Height:  {:.2}",h);
        window.draw_text(&height[..], &Point2::new(x_pos, (2*window.height()-320) as f32), 50.0, &self.font, &Point3::new(0.0, 0.0, 0.0));
        let width = format!("Width:   {:.2}",w);
        window.draw_text(&width[..], &Point2::new(x_pos, (2*window.height()-260) as f32), 50.0, &self.font, &Point3::new(0.0, 0.0, 0.0));
        let depth = format!("Depth:   {:.2}",d);
        window.draw_text(&depth[..], &Point2::new(x_pos, (2*window.height()-200) as f32), 50.0, &self.font, &Point3::new(0.0, 0.0, 0.0));
        let ratio = format!("A-Ratio:   {:2.2}",r);
        window.draw_text(&ratio[..], &Point2::new(x_pos, (2*window.height()-140) as f32), 50.0, &self.font, &Point3::new(0.0, 0.0, 0.0));
        let ratio = format!("L-Ratio:   {:1.3}", len1 / (len1+len2));
        window.draw_text(&ratio[..], &Point2::new(x_pos, (2*window.height()-80) as f32), 50.0, &self.font, &Point3::new(0.0, 0.0, 0.0));
    }       

    pub fn draw_key(&mut self, window: &mut Window, key: &str, rel_x: f32, rel_y: f32) {  
        self.draw_wide_key(window, key, rel_x, rel_y, 50.0);
    }

    pub fn draw_wide_key(&mut self, window: &mut Window, key: &str, rel_x: f32, rel_y: f32, width: f32) {
        // let trans = &Translation2::new(-pos_x/8.0, pos_y/8.0);
        let win_x = window.width() as f32;
        let win_y = window.height() as f32;
        let trans = &Translation2::new(rel_x/200.0*win_x, rel_y/200.0*win_y);
        let mut outline = self.help.add_rectangle(width, 50.0);
        outline.append_translation(trans);
        outline.set_color(0.8, 0.8, 0.8);
        let text_x = (rel_x/100.0 + 1.0)*win_x - width/2.0 - 10.0;
        let text_y = (-rel_y/100.0 + 1.0)*win_y - 30.0;
        window.draw_text(key, &Point2::new(text_x, text_y), 60.0, &self.font, &Point3::new(0.0, 0.0, 0.0));       

    }

    pub fn draw_header(&mut self, window: &mut Window, key: &str, rel_x: f32, rel_y: f32) {
        let win_x = window.width() as f32;
        let win_y = window.height() as f32;
        let text_x = (rel_x/100.0 + 1.0)*win_x - 50.0;
        let text_y = (-rel_y/100.0 + 1.0)*win_y + 15.0;
        window.draw_text(key, &Point2::new(text_x, text_y), 60.0, &self.font, &Point3::new(0.0, 0.0, 0.0));       
    }

    pub fn draw_text(&mut self, window: &mut Window, key: &str, rel_x: f32, rel_y: f32) {
        let win_x = window.width() as f32;
        let win_y = window.height() as f32;
        let text_x = (rel_x/100.0 + 1.0)*win_x - 50.0;
        let text_y = (-rel_y/100.0 + 1.0)*win_y - 20.0;
        window.draw_text(key, &Point2::new(text_x, text_y), 50.0, &self.font, &Point3::new(0.0, 0.0, 0.0));       

    }



    pub fn show_help(&mut self, window: &mut Window) {
        // redraw scene (unfortunately needed)
        window.remove_planar_node(&mut self.help);
        self.help = window.add_planar_group();

        let y: f32 = 90.0;
        self.draw_text(window, "René Kullock", -47.5, y);
        self.draw_text(window, "Spring 2020", 70.5, y);


        // navigation keys
        let x: f32 = -40.0;
        let y: f32 = 40.0;
        let dx: f32 = 7.5;
        let dy: f32 = 15.0;
        self.draw_header(window, "Navigation", x - dx, y + 2.0*dy);
        self.draw_key(window, " Q", x - dx, y + dy);
        self.draw_key(window, " W", x     , y + dy);
        self.draw_key(window, " E", x + dx, y + dy);
        self.draw_key(window, " A", x - dx, y);
        // self.draw_key(window, "S", x     , y);
        self.draw_key(window, " D", x + dx, y);
        self.draw_key(window, " Y", x - dx, y - dy);
        self.draw_key(window, " X", x     , y - dy);
        self.draw_key(window, " C", x + dx, y - dy);

        // show/hide keys
        let x: f32 = -40.0;
        let y: f32 = -17.5;
        let dx: f32 = 7.5;
        let dy: f32 = 15.0;
        let dt: f32 = 16.0;
        self.draw_header(window, "Show/Hide", x - dx, y + 2.2*dy);
        self.draw_text(window, "int[r]uments", x - dx - 1.8*dt, y + dy);
        self.draw_key(window, " R", x - dx, y + dy);
        self.draw_text(window, "dir[t]", x - dt, y + dy);
        self.draw_key(window, " T", x     , y + dy);
        // self.draw_key(window, "Z", x + dx, y + dy);
        self.draw_text(window, "la[s]t atom", x + 1.2*dt, y + dy*1.2);
        self.draw_key(window, " S", x + dx*1.2, y + dy*1.2);
        self.draw_text(window, "sur[f]ace", x - dx - 1.8*dt, y);
        self.draw_key(window, " F", x - dx, y);
        self.draw_text(window, "[g]old", x - dt, y);
        self.draw_key(window, " G", x     , y);
        self.draw_text(window, "[h]exagon", x + dt, y);
        self.draw_key(window, " H", x + dx, y);
        self.draw_text(window, "[v]isualize", x - dx - 1.8*dt, y - dy);
        self.draw_key(window, " V", x - dx, y - dy);
        self.draw_text(window, "[b]ox", x - dt, y - dy);
        self.draw_key(window, " B", x     , y - dy);
        // self.draw_key(window, "N", x + dx, y - dy);
        self.draw_text(window, "substrate", x + 1.2*dt, y - dy*1.2);
        self.draw_key(window, " -", x + dx*1.2, y - dy*1.2);
        
        // add structures
        let x: f32 = 37.5;
        let y: f32 = -17.5;
        let dx: f32 = 7.5;
        let dy: f32 = 15.0;
        let dt: f32 = 12.0;
        self.draw_header(window, "Add Stuff", x - dx, y + 2.2*dy);
        self.draw_key(window, " U", x - dx, y + dy);
        self.draw_text(window, "rounded", x - dx - dt, y + dy);
        self.draw_key(window, " I", x     , y + dy);
        self.draw_text(window, "dirt layer", x + 2.0*dx, y + dy);
        self.draw_key(window, " O", x + dx, y + dy);
        self.draw_text(window, "   ...Jord", x - dx - dt, y);
        self.draw_key(window, " J", x - dx, y);
        self.draw_text(window, "dipole", x - 0.15*dx, y - 0.6*dy);
        self.draw_key(window, " K", x     , y);
        self.draw_text(window, "gold layer", x + 2.0*dx, y);
        self.draw_key(window, " L", x + dx, y);

        // show/hide vacancies
        let x: f32 = -15.0;
        let y: f32 = 55.0;
        let dx: f32 = 7.5;
        self.draw_header(window, "Show/Hide Vacancies", x, y + dy);
        self.draw_key(window, "F1", x + 0.0*dx, y);
        self.draw_key(window, "F2", x + 1.0*dx, y);
        self.draw_key(window, "F3", x + 2.0*dx, y);
        self.draw_key(window, "F4", x + 3.0*dx, y);
        self.draw_key(window, "F5", x + 4.0*dx, y);
        self.draw_key(window, "F6", x + 5.0*dx, y);
        self.draw_key(window, "F7", x + 6.0*dx, y);
        self.draw_key(window, "F8", x + 7.0*dx, y);
        self.draw_key(window, "F9", x + 8.0*dx, y);

        // random add
        let x: f32 = -15.0;
        let y: f32 = 25.0;
        let dx: f32 = 7.5;
        self.draw_header(window, "Add a Number of Random Atoms", x, y + dy);
        self.draw_key(window, " 1", x + 0.0*dx, y);
        self.draw_key(window, " 2", x + 1.0*dx, y);
        self.draw_key(window, " 3", x + 2.0*dx, y);
        self.draw_key(window, " 4", x + 3.0*dx, y);
        self.draw_key(window, " 5", x + 4.0*dx, y);
        self.draw_key(window, " 6", x + 5.0*dx, y);
        self.draw_key(window, " 7", x + 6.0*dx, y);
        self.draw_key(window, " 8", x + 7.0*dx, y);
        self.draw_key(window, " 9", x + 8.0*dx, y);
        
        // toggle lattice
        let x: f32 = -40.0 + 3.0;
        let y: f32 = -60.0;
        let dx: f32 = 7.5;
        let dy: f32 = 15.0;
        let dt: f32 = 9.0;
        self.draw_header(window, "Stacking Faults", x- dx-3.0, y + dy);
        self.draw_text(window, "add", x - dx - dt, y);
        self.draw_wide_key(window, "PgUp", x - dx, y, 100.0);
        self.draw_wide_key(window, "PgDn", x + dx, y, 100.0);
        self.draw_text(window, "remove", x + 2.5*dx, y);
        self.draw_key(window, " ↑", x + 4.0*dx, y);
        
        // substrate
        let x: f32 = 45.0;
        let y: f32 = -60.0;
        let dy: f32 = 15.0;
        let dt: f32 = 30.0;
        self.draw_header(window, "Add/Remove Substrate", x - dt, y + dy);
        self.draw_key(window, " ↓", x, y);

        // special keys
        let x: f32 = 80.0;
        let y: f32 = 40.0;
        let dy: f32 = 15.0;
        let dt: f32 = 18.0;
        self.draw_header(window, "Special Keys", x - dt, y + 2.0*dy);
        self.draw_text(window, "Probabilities", x - dt, y);
        self.draw_key(window, " P", x, y);
        

        #[cfg(target_arch = "wasm32")]
        {
        self.draw_text(window, "Reset", x - dt, y + dy);
        self.draw_key(window, " :", x, y + dy);

        }
        #[cfg(not(target_arch = "wasm32"))]
        {
        self.draw_text(window, "Reset", x - dt, y + dy);
        self.draw_wide_key(window, "←−−", x  - 2.0, y + dy, 80.0);
        self.draw_text(window, "Statistics", x - dt, y - dy);
        self.draw_key(window, "  ;", x, y - dy);
        }

        // Space
        self.draw_wide_key(window, "Space → show/hide help", 7.5, -85.0, 600.0);

        // window.draw_text("help", &Point2::new(pos_x, pos_y), 50.0, &self.font, &Point3::new(0.0, 0.0, 0.0));       
    }

    pub fn update_indicators(&mut self, window: &mut Window, flake: &Crystal) {
        // vacancies
        let pos_x = -(window.width() as f32)/2.0 + 2.2*INDIX[4];
        let pos_y = (window.height() as f32)/2.0 - 0.5*INDIY[4];
        let step_y = -0.625*INDIY[5];
        for (index, color) in VAC_COLORS.iter().enumerate(){
            let usage = (flake.vacancies.list[index].len() as f32)/(flake.surface.list.len() as f32) *INDI_LENGTH;
            // let usage = (flake.vacancies.list[index].len() as f32)/(flake.vacancies.list[index].capacity() as f32) *INDI_LENGTH;
            self.indicators[index].set_local_scale(usage,INDI_HEIGHT);
            self.indicators[index].set_local_translation(Translation2::new(pos_x + usage/2.0 - INDI_LENGTH/2.0, pos_y + step_y*(index as f32)));
            self.indicators[index].set_color(color.0, color.1, color.2);
        }

        // stacking
        window.remove_planar_node(&mut self.layers);
        self.layers = window.add_planar_group();
        let max_k = flake.extrema_ijk.z_max.k;
        let min_k = flake.extrema_ijk.z_min.k;
        let max = match flake.substrate_pos {
            1 => max_k - min_k,
            _ => max_k - flake.substrate_pos 
        };
        // let max = max_k-min_k;
        let number = std::cmp::max(max,20) as f32;
        let step_y = (window.height() as f32) / (-1.3*number);
        let pos_x = (window.width()/2) as f32 - 50.0;
        let pos_y = (window.height()/4) as f32 - 160.0 - step_y/2.0*(max as f32);
        // let step_y = 30.0;
        for index in min_k..=max_k {
            let trans = &Translation2::new(pos_x, pos_y + step_y*((max_k - index) as f32));
            let layer = (flake.lattice.stacking.pos[index as usize]).rem_euclid(3) as usize;     // a better modulo function
            let mut outline = self.layers.add_circle(8.0);
            outline.append_translation(trans);
            outline.set_color(0.0, 0.0, 0.0);
            let mut atom = self.layers.add_circle(6.0);
            atom.append_translation(trans);
            atom.set_color(LAYER_COLORS[layer].0, LAYER_COLORS[layer].1, LAYER_COLORS[layer].2);
            for faults in &flake.lattice.stacking_faults {
                if index + 1 == *faults { 
                    atom.set_color(0.8, 0.0, 0.0); 
                }
            }
        }
        if flake.substrate_pos > 1 {
            let trans = &Translation2::new(pos_x, pos_y + step_y*((max) as f32));
            let mut outline = self.layers.add_rectangle(30.0, 10.0);
            outline.append_translation(trans);
            outline.set_color(0.0, 0.0, 0.0);
            let mut outline = self.layers.add_rectangle(28.0, 8.0);
            outline.append_translation(trans);
            outline.set_color(0.6, 0.8, 1.1);
        }
    }
}