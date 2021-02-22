#[macro_use]
extern crate glium;

#[allow(unused_imports)]
use glium::{glutin, Surface};

use glium::glutin::dpi::Size::Logical;
use glium::glutin::dpi::LogicalSize;

use glium::glutin::event::Event;
use glium::glutin::event::DeviceEvent;
use glium::glutin::event::KeyboardInput;
use glium::glutin::event::VirtualKeyCode;
use glium::glutin::event::ElementState::Pressed;

use std::time::Duration;
use std::time::Instant;


pub trait GUIController {
    fn title(&self) -> &str;
    fn dimensions(&self) -> (u32, u32);
    fn frames_per_second(&self) -> u32;
    fn next_frame(&mut self) -> &RgbaImage;
}

const VERTEX_SHADER_SRC: &str = r#"
    #version 150

    in vec2 position;
    in vec2 tex_coords;
    out vec2 v_tex_coords;

    void main() {
        v_tex_coords = tex_coords;
        gl_Position = vec4(position, 0.0, 1.0);
    }
"#;

const FRAGMENT_SHADER_SRC: &str = r#"
    #version 150

    in vec2 v_tex_coords;
    out vec4 color;

    uniform sampler2D sampler;

    void main() {
        color = texture(sampler, v_tex_coords);
    }
"#;


pub struct RgbaImage {
    pub width: u32,
    pub height: u32,
    pub bytes: Vec<u8>,
}

pub type RgbaPixel = (u8,u8,u8,u8);

// #[cfg(feature = "image_manipulation")]
impl RgbaImage {
    pub fn nearest_neighbor_scale(img: &RgbaImage, factor: f32) -> RgbaImage {
        let mut new_img = RgbaImage::new(
            (img.width as f32 * factor) as u32,
            (img.height as f32 * factor) as u32,
        );

        // Calculating a ratio of a single pixel's size to the whole image
        let ratio_x = 1.0 / new_img.width as f32;
        let ratio_y = 1.0 / new_img.height as f32;

        for y in 0..new_img.height {
            for x in 0..new_img.width {

                // Determining which x and y values to sample from
                let progress_x = ratio_x * x as f32;
                let progress_y = ratio_y * y as f32;

                let src_x = progress_x * img.width as f32;
                let src_y = progress_y * img.height as f32;

                // Applying the sampled pixel to the output image
                let pixel = img.get_pixel(src_x as u32, src_y as u32);
                new_img.set_pixel(x, y, pixel);
            }
        }

        new_img
    }
}

impl RgbaImage {
    pub fn new(width: u32, height: u32) -> RgbaImage {
        RgbaImage {
            width,
            height,
            bytes: {
                let byte_count = (width * height) * 4;
                let mut bytes = vec![];
                
                for _ in (0..byte_count).step_by(4) {
                    bytes.push(0);
                    bytes.push(0);
                    bytes.push(0);
                    bytes.push(255);
                }
                bytes
            }
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, pixel: RgbaPixel) -> bool {
        if x >= self.width { return false; }
        if y >= self.height { return false; }

        let index = (((self.width * y) + x) * 4) as usize;
        self.bytes[index + 0] = pixel.0;
        self.bytes[index + 1] = pixel.1;
        self.bytes[index + 2] = pixel.2;
        self.bytes[index + 3] = pixel.3;

        true
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> RgbaPixel {
        let index = (((self.width * y) + x) * 4) as usize;
        (
            self.bytes[index + 0],
            self.bytes[index + 1],
            self.bytes[index + 2],
            self.bytes[index + 3],
        )
    }

    pub fn draw(&mut self, img: &RgbaImage, x: i32, y: i32) {
        for img_y in 0..img.height {
            for img_x in 0..img.width {
                let pixel = img.get_pixel(img_x, img_y);

                let canvas_x = x + img_x as i32;
                let canvas_y = y + img_y as i32;

                if canvas_x >= 0 && canvas_y >= 0 {
                    self.set_pixel(
                        canvas_x as u32,
                        canvas_y as u32,
                        pixel
                    );
                }
            }
        }
    }

    pub fn fill(&mut self, color: RgbaPixel) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.set_pixel(x, y, color);
            }
        }
    }
}

