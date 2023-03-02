use bevy::prelude::*;

mod antfarm;

fn main() {
    console_error_panic_hook::set_once();

    let mut app = App::new();

    antfarm::main(&mut app);

    app.run();
}
