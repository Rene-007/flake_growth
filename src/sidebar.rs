use kiss3d::conrod;
use kiss3d::conrod::position::{Align, Positionable, Direction, Padding, Position, Relative};
use kiss3d::conrod::{widget_ids, widget, Labelable, Sizeable, Widget};
use kiss3d::event::Key;
use kiss3d::window::Window;

use crate::parameters::*;

pub const WIN_W: u32 = 600;
pub const WIN_H: u32 = 420;

const MARGIN: conrod::Scalar = 10.0;
const TITLE_SIZE: conrod::FontSize = 24;
const LABEL_SIZE: conrod::FontSize = 10;

const DISTANCE: f64 = 30.0;
const SHORT_DIST: f64 = 20.0;
const SIDE: f64 = 40.0;
const SHORT_DIST2: f64 = 10.0;
const SIDE2: f64 = 30.0;
const WIDE2: f64 = 70.0;

// Generate ids for all widgets
widget_ids! {
    pub struct Ids {
        canvas,
        scrollbar,
        title,
        help, reset,
        move_around,
        q, w, e,
        a, d,
        y, x, c,
        add_atoms,
        key1, key2, key3, 
        key4, key5, key6, 
        key7, key8, key9,
        add_stacking_faults,
        page_up, page_down, remove_faults, 
        show_hide,
        s, v,
        f, g,
        b, h,
        t, minus,
        copyright,
    }
}


pub struct SideBar {
    pub ids: Ids,
    pub virtual_key: Key,
    // pub ui: conrod::UiCell,
}

impl SideBar {
    pub fn new(window: &mut Window) -> Self { 
        let ids = Ids::new(window.conrod_ui_mut().widget_id_generator());
        let virtual_key = Key::Unknown;
        window.conrod_ui_mut().theme = conrod::Theme {
            name: "SideBar Theme".to_string(),
            padding: Padding::none(),
            x_position: Position::Relative(Relative::Align(Align::Start), None),
            y_position: Position::Relative(Relative::Direction(Direction::Backwards, 20.0), None),
            // background_color: conrod::color::DARK_CHARCOAL,
            // shape_color: conrod::color::LIGHT_CHARCOAL,
            background_color: conrod::color::Color::Rgba(46.0/255.0, 52.0/255.0, 54.0/255.0, 0.75),
            shape_color: conrod::color::Color::Rgba(136.0/255.0, 138.0/255.0, 133.0/255.0, 0.75),
            border_color: conrod::color::BLACK,
            border_width: 0.0,
            label_color: conrod::color::WHITE,
            font_id: None,
            font_size_large: 18,
            font_size_medium: 14,
            font_size_small: 10,
            widget_styling: conrod::theme::StyleMap::default(),
            mouse_drag_threshold: 0.0,
            double_click_threshold: std::time::Duration::from_millis(500),
        };
        // let mut ui = window.conrod_ui_mut().set_widgets();
        Self { ids, virtual_key } 
    }


