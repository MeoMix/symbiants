use symbiants_lib::SymbiantsPlugin;
use bevy::app::App;

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new().add_plugins(SymbiantsPlugin).run();
}
