use antfarm::AntfarmPlugin;
use bevy::prelude::*;

mod antfarm;

fn main() {
    console_error_panic_hook::set_once();

    App::new().add_plugin(AntfarmPlugin).run();
}
