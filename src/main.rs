
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

    // Attempt to change the snake's direction. Reversing is disallowed
    pub fn change_direction(&mut self, direction: Direction) {
        if direction == Direction::Up && self.direction == Direction::Down { return }
        if direction == Direction::Down && self.direction == Direction::Up { return }
        if direction == Direction::Right && self.direction == Direction::Left { return }
        if direction == Direction::Left && self.direction == Direction::Right { return }
        self.direction = direction;
    }
}

// A piece of the snake
struct Segment {
    x: i32,
    y: i32,
}

// The directions that the snake can move
#[derive(PartialEq, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

// A piece of food for the snake to eat
struct Food {
    x: i32,
    y: i32,
}

// An rudimentary pseudo-random number generator
pub struct PseudoRandomness {
    seed: std::time::Instant,
}
impl PseudoRandomness {
    // Create a new instance, "seeded" with the current time
    pub fn new() -> PseudoRandomness {
        PseudoRandomness { seed: std::time::Instant::now() }
    }
    // Generate a seemingly random i32 that's >= min and < max
    pub fn integer_between(&self, min: i32, max: i32) -> i32 {
        let now = std::time::Instant::now();
        let large_number = now.duration_since(self.seed).as_nanos() as i32;
        min + (large_number.abs() % (max - min))
    }
}

// The data that the application will store in memory
pub struct SnakeGame {
    canvas: RgbaImage,
    snake: Snake,
    food: Food,
    frame_count: u64,
    rng: PseudoRandomness,
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
            segments: vec![
                Segment { x: 0, y: 1 },
                Segment { x: 0, y: 0 },
            ],
        };

        let rng = PseudoRandomness::new();

        let food = Food {
            x: rng.integer_between(0, (canvas.width - 1) as i32),
            y: rng.integer_between(0, (canvas.height - 1) as i32),
        };
    
        SnakeGame {
            frame_count: 0,
            canvas,
            snake,
            food,
            rng,
        }
    }

    // Check to see if any segment of the snake is touching the food
    pub fn snake_body_touches_food(&self) -> bool {
        for segment in &self.snake.segments {
            if segment.x == self.food.x && segment.y == self.food.y {
                return true;
            }
        }
        false
    }

    // Check to see if the first segment of the snake is touching the food
    pub fn snake_head_touches_food(&self) -> bool {
        let head = self.snake.segments.first().unwrap();
        head.x == self.food.x && head.y == self.food.y
    }

    // Place food in a new random spot
    pub fn replace_food(&mut self) {
        self.food = Food {
            x: self.rng.integer_between(0, (self.canvas.width - 1) as i32),
            y: self.rng.integer_between(0, (self.canvas.height - 1) as i32),
        };
    }



    // A method that we'll use to store our "game logic". This will decide
    // how the game data changes from frame to frame.
    pub fn calculate_changes(&mut self) {

        self.frame_count += 1;

        // Only applying changes once every 10 frames, so the game doesn't move
        // to quickly for the player to respond. A similar effect could be
        // achieved by using floating point numbers for `x` and
        // `y`, or just lowering the framerate.
        if self.frame_count % 4 == 0 {
            let head = self.snake.segments.first().unwrap();

            // Determining the new position of the head
            let (next_x, next_y)= match self.snake.direction {
                Direction::Up => (head.x, head.y - 1),
                Direction::Down => (head.x, head.y + 1),
                Direction::Right => (head.x + 1, head.y),
                Direction::Left => (head.x - 1, head.y),
            };

            // Adding the new head in the proper direction
            self.snake.segments.insert(0, Segment { x: next_x, y: next_y });

            // Replacing the food when it touches the snake's head
            if self.snake_head_touches_food() {
                self.replace_food();

                // Making sure that we haven't placed the food on the snake
                while self.snake_body_touches_food() {
                    self.replace_food();
                }
            }

            // Cutting the tail to create the illusion of motion, unless the
            // snake is supposed to get longer because it just ate food
            else {
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
                        self.snake.change_direction(Direction::Up);
                    }
                    if event.key == Down && event.action == Press {
                        self.snake.change_direction(Direction::Down);
                    }
                    if event.key == Right && event.action == Press {
                        self.snake.change_direction(Direction::Right);
                    }
                    if event.key == Left && event.action == Press {
                        self.snake.change_direction(Direction::Left);
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
        let mut segment_image = RgbaImage::new(1, 1);
        segment_image.fill((255,255,255,255));

        let mut food_image = RgbaImage::new(1, 1);
        food_image.fill((255,255,0,255));

        self.canvas.draw(
            &food_image,
            self.food.x,
            self.food.y,
        );

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
