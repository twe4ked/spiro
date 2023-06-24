use bevy::prelude::*;

use spiro::LibPlugin;

fn main() -> AppExit {
    App::new().add_plugins(LibPlugin).run()
}
