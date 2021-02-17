#[macro_use]
extern crate glium;

fn main() {
    #[allow(unused_imports)]
    use glium::{glutin, Surface};

    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let mut bytes: Vec<u8> = vec![];

    for i in 0..16 {
        if i % 2 == 0 {
            bytes.push(0);
            bytes.push(255);
            bytes.push(0);
            bytes.push(255);
        }
        else {
            bytes.push(255);
            bytes.push(0);
            bytes.push(0);
            bytes.push(255);
        }
    }

    let image = glium::texture::RawImage2d::from_raw_rgba_reversed(
        &bytes,
        (4, 4)
    );

    let texture = glium::texture::Texture2d::new(&display, image).unwrap();


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

    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    let mut t: f32 = -0.5;

    let vertex_shader_src = r#"
        #version 150

        in vec2 position;
        in vec2 tex_coords;
        flat out vec2 v_tex_coords;

        uniform mat4 matrix;

        void main() {
            v_tex_coords = tex_coords;
            gl_Position = matrix * vec4(position, 0.0, 1.0);
        }
    "#;

    let fragment_shader_src = r#"
        #version 150

        flat in vec2 v_tex_coords;
        out vec4 color;
    
        uniform sampler2D tex;
    
        void main() {
            color = texture(tex, v_tex_coords);
        }
    "#;



    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

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

        t += 0.002;
        if t > 0.5 {
            t = -0.5;
        }

        let uniforms = uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [ t , 0.0, 0.0, 1.0f32],
            ],
            tex: &texture,
        };

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);

        target.draw(&vertex_buffer, &indices, &program, &uniforms,
            &Default::default()).unwrap();

        target.finish().unwrap();
    });
}