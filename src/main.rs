use antfarm::AntfarmPlugin;
use bevy::app::App;

fn main() {
    console_error_panic_hook::set_once();

    App::new().add_plugins(AntfarmPlugin).run();
}
