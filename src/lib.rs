#[macro_use]
extern crate glium;

#[allow(unused_imports)]
use glium::{glutin, Surface};
use glium::glutin::dpi::LogicalSize;
use glium::glutin::event::Event;
use glium::glutin::event::DeviceEvent;
use glium::glutin::event::VirtualKeyCode;

use std::time::Duration;
use std::time::Instant;
use std::hash::Hasher;
use std::hash::Hash;
use std::collections::hash_map::DefaultHasher;

pub struct UIBlueprint {
    pub title: String,
    pub dimensions: (u32, u32),
    pub resizeable: bool,
    pub maximized: bool,
    pub preserve_aspect_ratio: bool,
    pub frames_per_second: u32,
}

impl UIBlueprint {
    pub fn default() -> UIBlueprint {
        UIBlueprint {
            title: "".to_string(),
            dimensions: (800, 800),
            resizeable: true,
            maximized: false,
            preserve_aspect_ratio: true,
            frames_per_second: 60,
        }
    }

    pub fn title(self, title: &str) -> UIBlueprint {
        UIBlueprint { title: title.to_string(), ..self }
    }

    pub fn dimensions(self, dimensions: (u32, u32)) -> UIBlueprint {
        UIBlueprint { dimensions, ..self }
    }

    pub fn resizeable(self, resizeable: bool) -> UIBlueprint {
        UIBlueprint { resizeable, ..self }
    }

    pub fn maximized(self, maximized: bool) -> UIBlueprint {
        UIBlueprint { maximized, ..self }
    }

    pub fn preserve_aspect_ratio(self, preserve_aspect_ratio: bool) -> UIBlueprint {
        UIBlueprint { preserve_aspect_ratio, ..self }
    }

    pub fn frames_per_second(self, frames_per_second: u32) -> UIBlueprint {
        UIBlueprint { frames_per_second, ..self }
    }
}

pub trait UIController {
    fn blueprint(&self) -> UIBlueprint;
    fn next_frame(&mut self) -> Option<&RgbaImage>;
    fn process_events(&mut self, events: &Vec<UIEvent>);
}

const VERTEX_SHADER_SRC: &str = r#"
    #version 150

    in vec2 dest;
    in vec2 src;
    out vec2 v_src;

    void main() {
        v_src = src;
        gl_Position = vec4(dest, 0.0, 1.0);
    }
"#;

const FRAGMENT_SHADER_SRC: &str = r#"
    #version 150

    in vec2 v_src;
    out vec4 color;

    uniform sampler2D sampler;

    void main() {
        color = texture(sampler, v_src);
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
                    bytes.push(0);
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

#[derive(Copy, Clone, Debug)]
struct Vertex {
    // The vector denoting the area of incoming textures that will be
    // used for drawing. This could be used to crop incoming textures.
    src: [f32; 2],
    // The vector denoting the area of the window that incoming textures
    // will be drawn onto. This could be used to only draw textures on
    // a partial area of the output.
    dest: [f32; 2],
}

fn calculate_vertices(size: &LogicalSize<f32>, pixels: &RgbaImage) -> Vec<Vertex> {
    let ui_h = size.height;
    let ui_w = size.width;

    // Defining the number that the image will be scaled by
    // to fit nicely on the UI
    let scalar = {
        if ui_w > ui_h { ui_h / pixels.height as f32 }
        else { ui_w / pixels.width as f32 }
    };

    // Defining "actual image width / height"
    let img_w = pixels.width as f32 * scalar;
    let img_h = pixels.height as f32 * scalar;

    // Defining vector magnitudes that will correctly
    // position the 4 vertices.
    let mag_x = img_w / ui_w;
    let mag_y = img_h / ui_h;

    vec![
        Vertex { dest: [-mag_x, -mag_y ], src: [0.0, 0.0] },
        Vertex { dest: [ mag_x, -mag_y ], src: [1.0, 0.0] },
        Vertex { dest: [ mag_x,  mag_y ], src: [1.0, 1.0] },
        Vertex { dest: [-mag_x,  mag_y ], src: [0.0, 1.0] },
    ]
}


pub struct UI;

impl UI {
    pub fn launch<T: 'static + UIController>(mut controller: T) {
        implement_vertex!(Vertex, dest, src);

        let blueprint = controller.blueprint();
        let event_loop = glutin::event_loop::EventLoop::new();

        let (width, height) = blueprint.dimensions;
        let mut size = LogicalSize::new(width as f32, height as f32);
        let preserve_aspect_ratio = blueprint.preserve_aspect_ratio;

        let wb = glutin::window::WindowBuilder::new()
            .with_title(blueprint.title)
            .with_inner_size(size)
            .with_maximized(blueprint.maximized)
            .with_resizable(blueprint.resizeable);

        let cb = glutin::ContextBuilder::new();
        let display = glium::Display::new(wb, cb, &event_loop).unwrap();

        let indices: [u16; 6] = [0,1,2,2,3,0];
        let indices = glium::IndexBuffer::new(
            &display,
            glium::index::PrimitiveType::TrianglesList,
            &indices
        ).unwrap();
    

        let program = glium::Program::from_source(
            &display,
            VERTEX_SHADER_SRC,
            FRAGMENT_SHADER_SRC,
            None
        ).unwrap();

        let mut shape = vec![
            Vertex { dest: [-1.0, -1.0 ], src: [0.0, 0.0] },
            Vertex { dest: [ 1.0, -1.0 ], src: [1.0, 0.0] },
            Vertex { dest: [ 1.0,  1.0 ], src: [1.0, 1.0] },
            Vertex { dest: [-1.0,  1.0 ], src: [0.0, 1.0] },
        ];

        let mut vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();

        // Setting up timekeeping
        let mut last_render = Instant::now();
        let fps = blueprint.frames_per_second;
        let refresh_interval = Duration::from_millis(1000 / fps as u64);

        let mut ui_events = vec![];

        event_loop.run(move |event, _, control_flow| {

            // Maybe draw the next frame
            if last_render + refresh_interval < Instant::now() {
                let maybe_pixels = &controller.next_frame();

                if maybe_pixels.is_none() {
                    return *control_flow = glutin::event_loop::ControlFlow::Exit;
                }

                let pixels = maybe_pixels.unwrap();


                let image = glium::texture::RawImage2d::from_raw_rgba_reversed(
                    &pixels.bytes,
                    (pixels.width, pixels.height),
                );

                // If the aspect ratio of the UI doesn't match that of `image`
                // imposing letterboxing to leave the aspect ratio of `image` unchanged.
                if preserve_aspect_ratio {
                    shape = calculate_vertices(&size, &pixels);
                    // println!("{:?}\n", shape);
                    vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
                }

                let texture = glium::texture::Texture2d::new(&display, image).unwrap();

                let uniforms = uniform! {
                    // Applying filters to prevent unwanted image smoothing
                    sampler: texture.sampled()
                        .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
                        .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
                };

            
                let mut frame = display.draw();

                // Erasing the previous frame. This only seems to be necessary
                // to prevent a flickering on the edges of the drawn frame when
                // the window is maximized.
                frame.clear_color(0.0,0.0,0.0,255.0);

                // Drawing on the next frame
                frame.draw(&vertex_buffer, &indices, &program, &uniforms,
                    &Default::default()).unwrap();

                // Committing the drawn frame
                frame.finish().unwrap();

                // Updating the frame clock
                last_render = Instant::now();

                // Processing and flushing events
                // Should this happen at the beginning or end of each frame?
                controller.process_events(&ui_events);
                ui_events = vec![];
            }

            // Responding to UI events
            match event {
                glutin::event::Event::WindowEvent { event, .. } => match event {
                    glutin::event::WindowEvent::CloseRequested => {
                        *control_flow = glutin::event_loop::ControlFlow::Exit;
                        return;
                    },
                    glutin::event::WindowEvent::KeyboardInput { device_id, input, .. } => {
                        apply_keyboard_event(&device_id, &input, &mut ui_events);
                    },
                    glutin::event::WindowEvent::MouseInput { device_id, state, button, .. } => {
                        apply_mouse_button_event(&device_id, &state, &button, &mut ui_events);
                    },
                    glutin::event::WindowEvent::Resized(phys_size) => {
                        size = phys_size.to_logical(1.0);
                        apply_resize_event(&size, &mut ui_events);
                    },
                    glutin::event::WindowEvent::CursorMoved { device_id, position, .. } => {
                        apply_cursor_movement_event(&device_id, &position, &mut ui_events);
                    },
                    _ => return,
                },
                _ => {}
            }

        });

    }
}