pub struct GUI;

impl GUI {
    pub fn launch<T: 'static + GUIController>(mut controller: T) {
        let event_loop = glutin::event_loop::EventLoop::new();

        let (width, height) = controller.dimensions();
        let size = Logical(LogicalSize::new(width as f64, height as f64));

        let wb = glutin::window::WindowBuilder::new()
            .with_title(controller.title())
            .with_inner_size(size);
    
        let cb = glutin::ContextBuilder::new();
        let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    

        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2],
            tex_coords: [f32; 2],
        }
    
        implement_vertex!(Vertex, position, tex_coords);

        let vertex1 = Vertex { position: [-1.0, -1.0 ], tex_coords: [0.0, 0.0] };
        let vertex2 = Vertex { position: [ 1.0, -1.0 ], tex_coords: [1.0, 0.0] };
        let vertex3 = Vertex { position: [ 1.0,  1.0 ], tex_coords: [1.0, 1.0] };
        let vertex4 = Vertex { position: [-1.0,  1.0 ], tex_coords: [0.0, 1.0] };
    
        let shape = vec![vertex1, vertex2, vertex3, vertex4];
        
        let indices: [u16; 6] = [0,1,2,2,3,0];
        let indices = glium::IndexBuffer::new(
            &display,
            glium::index::PrimitiveType::TrianglesList,
            &indices
        ).unwrap();
    
        let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    
        let program = glium::Program::from_source(
            &display,
            VERTEX_SHADER_SRC,
            FRAGMENT_SHADER_SRC,
            None
        ).unwrap();

        // Setting up timekeeping
        let mut last_render = Instant::now();
        let fps = controller.frames_per_second();
        let refresh_interval = Duration::from_millis(1000 / fps as u64);

        // Setting up input state
        let mut input_state = InputState {
            keyboards: vec![
                KeyboardState::default(),
            ]
        };

        event_loop.run(move |event, _, control_flow| {
            // Maybe draw the next frame
            if last_render + refresh_interval < Instant::now() {
                let pixels = &controller.next_frame();

                let image = glium::texture::RawImage2d::from_raw_rgba_reversed(
                    &pixels.bytes,
                    (pixels.width, pixels.height),
                );

                let texture = glium::texture::Texture2d::new(&display, image)
                    .unwrap();

                let uniforms = uniform! {
                        // Applying filters to prevent unwanted image smoothing
                        sampler: texture.sampled()
                            .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
                            .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
                    };
            
                let mut target = display.draw();

                target.draw(&vertex_buffer, &indices, &program, &uniforms,
                    &Default::default()).unwrap();

                target.finish().unwrap();

                last_render = Instant::now();
            }

            // Responding to GUI events
            match event {
                glutin::event::Event::WindowEvent { event, .. } => match event {
                    glutin::event::WindowEvent::CloseRequested => {
                        *control_flow = glutin::event_loop::ControlFlow::Exit;
                        return;
                    },
                    glutin::event::WindowEvent::KeyboardInput { device_id, input, .. } => {
                        apply_keyboard_input(&input, &mut input_state);
                        // println!("KB {:?} {:?}", device_id, input);
                    },
                    _ => return,
                },
                Event::DeviceEvent { event, .. } => {
                    apply_device_event(&event, &mut input_state);
                },
                _ => {}
            }

        });

    }
}

pub struct InputState {
    keyboards: Vec<KeyboardState>,
}

