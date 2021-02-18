#[macro_use]
extern crate glium;

#[allow(unused_imports)]
use glium::{glutin, Surface};

use glium::glutin::dpi::Size::Logical;
use glium::glutin::dpi::LogicalSize;

const VERTEX_SHADER_SRC: &str = r#"
    #version 150

    in vec2 position;
    in vec2 tex_coords;
    out vec2 v_tex_coords;

    uniform mat4 matrix;

    void main() {
        v_tex_coords = tex_coords;
        gl_Position = matrix * vec4(position, 0.0, 1.0);
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

impl RgbaImage {
    pub fn new(width: u32, height: u32) -> RgbaImage {
        RgbaImage {
            width,
            height,
            bytes: {
                let byte_count = (width * height) * 4;
                let mut bytes = vec![];
                
                for i in (0..byte_count).step_by(4) {
                    let i = i as usize;
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
        if 0 < x && x < self.width { return false; }
        if 0 < y && y < self.height { return false; }

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
}

pub struct Window {
    width: u32,
    height: u32,
    canvas: RgbaImage,
}

impl Window {
    pub fn open(&mut self) {

        let event_loop = glutin::event_loop::EventLoop::new();

        let size = Logical(
            LogicalSize::new(self.width as f64, self.height as f64)
        );

        let wb = glutin::window::WindowBuilder::new()
            .with_inner_size(size);
    
        let cb = glutin::ContextBuilder::new();
        let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    
        let mut bytes: Vec<u8> = vec![
            0, 255, 0, 255,
            0, 255, 0, 255,
            0, 255, 0, 255,
    
            255, 0, 0, 255,
            255, 0, 0, 255,
            255, 0, 0, 255,
    
            0, 0, 255, 255,
            0, 0, 255, 255,
            0, 0, 255, 255,
        ];
    
        let image = glium::texture::RawImage2d::from_raw_rgba_reversed(
            &bytes,
            (3, 3) // dimensions
        );
    
        let texture = glium::texture::Texture2d::new(&display, image)
            .unwrap();
    
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2],
            tex_coords: [f32; 2],
        }
    
        implement_vertex!(Vertex, position, tex_coords);
    
        let vertex1 = Vertex { position: [ 0.0,  0.0 ], tex_coords: [0.0, 0.0] };
        let vertex2 = Vertex { position: [ 0.5,  0.0 ], tex_coords: [1.0, 0.0] };
        let vertex3 = Vertex { position: [ 0.5,  0.5 ], tex_coords: [1.0, 1.0] };
        let vertex4 = Vertex { position: [ 0.0,  0.5 ], tex_coords: [0.0, 1.0] };
        
        let shape = vec![vertex1, vertex2, vertex3, vertex4];
        
        let indices: [u16; 6] = [0,1,2,2,3,0];
        let indices = glium::IndexBuffer::new(
            &display,
            glium::index::PrimitiveType::TrianglesList,
            &indices
        ).unwrap();
    
        let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    
        let mut t: f32 = 0.0;
    
        let program = glium::Program::from_source(
            &display,
            VERTEX_SHADER_SRC,
            FRAGMENT_SHADER_SRC,
            None
        ).unwrap();
    
        event_loop.run(move |event, _, control_flow| {
            let next_frame_time = std::time::Instant::now() +
                std::time::Duration::from_nanos(16_666_667);
    
            *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);
    
            match event {

                glutin::event::Event::WindowEvent { event, .. } => match event {
                    glutin::event::WindowEvent::CloseRequested => {
                        *control_flow = glutin::event_loop::ControlFlow::Exit;
                        return;
                    },
                    _ => return,
                },

                glutin::event::Event::NewEvents(cause) => match cause {
                    glutin::event::StartCause::ResumeTimeReached { .. } => (),
                    glutin::event::StartCause::Init => (),
                    _ => return,
                },
                _ => return,
            }
    
            let uniforms = uniform! {
                matrix: [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [ t , 0.0, 0.0, 1.0],
                ],
    
                // Applying filters to prevent unwanted image smoothing
                sampler: texture.sampled()
                    .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
                    .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
            };
    
            let mut target = display.draw();
            target.clear_color(0.0, 0.0, 0.0, 1.0);
    
            target.draw(&vertex_buffer, &indices, &program, &uniforms,
                &Default::default()).unwrap();
    
            target.finish().unwrap();
        });
    }

    pub fn draw(&mut self, img: &RgbaImage, x: i32, y: i32) {
        for img_y in 0..img.height {
            for img_x in 0..img.width {
                let pixel = img.get_pixel(img_x, img_y);
                
                let canvas_x = x + img_x as i32;
                let canvas_y = x + img_x as i32;

                if canvas_x >= 0 && canvas_y >= 0 {
                    self.canvas.set_pixel(
                        canvas_x as u32,
                        canvas_y as u32,
                        pixel
                    );
                }
            }
        }
    }

}

pub struct WindowBuilder {
    width: u32,
    height: u32,
}

impl WindowBuilder {

    pub fn new() -> WindowBuilder {
        WindowBuilder {
            width: 0,
            height: 0,
        }
    }

    pub fn build(&self) -> Window {
        Window {
            width: self.width,
            height: self.height,
            canvas: RgbaImage::new(self.width, self.height),
        }
    }

    pub fn width(self, width: u32) -> WindowBuilder {
        WindowBuilder { width, ..self }
    }

    pub fn height(self, height: u32) -> WindowBuilder {
        WindowBuilder { height, ..self }
    }

}


fn main() {

    let mut window = WindowBuilder::new()
        .width(400)
        .height(400)
        .build();

    window.open();

    let img = RgbaImage {
        width: 3,
        height: 3,
        bytes: vec![
            0, 255, 0, 255,
            0, 255, 0, 255,
            0, 255, 0, 255,
            255, 0, 0, 255,
            255, 0, 0, 255,
            255, 0, 0, 255,
            0, 0, 255, 255,
            0, 0, 255, 255,
            0, 0, 255, 255,
        ],
    };

    window.draw(&img, 0, 0);

}