fn hash<T: Hash>(value: T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

fn apply_resize_event(
    size: &glutin::dpi::LogicalSize<f32>,
    ui_events: &mut Vec<UIEvent>,
) {
    ui_events.push(UIEvent::Resize(ResizeEvent {
        width: size.width as u32,
        height: size.height as u32,
    }));
}


fn apply_cursor_movement_event(
    device_id: &glutin::event::DeviceId,
    position:  &glutin::dpi::PhysicalPosition<f64>,
    ui_events: &mut Vec<UIEvent>,
) {
    let position = position.to_logical::<f32>(1.0);

    ui_events.push(UIEvent::CursorMovement(CursorMovementEvent {
        device_id: hash(device_id),
        x: position.x as u32,
        y: position.y as u32,
    }));
}


fn apply_keyboard_event(
    device_id: &glutin::event::DeviceId,
    input: &glutin::event::KeyboardInput,
    ui_events: &mut Vec<UIEvent>
) {
    let device_id = hash(device_id);

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

// Converting glutin mouse events to native mouse button events
fn apply_mouse_button_event(
    device_id: &glutin::event::DeviceId,
    state: &glutin::event::ElementState,
    button: &glutin::event::MouseButton,
    ui_events: &mut Vec<UIEvent>,
) {
    let device_id = hash(device_id);

    // Determining button pressed/released
    let button = match button {
        glutin::event::MouseButton::Left => MouseButton::Left,
        glutin::event::MouseButton::Right => MouseButton::Right,
        glutin::event::MouseButton::Middle => MouseButton::Middle,
        glutin::event::MouseButton::Other(num) => MouseButton::Other(*num),
    };

    let action = match state {
        glutin::event::ElementState::Pressed => MouseButtonAction::Press,
        glutin::event::ElementState::Released => MouseButtonAction::Release,
    };

    let event = MouseButtonEvent {
        device_id,
        button,
        action,
    };

    ui_events.push(UIEvent::MouseButton(event));
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum KeyboardAction {
    Press,
    Release,
}

#[derive(Debug, Copy, Clone, PartialEq)]
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct KeyboardEvent {
    pub device_id: u64,
    pub key: KeyboardKey,
    pub action: KeyboardAction,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MouseButtonEvent {
    pub device_id: u64,
    pub button: MouseButton,
    pub action: MouseButtonAction,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MouseButtonAction {
    Press,
    Release,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CursorMovementEvent {
    pub device_id: u64,
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ResizeEvent {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Copy, Clone)]
pub enum UIEvent {
    Keyboard(KeyboardEvent),
    MouseButton(MouseButtonEvent),
    CursorMovement(CursorMovementEvent),
    Resize(ResizeEvent)
}