#[derive(Default, Debug)]
pub struct KeyboardState {
    key_1: bool,
    key_2: bool,
    key_3: bool,
    key_4: bool,
    key_5: bool,
    key_6: bool,
    key_7: bool,
    key_8: bool,
    key_9: bool,
    key_0: bool,
    a: bool,
    b: bool,
    c: bool,
    d: bool,
    e: bool,
    f: bool,
    g: bool,
    h: bool,
    i: bool,
    j: bool,
    k: bool,
    l: bool,
    m: bool,
    n: bool,
    o: bool,
    p: bool,
    q: bool,
    r: bool,
    s: bool,
    t: bool,
    u: bool,
    v: bool,
    w: bool,
    x: bool,
    y: bool,
    z: bool,
    escape: bool,
    f1: bool,
    f2: bool,
    f3: bool,
    f4: bool,
    f5: bool,
    f6: bool,
    f7: bool,
    f8: bool,
    f9: bool,
    f10: bool,
    f11: bool,
    f12: bool,
    f13: bool,
    f14: bool,
    f15: bool,
    f16: bool,
    f17: bool,
    f18: bool,
    f19: bool,
    f20: bool,
    f21: bool,
    f22: bool,
    f23: bool,
    f24: bool,
    snapshot: bool,
    scroll: bool,
    pause: bool,
    insert: bool,
    home: bool,
    delete: bool,
    end: bool,
    page_down: bool,
    page_up: bool,
    left: bool,
    up: bool,
    right: bool,
    down: bool,
    back: bool,
    _return: bool,
    space: bool,
    compose: bool,
    caret: bool,
    numlock: bool,
    numpad0: bool,
    numpad1: bool,
    numpad2: bool,
    numpad3: bool,
    numpad4: bool,
    numpad5: bool,
    numpad6: bool,
    numpad7: bool,
    numpad8: bool,
    numpad9: bool,
    numpad_add: bool,
    numpad_divide: bool,
    numpad_decimal: bool,
    numpad_comma: bool,
    numpad_enter: bool,
    numpad_equals: bool,
    numpad_multiply: bool,
    numpad_subtract: bool,
    abnt_c1: bool,
    abnt_c2: bool,
    apostrophe: bool,
    apps: bool,
    asterisk: bool,
    at: bool,
    ax: bool,
    backslash: bool,
    calculator: bool,
    capital: bool,
    colon: bool,
    comma: bool,
    convert: bool,
    equals: bool,
    grave: bool,
    kana: bool,
    kanji: bool,
    l_alt: bool,
    l_bracket: bool,
    l_control: bool,
    l_shift: bool,
    l_win: bool,
    mail: bool,
    media_select: bool,
    media_stop: bool,
    minus: bool,
    mute: bool,
    my_computer: bool,
    navigate_forward: bool,
    navigate_backward: bool,
    next_track: bool,
    no_convert: bool,
    oem102: bool,
    period: bool,
    play_pause: bool,
    plus: bool,
    power: bool,
    prev_track: bool,
    r_alt: bool,
    r_bracket: bool,
    r_control: bool,
    r_shift: bool,
    r_win: bool,
    semicolon: bool,
    slash: bool,
    sleep: bool,
    stop: bool,
    sysrq: bool,
    tab: bool,
    underline: bool,
    unlabeled: bool,
    volume_down: bool,
    volume_up: bool,
    wake: bool,
    web_back: bool,
    web_favorites: bool,
    web_forward: bool,
    web_home: bool,
    web_refresh: bool,
    web_search: bool,
    web_stop: bool,
    yen: bool,
    copy: bool,
    paste: bool,
    cut: bool,
}

