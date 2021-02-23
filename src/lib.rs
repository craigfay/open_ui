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
use std::hash::Hasher;
use std::hash::Hash;
use std::collections::hash_map::DefaultHasher;

pub trait UIController {
    fn title(&self) -> &str;
    fn dimensions(&self) -> (u32, u32);
    fn frames_per_second(&self) -> u32;
    fn next_frame(&mut self) -> &RgbaImage;
    fn process_events(&mut self, events: &Vec<UIEvent>);
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

pub struct UI;

impl UI {
    pub fn launch<T: 'static + UIController>(mut controller: T) {
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

        let mut ui_events = vec![];

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

            // Responding to UI events
            match event {
                glutin::event::Event::WindowEvent { event, .. } => match event {
                    glutin::event::WindowEvent::CloseRequested => {
                        *control_flow = glutin::event_loop::ControlFlow::Exit;
                        return;
                    },
                    glutin::event::WindowEvent::KeyboardInput { device_id, input, .. } => {
                        apply_keyboard_input(&device_id, &input, &mut ui_events);
                    },
                    glutin::event::WindowEvent::MouseInput { device_id, state, button, .. } => {
                        apply_mouse_input(&device_id, &state, &button, &mut ui_events);
                    },
                    _ => return,
                },
                Event::DeviceEvent { event, .. } => {
                    //
                },
                _ => {}
            }

            // Processing and flushing events
            controller.process_events(&ui_events);
            ui_events = vec![];

        });

    }
}

