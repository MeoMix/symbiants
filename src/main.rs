use symbiants_lib::SymbiantsPlugin;
use bevy::app::App;

fn main() {
    console_error_panic_hook::set_once();

    App::new().add_plugins(SymbiantsPlugin).run();
}
