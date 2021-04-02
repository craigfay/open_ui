# OpenUI
A user friendly Rust library for creating cross-platform GUI apps easily. Built on OpenGL.

# Usage
To create a Rust program that renders a UI, simply define a struct that implements the `UIController` interface.

```rust
use open_ui::UIController;

struct SnakeGame {};

impl UIController for SnakeGame {
  // Implement the functions associated with UIController here
}
```

Then pass an instance of that struct into `UI::launch()`.

```rust
use open_ui::UI;

fn main() {
    let application = SnakeGame::new();
    UI::launch(application);
}
```
