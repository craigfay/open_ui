
use open_gui::{GUI, GUIController, RgbaImage};

pub struct MyApplication {
    canvas: RgbaImage,
    xval: i32,
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
    
        let img = RgbaImage::nearest_neighbor_scale(&img, 20.0);
    
        MyApplication {
            canvas: RgbaImage::new(250, 250),
            images: vec![img],
            xval: 0,
        }
    }
}

impl GUIController for MyApplication {
    fn title(&self) -> &str {
        "My Application"
    }

    fn dimensions(&self) -> (u32, u32) {
        (self.canvas.height * 2, self.canvas.width * 2)
    }

    fn frames_per_second(&self) -> u32 {
        60
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
    let application = MyApplication::new();
    GUI::launch(application);
}
