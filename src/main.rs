
use winlib::{RgbaImage, Window, WindowController};

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
    
        let img = RgbaImage::nearest_neighbor_scale(&img, 20.0);
    
        MyWindow {
            canvas: RgbaImage::new(250, 250),
            images: vec![img],
            xval: 0,
        }
    }
}

impl WindowController for MyWindow {
    fn title(&self) -> &str {
        "My Window"
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
    let my_window = MyWindow::new();
    Window::open(my_window);
}
