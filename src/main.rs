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


// Scale an image using nearest neighbor interpolation
pub fn scale(img: &RgbaImage, factor: f32) -> RgbaImage {
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

            // Determining which x and y values to write to
            let dest_x = ratio_x * new_img.width as f32;
            let dest_y = ratio_y * new_img.height as f32;

            // Applying the sampled pixel to the output image
            let pixel = img.get_pixel(src_x as u32, src_y as u32);
            new_img.set_pixel(x, y, pixel);
        }
    }

    new_img
}

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
        if 0 > x || x >= self.width { return false; }
        if 0 > y || y >= self.height { return false; }

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

pub struct Window;

impl Window {
    pub fn open<T: 'static + WindowController>(mut state_manager: T) {
        let event_loop = glutin::event_loop::EventLoop::new();

        let (width, height) = state_manager.dimensions();
        let size = Logical(LogicalSize::new(width as f64, height as f64));

        let wb = glutin::window::WindowBuilder::new()
            .with_title(state_manager.title())
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

        event_loop.run(move |event, _, control_flow| {

            let pixels = &state_manager.next_frame();

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

            let next_frame_time = std::time::Instant::now() +
                std::time::Duration::from_nanos(6_666_667);

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
                _ => {}
            }

        });

    }
}




pub trait WindowController {
    fn title(&self) -> &str;
    fn dimensions(&self) -> (u32, u32);
    fn next_frame(&mut self) -> &RgbaImage;
}


pub struct MyWindow {
    canvas: RgbaImage,
    xval: i32,
    images: Vec<RgbaImage>,
}

impl MyWindow {
    pub fn new() -> MyWindow {
        let img = RgbaImage {
            width: 3,
            height: 3,
            bytes: vec![
                255, 0, 0, 255,
                255, 0, 0, 255,
                255, 0, 0, 255,
                0, 255, 0, 255,
                0, 255, 0, 255,
                0, 255, 0, 255,
                0, 0, 255, 255,
                0, 0, 255, 255,
                0, 0, 255, 255,
            ],
        }; 
    
        let img = scale(&img, 20.0);
    
        MyWindow {
            canvas: RgbaImage::new(250, 250),
            images: vec![img],
            xval: 0,
        }
    }
}

impl WindowController for MyWindow {

    fn title(&self) -> &str {
        "MyWindow"
    }

    fn dimensions(&self) -> (u32, u32) {
        (self.canvas.height, self.canvas.width)
    }

    fn next_frame(&mut self) -> &RgbaImage {
        self.canvas.fill((0,0,0,255));

        for i in 0..self.images.len() {
            let image = &self.images[i];
            self.canvas.draw(&image, 0, self.xval);
        }

        self.xval += 1;
        &self.canvas
    }


}

fn main() {
    let my_window = MyWindow::new();
    Window::open(my_window);
}
