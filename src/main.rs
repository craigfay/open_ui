
use open_ui::{
    UI,
    UIController,
    RgbaImage,
    UIEvent,
    KeyboardKey::*,
    KeyboardAction::*,
};


// The character that the player controls
pub struct Snake {
    segments: Vec<Segment>,
    direction: Direction,
}

// A piece of the snake
struct Segment {
    x_position: i32,
    y_position: i32,
}

// The directions that the snake can move
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

// The data that the application will store in memory
pub struct SnakeApplication {
    canvas: RgbaImage,
    snake: Snake,
}

impl SnakeApplication {
    // A method we can use to initialize the application's data
    pub fn new() -> SnakeApplication {

        // Defining the dimensions of an image that we will draw pixels onto
        // and use to render each frame
        let canvas = RgbaImage::new(24, 24);

        // Defining the initial state of the snake
        let snake = Snake {
            direction: Direction::Down,
            segments: vec![Segment {
                x_position: 0,
                y_position: 0,
            }],
        };
    
        SnakeApplication {
            canvas,
            snake,
        }
    }
}


impl UIController for SnakeApplication {
    fn title(&self) -> &str {
        "Snake Game"
    }

    fn dimensions(&self) -> (u32, u32) {
        (self.canvas.height * 20, self.canvas.width * 20)
    }

    fn frames_per_second(&self) -> u32 {
        60
    }

    // A function that will use a player's inputs to affect application data
    fn process_events(&mut self, events: &Vec<UIEvent>) {
        for &event in events {
            match event {
                UIEvent::Keyboard(event) => {
                    if event.key == Up && event.action == Press {
                        self.snake.direction = Direction::Up;
                    }
                    if event.key == Down && event.action == Press {
                        self.snake.direction = Direction::Down;
                    }
                    if event.key == Right && event.action == Press {
                        self.snake.direction = Direction::Right;
                    }
                    if event.key == Left && event.action == Press {
                        self.snake.direction = Direction::Left;
                    }
                },
                _ => {},
            }
        }
    }

    // A function that will use application data to decide which image to
    // render on the next frame
    fn next_frame(&mut self) -> &RgbaImage {

        // Erasing the canvas
        self.canvas.fill((0,0,0,255));

        // Defining the image that will represent a segment of the snake
        let segment_image = RgbaImage {
            width: 1,
            height: 1,
            bytes: vec![255, 255, 255, 255],
        }; 

        // Drawing each snake segment to the canvas
        for segment in &self.snake.segments {
            self.canvas.draw(
                &segment_image,
                segment.x_position,
                segment.y_position,
            );
        }

        &self.canvas
    }
}

fn main() {
    let application = SnakeApplication::new();
    UI::launch(application);
}
