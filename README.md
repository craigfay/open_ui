# OpenUI
A user friendly Rust library for creating cross-platform GUI apps easily. Built on OpenGL.

# Installation

Add OpenUI as a dependency in any Cargo project:

```
# Cargo.toml
[dependencies]
open_ui = { git="https://github.com/craigfay/open_ui", branch="main", ref="fb638f9" }
```

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

# Examples
For an example of how to use OpenUI, see `src/main.rs`.