    // Instantiate the sidebar with the conrod syntax.
    pub fn update(&mut self, window: &mut Window) -> Key {
        
        // reinitialize widget ui
        let ui = &mut window.conrod_ui_mut().set_widgets();
        
        // set back to standard return value 
        self.virtual_key = Key::Unknown;
        
        // Canvas
        widget::Canvas::new()
            .pad(MARGIN)
            .align_right()
            .w(SIDEBARWIDTH as f64)
            .scroll_kids_vertically()
            .set(self.ids.canvas, ui);
        
        // Scrollbar
        widget::Scrollbar::y_axis(self.ids.canvas)
            .auto_hide(true)
            .set(self.ids.scrollbar, ui);

        // Title
        widget::Text::new("Flake Growth")
            .font_size(TITLE_SIZE)
            .mid_top_of(self.ids.canvas)
            .set(self.ids.title, ui);

        // help/reset
        for _press in widget::Button::new()
            .label("help")
            .label_font_size(LABEL_SIZE)
            .mid_left_with_margin_on(self.ids.canvas, MARGIN)
            .down(SHORT_DIST2)
            .w_h(WIDE2, SIDE2)
            .set(self.ids.help, ui)
        {
            self.virtual_key = Key::Space;
        }

        for _press in widget::Button::new()
            .label("reset")
            .label_font_size(LABEL_SIZE)
            .mid_right_with_margin_on(self.ids.canvas, MARGIN)
            .y_position_relative(Relative::Align(Align::Start))
            .w_h(WIDE2, SIDE2)
            .set(self.ids.reset, ui)
        {
            self.virtual_key = Key::Back;
        }

        // move around
        widget::Text::new("move around")
            .padded_w_of(self.ids.canvas, MARGIN)
            .down(DISTANCE)
            .align_middle_x_of(self.ids.canvas)
            .set(self.ids.move_around, ui);

        for _press in widget::Button::new()
            .label("↖")
            .mid_left_with_margin_on(self.ids.canvas, MARGIN)
            .down(SHORT_DIST)
            .w_h(SIDE, SIDE)
            .set(self.ids.q, ui)
        {
            self.virtual_key = Key::Q;
        }

        for _press in widget::Button::new()
            .label("↑")
            .middle()
            .y_position_relative(Relative::Align(Align::Start))
            .w_h(SIDE, SIDE)
            .set(self.ids.w, ui)
        {
            self.virtual_key = Key::W;
        }    

        for _press in widget::Button::new()
            .label("↗")
            .mid_right_with_margin_on(self.ids.canvas, MARGIN)
            .y_position_relative(Relative::Align(Align::Start))
            .w_h(SIDE, SIDE)
            .set(self.ids.e, ui)
        {
            self.virtual_key = Key::E;
        }

        for _press in widget::Button::new()
            .label("←")
            .mid_left_with_margin_on(self.ids.canvas, MARGIN)
            .down(SHORT_DIST)
            .w_h(SIDE, SIDE)
            .set(self.ids.a, ui)
        {
            self.virtual_key = Key::A;
        } 

        for _press in widget::Button::new()
            .label("→")
            .mid_right_with_margin_on(self.ids.canvas, MARGIN)
            .y_position_relative(Relative::Align(Align::Start))
            .w_h(SIDE, SIDE)
            .set(self.ids.d, ui)
        {
            self.virtual_key = Key::D;
        }

        for _press in widget::Button::new()
            .label("↙")
            .mid_left_with_margin_on(self.ids.canvas, MARGIN)
            .down(SHORT_DIST)
            .w_h(SIDE, SIDE)
            .set(self.ids.y, ui)
        {
            self.virtual_key = Key::Y;
        }

        for _press in widget::Button::new()
            .label("↓")
            .middle()
            .y_position_relative(Relative::Align(Align::Start))
            .w_h(SIDE, SIDE)
            .set(self.ids.x, ui)
        {
            self.virtual_key = Key::X;
        }    

        for _press in widget::Button::new()
            .label("↘")
            .mid_right_with_margin_on(self.ids.canvas, MARGIN)
            .y_position_relative(Relative::Align(Align::Start))
            .w_h(SIDE, SIDE)
            .set(self.ids.c, ui)
        {
            self.virtual_key = Key::C;
        }

        // add atoms
        widget::Text::new("add atoms")
            .padded_w_of(self.ids.canvas, MARGIN)
            .down(DISTANCE)
            .align_middle_x_of(self.ids.canvas)
            .set(self.ids.add_atoms, ui);

        for _press in widget::Button::new()
            .label("1")
            .label_font_size(LABEL_SIZE)
            .mid_left_with_margin_on(self.ids.canvas, MARGIN)
            .down(SHORT_DIST)
            .w_h(SIDE, SIDE)
            .set(self.ids.key1, ui)
        {
            self.virtual_key = Key::Key1;
        }

        for _press in widget::Button::new()
            .label("10")
            .label_font_size(LABEL_SIZE)
            .middle()
            .y_position_relative(Relative::Align(Align::Start))
            .w_h(SIDE, SIDE)
            .set(self.ids.key2, ui)
        {
            self.virtual_key = Key::Key2;
        }    

        for _press in widget::Button::new()
            .label("100")
            .label_font_size(LABEL_SIZE)
            .mid_right_with_margin_on(self.ids.canvas, MARGIN)
            .y_position_relative(Relative::Align(Align::Start))
            .w_h(SIDE, SIDE)
            .set(self.ids.key3, ui)
        {
            self.virtual_key = Key::Key3;
        }
        
        for _press in widget::Button::new()
            .label("1\n000")
            .label_font_size(LABEL_SIZE)
            .mid_left_with_margin_on(self.ids.canvas, MARGIN)
            .down(SHORT_DIST)
            .w_h(SIDE, SIDE)
            .set(self.ids.key4, ui)
        {
            self.virtual_key = Key::Key4;
        }

        for _press in widget::Button::new()
            .label("10\n000")
            .label_font_size(LABEL_SIZE)
            .middle()
            .y_position_relative(Relative::Align(Align::Start))
            .w_h(SIDE, SIDE)
            .set(self.ids.key5, ui)
        {
            self.virtual_key = Key::Key5;
        }    

        for _press in widget::Button::new()
            .label("100\n000")
            .label_font_size(LABEL_SIZE)
            .mid_right_with_margin_on(self.ids.canvas, MARGIN)
            .y_position_relative(Relative::Align(Align::Start))
            .w_h(SIDE, SIDE)
            .set(self.ids.key6, ui)
        {
            self.virtual_key = Key::Key6;
        }

        for _press in widget::Button::new()
            .label("1\n000\n000")
            .label_font_size(LABEL_SIZE)
            .mid_left_with_margin_on(self.ids.canvas, MARGIN)
            .down(SHORT_DIST)
            .w_h(SIDE, SIDE)
            .set(self.ids.key7, ui)
        {
            self.virtual_key = Key::Key7;
        }

        for _press in widget::Button::new()
            .label("10\n000\n000")
            .label_font_size(LABEL_SIZE)
            .middle()
            .y_position_relative(Relative::Align(Align::Start))
            .w_h(SIDE, SIDE)
            .set(self.ids.key8, ui)
        {
            self.virtual_key = Key::Key8;
        }    

        for _press in widget::Button::new()
            .label("100\n000\n000")
            .label_font_size(LABEL_SIZE)
            .mid_right_with_margin_on(self.ids.canvas, MARGIN)
            .y_position_relative(Relative::Align(Align::Start))
            .w_h(SIDE, SIDE)
            .set(self.ids.key9, ui)
        {
            self.virtual_key = Key::Key9;
        }

        // add stacking faults
        widget::Text::new("add stacking faults")
            .padded_w_of(self.ids.canvas, MARGIN)
            .down(DISTANCE)
            .align_middle_x_of(self.ids.canvas)
            .set(self.ids.add_stacking_faults, ui);

        for _press in widget::Button::new()
            .label("↑")
            .mid_left_with_margin_on(self.ids.canvas, MARGIN)
            .down(SHORT_DIST)
            .w_h(SIDE, SIDE)
            .set(self.ids.page_up, ui)
        {
            self.virtual_key = Key::PageUp;
        }

        for _press in widget::Button::new()
            .label("↓")
            .middle()
            .y_position_relative(Relative::Align(Align::Start))
            .w_h(SIDE, SIDE)
            .set(self.ids.page_down, ui)
        {
            self.virtual_key = Key::PageDown;
        }  

        for _press in widget::Button::new()
            .label("X")
            .mid_right_with_margin_on(self.ids.canvas, MARGIN)
            .y_position_relative(Relative::Align(Align::Start))
            .w_h(SIDE, SIDE)
            .set(self.ids.remove_faults, ui)
        {
            self.virtual_key = Key::Up;
        }

        // show/hide
        widget::Text::new("show/hide")
            .padded_w_of(self.ids.canvas, MARGIN)
            .down(DISTANCE)
            .align_middle_x_of(self.ids.canvas)
            .set(self.ids.show_hide, ui);

        for _press in widget::Button::new()
            .label("last atom")
            .label_font_size(LABEL_SIZE)
            .mid_left_with_margin_on(self.ids.canvas, MARGIN)
            .down(SHORT_DIST2)
            .w_h(WIDE2, SIDE2)
            .set(self.ids.s, ui)
        {
            self.virtual_key = Key::S;
        }

        for _press in widget::Button::new()
            .label("stacking")
            .label_font_size(LABEL_SIZE)
            .mid_right_with_margin_on(self.ids.canvas, MARGIN)
            .y_position_relative(Relative::Align(Align::Start))
            .w_h(WIDE2, SIDE2)
            .set(self.ids.v, ui)
        {
            self.virtual_key = Key::V;
        }

        for _press in widget::Button::new()
            .label("surface")
            .label_font_size(LABEL_SIZE)
            .mid_left_with_margin_on(self.ids.canvas, MARGIN)
            .down(SHORT_DIST2)
            .w_h(WIDE2, SIDE2)
            .set(self.ids.f, ui)
        {
            self.virtual_key = Key::F;
        }

        for _press in widget::Button::new()
            .label("gold")
            .label_font_size(LABEL_SIZE)
            .mid_right_with_margin_on(self.ids.canvas, MARGIN)
            .y_position_relative(Relative::Align(Align::Start))
            .w_h(WIDE2, SIDE2)
            .set(self.ids.g, ui)
        {
            self.virtual_key = Key::G;
        }

        for _press in widget::Button::new()
            .label("box")
            .label_font_size(LABEL_SIZE)
            .mid_left_with_margin_on(self.ids.canvas, MARGIN)
            .down(SHORT_DIST2)
            .w_h(WIDE2, SIDE2)
            .set(self.ids.b, ui)
        {
            self.virtual_key = Key::B;
        }

        for _press in widget::Button::new()
            .label("hexagon")
            .label_font_size(LABEL_SIZE)
            .mid_right_with_margin_on(self.ids.canvas, MARGIN)
            .y_position_relative(Relative::Align(Align::Start))
            .w_h(WIDE2, SIDE2)
            .set(self.ids.h, ui)
        {
            self.virtual_key = Key::H;
        }

        for _press in widget::Button::new()
            .label("dirt")
            .label_font_size(LABEL_SIZE)
            .mid_left_with_margin_on(self.ids.canvas, MARGIN)
            .down(SHORT_DIST2)
            .w_h(WIDE2, SIDE2)
            .set(self.ids.t, ui)
        {
            self.virtual_key = Key::T;
        }

        for _press in widget::Button::new()
            .label("substrate")
            .label_font_size(LABEL_SIZE)
            .mid_right_with_margin_on(self.ids.canvas, MARGIN)
            .y_position_relative(Relative::Align(Align::Start))
            .w_h(WIDE2, SIDE2)
            .set(self.ids.minus, ui)
        {
            self.virtual_key = Key::Minus;
        }

        // copyright
        widget::Text::new("© René Kullock 2020\n")
            .padded_w_of(self.ids.canvas, MARGIN)
            // .align_middle_x_of(self.ids.canvas)
            .align_right_of(self.ids.canvas)
            .align_bottom_of(self.ids.canvas)
            .set(self.ids.copyright, ui);

        self.virtual_key
    }
}