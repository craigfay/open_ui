
use open_ui::{UI, UIController, RgbaImage, UIEvent};

pub struct MyApplication {
    canvas: RgbaImage,
    xval: f32,
    images: Vec<RgbaImage>,
}

impl MyApplication {
    pub fn new() -> MyApplication {
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
    
        // let img = RgbaImage::nearest_neighbor_scale(&img, 20.0);
    
        MyApplication {
            canvas: RgbaImage::new(24, 24),
            images: vec![img],
            xval: 0.0,
        }
    }
}

impl UIController for MyApplication {
    fn title(&self) -> &str {
        "My Application"
    }

    fn dimensions(&self) -> (u32, u32) {
        (self.canvas.height * 20, self.canvas.width * 20)
    }

    fn frames_per_second(&self) -> u32 {
        60
    }

    fn process_events(&mut self, events: &Vec<UIEvent>) {
        if events.len() > 0 {
            println!("{:?}", events);
        }
    }

    fn next_frame(&mut self) -> &RgbaImage {
        self.canvas.fill((0,0,0,255));

        for i in 0..self.images.len() {
            let image = &self.images[i];
            self.canvas.draw(&image, 0, self.xval as i32);
        }

        self.xval += 0.01;
        &self.canvas
    }
}

fn main() {
    let application = MyApplication::new();
    UI::launch(application);
}
