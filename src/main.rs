#[macro_use]
extern crate glium;

#[allow(unused_imports)]
use glium::{glutin, Surface};


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



pub struct Window {
    width: u32,
    height: u32,
}

impl Window {
    pub fn open(&mut self) {

        let event_loop = glutin::event_loop::EventLoop::new();

        let wb = glutin::window::WindowBuilder::new()
            .with_inner_size(glutin::dpi::Size::Logical(glutin::dpi::LogicalSize::new(400.0, 400.0)));
    
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
    
            // t += 0.002;
    
            if t > 0.5 {
                t = -0.5;
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
}