fn apply_keyboard_input(
    device_id: &glutin::event::DeviceId,
    input: &glutin::event::KeyboardInput,
    ui_events: &mut Vec<UIEvent>
) {
    let mut hasher = DefaultHasher::new();
    device_id.hash(&mut hasher);
    let device_id = hasher.finish();

    let action = match input.state {
        glutin::event::ElementState::Pressed => KeyboardAction::Press,
        glutin::event::ElementState::Released => KeyboardAction::Release,
    };

    let key = match input.virtual_keycode {
        Some(VirtualKeyCode::Key0) => KeyboardKey::Num0,
        Some(VirtualKeyCode::Key1) => KeyboardKey::Num1,
        Some(VirtualKeyCode::Key2) => KeyboardKey::Num2,
        Some(VirtualKeyCode::Key3) => KeyboardKey::Num3,
        Some(VirtualKeyCode::Key4) => KeyboardKey::Num4,
        Some(VirtualKeyCode::Key5) => KeyboardKey::Num5,
        Some(VirtualKeyCode::Key6) => KeyboardKey::Num6,
        Some(VirtualKeyCode::Key7) => KeyboardKey::Num7,
        Some(VirtualKeyCode::Key8) => KeyboardKey::Num8,
        Some(VirtualKeyCode::Key9) => KeyboardKey::Num9,
        Some(VirtualKeyCode::A) => KeyboardKey::A,
        Some(VirtualKeyCode::B) => KeyboardKey::B,
        Some(VirtualKeyCode::C) => KeyboardKey::C,
        Some(VirtualKeyCode::D) => KeyboardKey::D,
        Some(VirtualKeyCode::E) => KeyboardKey::E,
        Some(VirtualKeyCode::F) => KeyboardKey::F,
        Some(VirtualKeyCode::G) => KeyboardKey::G,
        Some(VirtualKeyCode::H) => KeyboardKey::H,
        Some(VirtualKeyCode::I) => KeyboardKey::I,
        Some(VirtualKeyCode::J) => KeyboardKey::J,
        Some(VirtualKeyCode::K) => KeyboardKey::K,
        Some(VirtualKeyCode::L) => KeyboardKey::L,
        Some(VirtualKeyCode::M) => KeyboardKey::N,
        Some(VirtualKeyCode::N) => KeyboardKey::M,
        Some(VirtualKeyCode::O) => KeyboardKey::O,
        Some(VirtualKeyCode::P) => KeyboardKey::P,
        Some(VirtualKeyCode::Q) => KeyboardKey::Q,
        Some(VirtualKeyCode::R) => KeyboardKey::R,
        Some(VirtualKeyCode::S) => KeyboardKey::S,
        Some(VirtualKeyCode::T) => KeyboardKey::T,
        Some(VirtualKeyCode::U) => KeyboardKey::U,
        Some(VirtualKeyCode::V) => KeyboardKey::V,
        Some(VirtualKeyCode::W) => KeyboardKey::W,
        Some(VirtualKeyCode::X) => KeyboardKey::X,
        Some(VirtualKeyCode::Y) => KeyboardKey::Y,
        Some(VirtualKeyCode::Z) => KeyboardKey::Z,
        Some(VirtualKeyCode::Escape) => KeyboardKey::Escape,
        Some(VirtualKeyCode::F1) => KeyboardKey::F1,
        Some(VirtualKeyCode::F2) => KeyboardKey::F2,
        Some(VirtualKeyCode::F3) => KeyboardKey::F3,
        Some(VirtualKeyCode::F4) => KeyboardKey::F4,
        Some(VirtualKeyCode::F5) => KeyboardKey::F5,
        Some(VirtualKeyCode::F6) => KeyboardKey::F6,
        Some(VirtualKeyCode::F7) => KeyboardKey::F7,
        Some(VirtualKeyCode::F8) => KeyboardKey::F8,
        Some(VirtualKeyCode::F9) => KeyboardKey::F9,
        Some(VirtualKeyCode::F10) => KeyboardKey::F10,
        Some(VirtualKeyCode::F11) => KeyboardKey::F11,
        Some(VirtualKeyCode::F12) => KeyboardKey::F12,
        Some(VirtualKeyCode::F13) => KeyboardKey::F13,
        Some(VirtualKeyCode::F14) => KeyboardKey::F14,
        Some(VirtualKeyCode::F15) => KeyboardKey::F15,
        Some(VirtualKeyCode::F16) => KeyboardKey::F16,
        Some(VirtualKeyCode::F17) => KeyboardKey::F17,
        Some(VirtualKeyCode::F18) => KeyboardKey::F18,
        Some(VirtualKeyCode::F19) => KeyboardKey::F19,
        Some(VirtualKeyCode::F20) => KeyboardKey::F20,
        Some(VirtualKeyCode::F21) => KeyboardKey::F21,
        Some(VirtualKeyCode::F22) => KeyboardKey::F22,
        Some(VirtualKeyCode::F23) => KeyboardKey::F23,
        Some(VirtualKeyCode::F24) => KeyboardKey::F24,
        Some(VirtualKeyCode::Snapshot) => KeyboardKey::Snapshot,
        Some(VirtualKeyCode::Scroll) => KeyboardKey::Scroll,
        Some(VirtualKeyCode::Pause) => KeyboardKey::Pause,
        Some(VirtualKeyCode::Insert) => KeyboardKey::Insert,
        Some(VirtualKeyCode::Home) => KeyboardKey::Home,
        Some(VirtualKeyCode::Delete) => KeyboardKey::Delete,
        Some(VirtualKeyCode::End) => KeyboardKey::Delete,
        Some(VirtualKeyCode::PageDown) => KeyboardKey::Delete,
        Some(VirtualKeyCode::PageUp) => KeyboardKey::Delete,
        Some(VirtualKeyCode::Left) => KeyboardKey::Left,
        Some(VirtualKeyCode::Up) => KeyboardKey::Up,
        Some(VirtualKeyCode::Right) => KeyboardKey::Right,
        Some(VirtualKeyCode::Down) => KeyboardKey::Down,
        Some(VirtualKeyCode::Back) => KeyboardKey::Back,
        Some(VirtualKeyCode::Return) => KeyboardKey::Return,
        Some(VirtualKeyCode::Space) => KeyboardKey::Space,
        Some(VirtualKeyCode::Compose) => KeyboardKey::Compose,
        Some(VirtualKeyCode::Caret) => KeyboardKey::Caret,
        Some(VirtualKeyCode::Numlock) => KeyboardKey::Numlock,
        Some(VirtualKeyCode::Numpad1) => KeyboardKey::Numpad1,
        Some(VirtualKeyCode::Numpad2) => KeyboardKey::Numpad2,
        Some(VirtualKeyCode::Numpad3) => KeyboardKey::Numpad3,
        Some(VirtualKeyCode::Numpad4) => KeyboardKey::Numpad4,
        Some(VirtualKeyCode::Numpad5) => KeyboardKey::Numpad5,
        Some(VirtualKeyCode::Numpad6) => KeyboardKey::Numpad6,
        Some(VirtualKeyCode::Numpad7) => KeyboardKey::Numpad7,
        Some(VirtualKeyCode::Numpad8) => KeyboardKey::Numpad8,
        Some(VirtualKeyCode::Numpad9) => KeyboardKey::Numpad9,
        Some(VirtualKeyCode::NumpadAdd) => KeyboardKey::NumpadAdd,
        Some(VirtualKeyCode::NumpadDivide) => KeyboardKey::NumpadDivide,
        Some(VirtualKeyCode::NumpadDecimal) => KeyboardKey::NumpadDecimal,
        Some(VirtualKeyCode::NumpadComma) => KeyboardKey::NumpadComma,
        Some(VirtualKeyCode::NumpadEnter) => KeyboardKey::NumpadEnter,
        Some(VirtualKeyCode::NumpadEquals) => KeyboardKey::NumpadEquals,
        Some(VirtualKeyCode::NumpadMultiply) => KeyboardKey::NumpadMultiply,
        Some(VirtualKeyCode::NumpadSubtract) => KeyboardKey::NumpadSubtract,
        Some(VirtualKeyCode::AbntC1) => KeyboardKey::AbntC1,
        Some(VirtualKeyCode::AbntC2) => KeyboardKey::AbntC2,
        Some(VirtualKeyCode::Apostrophe) => KeyboardKey::Apostrophe,
        Some(VirtualKeyCode::Apps) => KeyboardKey::Apps,
        Some(VirtualKeyCode::Asterisk) => KeyboardKey::Asterisk,
        Some(VirtualKeyCode::At) => KeyboardKey::At,
        Some(VirtualKeyCode::Ax) => KeyboardKey::Ax,
        Some(VirtualKeyCode::Backslash) => KeyboardKey::Backslash,
        Some(VirtualKeyCode::Calculator) => KeyboardKey::Calculator,
        Some(VirtualKeyCode::Capital) => KeyboardKey::Capital,
        Some(VirtualKeyCode::Colon) => KeyboardKey::Colon,
        Some(VirtualKeyCode::Comma) => KeyboardKey::Comma,
        Some(VirtualKeyCode::Convert) => KeyboardKey::Convert,
        Some(VirtualKeyCode::Equals) => KeyboardKey::Equals,
        Some(VirtualKeyCode::Grave) => KeyboardKey::Grave,
        Some(VirtualKeyCode::Kana) => KeyboardKey::Kana,
        Some(VirtualKeyCode::Kanji) => KeyboardKey::Kanji,
        Some(VirtualKeyCode::LAlt) => KeyboardKey::LAlt,
        Some(VirtualKeyCode::LBracket) => KeyboardKey::LBracket,
        Some(VirtualKeyCode::LControl) => KeyboardKey::LControl,
        Some(VirtualKeyCode::LShift) => KeyboardKey::LShift,
        Some(VirtualKeyCode::LWin) => KeyboardKey::LWin,
        Some(VirtualKeyCode::Mail) => KeyboardKey::Mail,
        Some(VirtualKeyCode::MediaSelect) => KeyboardKey::MediaSelect,
        Some(VirtualKeyCode::MediaStop) => KeyboardKey::MediaStop,
        Some(VirtualKeyCode::Minus) => KeyboardKey::Minus,
        Some(VirtualKeyCode::Mute) => KeyboardKey::Mute,
        Some(VirtualKeyCode::MyComputer) => KeyboardKey::MyComputer,
        Some(VirtualKeyCode::NavigateForward) => KeyboardKey::NavigateForward,
        Some(VirtualKeyCode::NavigateBackward) => KeyboardKey::NavigateBackward,
        Some(VirtualKeyCode::NextTrack) => KeyboardKey::NextTrack,
        Some(VirtualKeyCode::NoConvert) => KeyboardKey::NoConvert,
        Some(VirtualKeyCode::OEM102) => KeyboardKey::OEM102,
        Some(VirtualKeyCode::Period) => KeyboardKey::Period,
        Some(VirtualKeyCode::PlayPause) => KeyboardKey::PlayPause,
        Some(VirtualKeyCode::Plus) => KeyboardKey::Plus,
        Some(VirtualKeyCode::Power) => KeyboardKey::Power,
        Some(VirtualKeyCode::PrevTrack) => KeyboardKey::PrevTrack,
        Some(VirtualKeyCode::RAlt) => KeyboardKey::RAlt,
        Some(VirtualKeyCode::RBracket) => KeyboardKey::RBracket,
        Some(VirtualKeyCode::RControl) => KeyboardKey::RControl,
        Some(VirtualKeyCode::RShift) => KeyboardKey::RShift,
        Some(VirtualKeyCode::RWin) => KeyboardKey::RWin,
        Some(VirtualKeyCode::Semicolon) => KeyboardKey::Semicolon,
        Some(VirtualKeyCode::Slash) => KeyboardKey::Slash,
        Some(VirtualKeyCode::Sleep) => KeyboardKey::Sleep,
        Some(VirtualKeyCode::Stop) => KeyboardKey::Stop,
        Some(VirtualKeyCode::Sysrq) => KeyboardKey::Sysrq,
        Some(VirtualKeyCode::Tab) => KeyboardKey::Tab,
        Some(VirtualKeyCode::Underline) => KeyboardKey::Underline,
        Some(VirtualKeyCode::Unlabeled) => KeyboardKey::Unlabeled,
        Some(VirtualKeyCode::VolumeDown) => KeyboardKey::VolumeDown,
        Some(VirtualKeyCode::VolumeUp) => KeyboardKey::VolumeUp,
        Some(VirtualKeyCode::Wake) => KeyboardKey::Wake,
        Some(VirtualKeyCode::WebBack) => KeyboardKey::WebBack,
        Some(VirtualKeyCode::WebFavorites) => KeyboardKey::WebFavorites,
        Some(VirtualKeyCode::WebForward) => KeyboardKey::WebForward,
        Some(VirtualKeyCode::WebHome) => KeyboardKey::WebHome,
        Some(VirtualKeyCode::WebRefresh) => KeyboardKey::WebRefresh,
        Some(VirtualKeyCode::WebSearch) => KeyboardKey::WebSearch,
        Some(VirtualKeyCode::WebStop) => KeyboardKey::WebStop,
        Some(VirtualKeyCode::Yen) => KeyboardKey::Yen,
        Some(VirtualKeyCode::Copy) => KeyboardKey::Copy,
        Some(VirtualKeyCode::Paste) => KeyboardKey::Paste,
        Some(VirtualKeyCode::Cut) => KeyboardKey::Cut,
        _ => return,
    };

    let keyboard_event = KeyboardEvent {
        device_id,
        action,
        key,
    };

    ui_events.push(UIEvent::Keyboard(keyboard_event));
}

