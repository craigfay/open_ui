
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

impl Snake {
    pub fn is_eating_self(&self) -> bool {
        let head = self.segments.first().unwrap();
        let length = self.segments.len();
        if length == 1 { return false }

        for i in 0..length {
            let segment = &self.segments[i];
            if head.x == segment.x && head.y == segment.y {
                return true
            }
        }

        false
    }
}

// A piece of the snake
struct Segment {
    x: i32,
    y: i32,
}

// The directions that the snake can move
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

// The data that the application will store in memory
pub struct SnakeGame {
    canvas: RgbaImage,
    snake: Snake,
    frame_count: u64,
}

impl SnakeGame {
    // A method we can use to initialize the application's data
    pub fn new() -> SnakeGame {

        // Defining the dimensions of an image that we will draw pixels onto
        // and use to render each frame
        let canvas = RgbaImage::new(24, 24);

        // Defining the initial state of the snake
        let snake = Snake {
            direction: Direction::Down,
            segments: vec![Segment {
                x: 0,
                y: 0,
            }],
        };
    
        SnakeGame {
            frame_count: 0,
            canvas,
            snake,
        }
    }

    // A method that we'll use to store our "game logic". This will decide
    // how the game data changes from frame to frame.
    pub fn calculate_changes(&mut self) {

        self.frame_count += 1;

        let did_eat = false;

        // Only applying changes once every 10 frames, so the game doesn't move
        // to quickly for the player to respond. A similar effect could be
        // achieved by using floating point numbers for `x` and
        // `y`, or just lowering the framerate.
        if self.frame_count % 5 == 0 {
            let head = self.snake.segments.first().unwrap();

            // Determining the new position of the head
            let (next_x, next_y)= match self.snake.direction {
                Direction::Up => (head.x, head.y - 1),
                Direction::Down => (head.x, head.y + 1),
                Direction::Right => (head.x + 1, head.y),
                Direction::Left => (head.x - 1, head.y),
            };

            if did_eat {
                
            }

            else {
                let new_head = Segment {
                    x: next_x,
                    y: next_y,
                };

                // Adding the new head in the proper direction
                self.snake.segments.insert(0, new_head);

                // Cutting the tail to compensate
                self.snake.segments.pop();
            }

        }

    }
}


impl UIController for SnakeGame {
    fn title(&self) -> &str {
        "Snake Game"
    }

    fn dimensions(&self) -> (u32, u32) {
        (self.canvas.height * 20, self.canvas.width * 20)
    }

    fn frames_per_second(&self) -> u32 {
        60
    }

    // A function that will use a player's inputs to affect application data.
    // This will be executed at the beginning of each frame.
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

        // Applying game logic
        self.calculate_changes();
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
                segment.x,
                segment.y,
            );
        }

        &self.canvas
    }
}

fn main() {
    let application = SnakeGame::new();
    UI::launch(application);
}
