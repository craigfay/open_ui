# OpenUI
A user friendly Rust library for creating high-performance cross-platform GUI apps. This is a great choice for anyone who doesn't want to work directly with [OpenGL](https://www.khronos.org/opengl/wiki/OpenGL_Shading_Language), but also doesn't want a whole desktop web framework like [Electron](https://github.com/electron/electron) or [Tauri](https://github.com/tauri-apps/tauri). It's also a great tool for building 2D games in Rust!

# Installation

Add OpenUI as a dependency in any Cargo project:

```
# Cargo.toml
[dependencies]
open_ui = { git = "https://github.com/craigfay/open_ui", ref = "1.1.0" }
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