fn apply_keyboard_input(input: &KeyboardInput, state: &mut InputState) {
    let keyboard_index = 0;
    let is_pressed = input.state == Pressed;

    match input.virtual_keycode {
        Some(VirtualKeyCode::Key1) => {
            state.keyboards[keyboard_index].key_1 = is_pressed;
        },
        Some(VirtualKeyCode::Key2) => {
            state.keyboards[keyboard_index].key_2 = is_pressed;
        },
        Some(VirtualKeyCode::Key3) => {
            state.keyboards[keyboard_index].key_3 = is_pressed;
        },
        Some(VirtualKeyCode::Key4) => {
            state.keyboards[keyboard_index].key_4 = is_pressed;
        },
        Some(VirtualKeyCode::Key5) => {
            state.keyboards[keyboard_index].key_5 = is_pressed;
        },
        Some(VirtualKeyCode::Key6) => {
            state.keyboards[keyboard_index].key_6 = is_pressed;
        },
        Some(VirtualKeyCode::Key7) => {
            state.keyboards[keyboard_index].key_7 = is_pressed;
        },
        Some(VirtualKeyCode::Key8) => {
            state.keyboards[keyboard_index].key_8 = is_pressed;
        },
        Some(VirtualKeyCode::Key9) => {
            state.keyboards[keyboard_index].key_9 = is_pressed;
        },
        Some(VirtualKeyCode::Key0) => {
            state.keyboards[keyboard_index].key_0 = is_pressed;
        },
        Some(VirtualKeyCode::A) => {
            state.keyboards[keyboard_index].a = is_pressed;
        },
        Some(VirtualKeyCode::B) => {
            state.keyboards[keyboard_index].b = is_pressed;
        },
        Some(VirtualKeyCode::C) => {
            state.keyboards[keyboard_index].c = is_pressed;
        },
        Some(VirtualKeyCode::D) => {
            state.keyboards[keyboard_index].d = is_pressed;
        },
        Some(VirtualKeyCode::E) => {
            state.keyboards[keyboard_index].e = is_pressed;
        },
        Some(VirtualKeyCode::F) => {
            state.keyboards[keyboard_index].f = is_pressed;
        },
        Some(VirtualKeyCode::G) => {
            state.keyboards[keyboard_index].g = is_pressed;
        },
        Some(VirtualKeyCode::H) => {
            state.keyboards[keyboard_index].h = is_pressed;
        },
        Some(VirtualKeyCode::I) => {
            state.keyboards[keyboard_index].i = is_pressed;
        },
        Some(VirtualKeyCode::J) => {
            state.keyboards[keyboard_index].j = is_pressed;
        },
        Some(VirtualKeyCode::K) => {
            state.keyboards[keyboard_index].k = is_pressed;
        },
        Some(VirtualKeyCode::L) => {
            state.keyboards[keyboard_index].l = is_pressed;
        },
        Some(VirtualKeyCode::M) => {
            state.keyboards[keyboard_index].m = is_pressed;
        },
        Some(VirtualKeyCode::N) => {
            state.keyboards[keyboard_index].n = is_pressed;
        },
        Some(VirtualKeyCode::O) => {
            state.keyboards[keyboard_index].o = is_pressed;
        },
        Some(VirtualKeyCode::P) => {
            state.keyboards[keyboard_index].p = is_pressed;
        },
        Some(VirtualKeyCode::Q) => {
            state.keyboards[keyboard_index].q = is_pressed;
        },
        Some(VirtualKeyCode::R) => {
            state.keyboards[keyboard_index].r = is_pressed;
        },
        Some(VirtualKeyCode::S) => {
            state.keyboards[keyboard_index].s = is_pressed;
        },
        Some(VirtualKeyCode::T) => {
            state.keyboards[keyboard_index].t = is_pressed;
        },
        Some(VirtualKeyCode::U) => {
            state.keyboards[keyboard_index].u = is_pressed;
        },
        Some(VirtualKeyCode::V) => {
            state.keyboards[keyboard_index].v = is_pressed;
        },
        Some(VirtualKeyCode::W) => {
            state.keyboards[keyboard_index].w = is_pressed;
        },
        Some(VirtualKeyCode::X) => {
            state.keyboards[keyboard_index].x = is_pressed;
        },
        Some(VirtualKeyCode::Y) => {
            state.keyboards[keyboard_index].y = is_pressed;
        },
        Some(VirtualKeyCode::Z) => {
            state.keyboards[keyboard_index].z = is_pressed;
        },
        Some(VirtualKeyCode::Escape) => {
            state.keyboards[keyboard_index].escape = is_pressed;
        },
        Some(VirtualKeyCode::F1) => {
            state.keyboards[keyboard_index].f1 = is_pressed;
        },
        Some(VirtualKeyCode::F2) => {
            state.keyboards[keyboard_index].f2 = is_pressed;
        },
        Some(VirtualKeyCode::F3) => {
            state.keyboards[keyboard_index].f3 = is_pressed;
        },
        Some(VirtualKeyCode::F4) => {
            state.keyboards[keyboard_index].f4 = is_pressed;
        },
        Some(VirtualKeyCode::F5) => {
            state.keyboards[keyboard_index].f5 = is_pressed;
        },
        Some(VirtualKeyCode::F6) => {
            state.keyboards[keyboard_index].f6 = is_pressed;
        },
        Some(VirtualKeyCode::F7) => {
            state.keyboards[keyboard_index].f7 = is_pressed;
        },
        Some(VirtualKeyCode::F8) => {
            state.keyboards[keyboard_index].f8 = is_pressed;
        },
        Some(VirtualKeyCode::F9) => {
            state.keyboards[keyboard_index].f9 = is_pressed;
        },
        Some(VirtualKeyCode::F10) => {
            state.keyboards[keyboard_index].f10 = is_pressed;
        },
        Some(VirtualKeyCode::F11) => {
            state.keyboards[keyboard_index].f11 = is_pressed;
        },
        Some(VirtualKeyCode::F12) => {
            state.keyboards[keyboard_index].f12 = is_pressed;
        },
        Some(VirtualKeyCode::F13) => {
            state.keyboards[keyboard_index].f13 = is_pressed;
        },
        Some(VirtualKeyCode::F14) => {
            state.keyboards[keyboard_index].f14 = is_pressed;
        },
        Some(VirtualKeyCode::F15) => {
            state.keyboards[keyboard_index].f15 = is_pressed;
        },
        Some(VirtualKeyCode::F16) => {
            state.keyboards[keyboard_index].f16 = is_pressed;
        },
        Some(VirtualKeyCode::F17) => {
            state.keyboards[keyboard_index].f17 = is_pressed;
        },
        Some(VirtualKeyCode::F18) => {
            state.keyboards[keyboard_index].f18 = is_pressed;
        },
        Some(VirtualKeyCode::F19) => {
            state.keyboards[keyboard_index].f19 = is_pressed;
        },
        Some(VirtualKeyCode::F20) => {
            state.keyboards[keyboard_index].f20 = is_pressed;
        },
        Some(VirtualKeyCode::F21) => {
            state.keyboards[keyboard_index].f21 = is_pressed;
        },
        Some(VirtualKeyCode::F22) => {
            state.keyboards[keyboard_index].f22 = is_pressed;
        },
        Some(VirtualKeyCode::F23) => {
            state.keyboards[keyboard_index].f23 = is_pressed;
        },
        Some(VirtualKeyCode::F24) => {
            state.keyboards[keyboard_index].f24 = is_pressed;
        },
        Some(VirtualKeyCode::Snapshot) => {
            state.keyboards[keyboard_index].snapshot = is_pressed;
        },
        Some(VirtualKeyCode::Scroll) => {
            state.keyboards[keyboard_index].scroll = is_pressed;
        },
        Some(VirtualKeyCode::Pause) => {
            state.keyboards[keyboard_index].pause = is_pressed;
        },
        Some(VirtualKeyCode::Insert) => {
            state.keyboards[keyboard_index].insert = is_pressed;
        },
        Some(VirtualKeyCode::Home) => {
            state.keyboards[keyboard_index].home = is_pressed;
        },
        Some(VirtualKeyCode::Delete) => {
            state.keyboards[keyboard_index].delete = is_pressed;
        },
        Some(VirtualKeyCode::End) => {
            state.keyboards[keyboard_index].end = is_pressed;
        },
        Some(VirtualKeyCode::PageDown) => {
            state.keyboards[keyboard_index].page_down = is_pressed;
        },
        Some(VirtualKeyCode::PageUp) => {
            state.keyboards[keyboard_index].page_up = is_pressed;
        },
        Some(VirtualKeyCode::Left) => {
            state.keyboards[keyboard_index].left = is_pressed;
        },
        Some(VirtualKeyCode::Up) => {
            state.keyboards[keyboard_index].up = is_pressed;
        },
        Some(VirtualKeyCode::Right) => {
            state.keyboards[keyboard_index].right = is_pressed;
        },
        Some(VirtualKeyCode::Down) => {
            state.keyboards[keyboard_index].down = is_pressed;
        },
        Some(VirtualKeyCode::Back) => {
            state.keyboards[keyboard_index].back = is_pressed;
        },
        Some(VirtualKeyCode::Return) => {
            state.keyboards[keyboard_index]._return = is_pressed;
        },
        Some(VirtualKeyCode::Space) => {
            state.keyboards[keyboard_index].space = is_pressed;
        },
        Some(VirtualKeyCode::Compose) => {
            state.keyboards[keyboard_index].compose = is_pressed;
        },
        Some(VirtualKeyCode::Caret) => {
            state.keyboards[keyboard_index].caret = is_pressed;
        },
        Some(VirtualKeyCode::Numlock) => {
            state.keyboards[keyboard_index].numlock = is_pressed;
        },
        Some(VirtualKeyCode::Numpad0) => {
            state.keyboards[keyboard_index].numpad0 = is_pressed;
        },
        Some(VirtualKeyCode::Numpad1) => {
            state.keyboards[keyboard_index].numpad1 = is_pressed;
        },
        Some(VirtualKeyCode::Numpad2) => {
            state.keyboards[keyboard_index].numpad2 = is_pressed;
        },
        Some(VirtualKeyCode::Numpad3) => {
            state.keyboards[keyboard_index].numpad3 = is_pressed;
        },
        Some(VirtualKeyCode::Numpad4) => {
            state.keyboards[keyboard_index].numpad4 = is_pressed;
        },
        Some(VirtualKeyCode::Numpad5) => {
            state.keyboards[keyboard_index].numpad5 = is_pressed;
        },
        Some(VirtualKeyCode::Numpad6) => {
            state.keyboards[keyboard_index].numpad6 = is_pressed;
        },
        Some(VirtualKeyCode::Numpad7) => {
            state.keyboards[keyboard_index].numpad7 = is_pressed;
        },
        Some(VirtualKeyCode::Numpad8) => {
            state.keyboards[keyboard_index].numpad8 = is_pressed;
        },
        Some(VirtualKeyCode::Numpad9) => {
            state.keyboards[keyboard_index].numpad9 = is_pressed;
        },
        Some(VirtualKeyCode::NumpadAdd) => {
            state.keyboards[keyboard_index].numpad_add = is_pressed;
        },
        Some(VirtualKeyCode::NumpadDivide) => {
            state.keyboards[keyboard_index].numpad_divide = is_pressed;
        },
        Some(VirtualKeyCode::NumpadDecimal) => {
            state.keyboards[keyboard_index].numpad_decimal = is_pressed;
        },
        Some(VirtualKeyCode::NumpadComma) => {
            state.keyboards[keyboard_index].numpad_comma = is_pressed;
        },
        Some(VirtualKeyCode::NumpadEnter) => {
            state.keyboards[keyboard_index].numpad_enter = is_pressed;
        },
        Some(VirtualKeyCode::NumpadEquals) => {
            state.keyboards[keyboard_index].numpad_equals = is_pressed;
        },
        Some(VirtualKeyCode::NumpadMultiply) => {
            state.keyboards[keyboard_index].numpad_multiply = is_pressed;
        },
        Some(VirtualKeyCode::NumpadSubtract) => {
            state.keyboards[keyboard_index].numpad_subtract = is_pressed;
        },
        Some(VirtualKeyCode::AbntC1) => {
            state.keyboards[keyboard_index].abnt_c1 = is_pressed;
        },
        Some(VirtualKeyCode::AbntC2) => {
            state.keyboards[keyboard_index].abnt_c2 = is_pressed;
        },
        Some(VirtualKeyCode::Apostrophe) => {
            state.keyboards[keyboard_index].apostrophe = is_pressed;
        },
        Some(VirtualKeyCode::Apps) => {
            state.keyboards[keyboard_index].apps = is_pressed;
        },
        Some(VirtualKeyCode::At) => {
            state.keyboards[keyboard_index].at = is_pressed;
        },
        Some(VirtualKeyCode::Ax) => {
            state.keyboards[keyboard_index].ax = is_pressed;
        },
        Some(VirtualKeyCode::Backslash) => {
            state.keyboards[keyboard_index].backslash = is_pressed;
        },
        Some(VirtualKeyCode::Calculator) => {
            state.keyboards[keyboard_index].calculator = is_pressed;
        },
        Some(VirtualKeyCode::Capital) => {
            state.keyboards[keyboard_index].capital = is_pressed;
        },
        Some(VirtualKeyCode::Colon) => {
            state.keyboards[keyboard_index].colon = is_pressed;
        },
        Some(VirtualKeyCode::Comma) => {
            state.keyboards[keyboard_index].comma = is_pressed;
        },
        Some(VirtualKeyCode::Convert) => {
            state.keyboards[keyboard_index].convert = is_pressed;
        },
        Some(VirtualKeyCode::Equals) => {
            state.keyboards[keyboard_index].equals = is_pressed;
        },
        Some(VirtualKeyCode::Grave) => {
            state.keyboards[keyboard_index].grave = is_pressed;
        },
        Some(VirtualKeyCode::Kana) => {
            state.keyboards[keyboard_index].kana = is_pressed;
        },
        Some(VirtualKeyCode::Kanji) => {
            state.keyboards[keyboard_index].kanji = is_pressed;
        },
        Some(VirtualKeyCode::LAlt) => {
            state.keyboards[keyboard_index].l_alt = is_pressed;
        },
        Some(VirtualKeyCode::LBracket) => {
            state.keyboards[keyboard_index].l_bracket = is_pressed;
        },
        Some(VirtualKeyCode::LControl) => {
            state.keyboards[keyboard_index].l_control = is_pressed;
        },
        Some(VirtualKeyCode::LShift) => {
            state.keyboards[keyboard_index].l_shift = is_pressed;
        },
        Some(VirtualKeyCode::LWin) => {
            state.keyboards[keyboard_index].l_win = is_pressed;
        },
        Some(VirtualKeyCode::Mail) => {
            state.keyboards[keyboard_index].mail = is_pressed;
        },
        Some(VirtualKeyCode::MediaSelect) => {
            state.keyboards[keyboard_index].media_select = is_pressed;
        },
        Some(VirtualKeyCode::MediaStop) => {
            state.keyboards[keyboard_index].media_stop = is_pressed;
        },
        Some(VirtualKeyCode::Minus) => {
            state.keyboards[keyboard_index].minus = is_pressed;
        },
        Some(VirtualKeyCode::Mute) => {
            state.keyboards[keyboard_index].mute = is_pressed;
        },
        _ => {},
    }
}

fn apply_device_event(device_event: &DeviceEvent, input_state: &mut InputState) {
    // match device_event {
    //     DeviceEvent::MouseMotion { delta } => {
    //         println!("{:?}", delta);
    //     },
    //     DeviceEvent::Button { state, button } => {
    //         println!("{:?} {:?}", state, button);
    //     },
    //     // DeviceEvent::Key(KeyboardInput { state, virtual_keycode, .. }) => {
    //     DeviceEvent::Key(KeyboardInput) => {
    //         let keyboard_index = 0;
    //         println!("KEYBOARD");
    //     },
    //     _ => {},
    // }
}