fn apply_mouse_input(
    device_id: &glutin::event::DeviceId,
    state: &glutin::event::ElementState,
    button: &glutin::event::MouseButton,
    ui_events: &mut Vec<UIEvent>,
) {

}

#[derive(Debug, Copy, Clone)]
pub enum KeyboardAction {
    Press,
    Release,
}

#[derive(Debug, Copy, Clone)]
pub enum KeyboardKey {
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Escape,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    Snapshot,
    Scroll,
    Pause,
    Insert,
    Home,
    Delete,
    End,
    PageDown,
    PageUp,
    Left,
    Up,
    Right,
    Down,
    Back,
    Return,
    Space,
    Compose,
    Caret,
    Numlock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadDivide,
    NumpadDecimal,
    NumpadComma,
    NumpadEnter,
    NumpadEquals,
    NumpadMultiply,
    NumpadSubtract,
    AbntC1,
    AbntC2,
    Apostrophe,
    Apps,
    Asterisk,
    At,
    Ax,
    Backslash,
    Calculator,
    Capital,
    Colon,
    Comma,
    Convert,
    Equals,
    Grave,
    Kana,
    Kanji,
    LAlt,
    LBracket,
    LControl,
    LShift,
    LWin,
    Mail,
    MediaSelect,
    MediaStop,
    Minus,
    Mute,
    MyComputer,
    NavigateForward,
    NavigateBackward,
    NextTrack,
    NoConvert,
    OEM102,
    Period,
    PlayPause,
    Plus,
    Power,
    PrevTrack,
    RAlt,
    RBracket,
    RControl,
    RShift,
    RWin,
    Semicolon,
    Slash,
    Sleep,
    Stop,
    Sysrq,
    Tab,
    Underline,
    Unlabeled,
    VolumeDown,
    VolumeUp,
    Wake,
    WebBack,
    WebFavorites,
    WebForward,
    WebHome,
    WebRefresh,
    WebSearch,
    WebStop,
    Yen,
    Copy,
    Paste,
    Cut,
}

#[derive(Debug, Copy, Clone)]
pub struct KeyboardEvent {
    device_id: u64,
    key: KeyboardKey,
    action: KeyboardAction,
}

#[derive(Debug, Copy, Clone)]
pub enum UIEvent {
    Keyboard(KeyboardEvent),
}
