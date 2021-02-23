
use open_ui::{
    UI,
    UIController,
    RgbaImage,
    UIEvent,
    KeyboardKey::*,
    KeyboardAction::*,
};

pub struct MyApplication {
    canvas: RgbaImage,
    images: Vec<RgbaImage>,
    position: (f32, f32),
    momentum: (f32, f32),
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
    
        MyApplication {
            canvas: RgbaImage::new(24, 24),
            images: vec![img],
            position: (0.0, 0.0),
            momentum: (0.0, 0.0),
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
        for &event in events {
            match event {
                UIEvent::Keyboard(event) => {
                    if event.key == Up && event.action == Press {
                        self.momentum.1 = -0.2;
                        self.momentum.0 = 0.0;
                    }
                    if event.key == Down && event.action == Press {
                        self.momentum.1 = 0.2;
                        self.momentum.0 = 0.0;
                    }
                    if event.key == Right && event.action == Press {
                        self.momentum.0 = 0.2;
                        self.momentum.1 = 0.0;
                    }
                    if event.key == Left && event.action == Press {
                        self.momentum.0 = -0.2;
                        self.momentum.1 = 0.0;
                    }
                },
                _ => {},
            }
        }

        // if events.len() > 0 {
            // println!("{:?}", events);
        // }
    }

    fn next_frame(&mut self) -> &RgbaImage {
        self.canvas.fill((0,0,0,255));

        for i in 0..self.images.len() {
            self.canvas.draw(
                &self.images[i],
                self.position.0 as i32,
                self.position.1 as i32,
            );
        }

        self.position.0 += self.momentum.0;
        self.position.1 += self.momentum.1;

        &self.canvas
    }
}

fn main() {
    let application = MyApplication::new();
    UI::launch(application);
}
