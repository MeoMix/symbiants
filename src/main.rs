use bevy::prelude::*;
use console_error_panic_hook::set_once as set_panic_hook;

mod antfarm;

fn main() {
    set_panic_hook();

    let mut app = App::new();

    antfarm::main(&mut app);

    app.run();
}
