use bevy::prelude::*;
use console_error_panic_hook::set_once as set_panic_hook;
use wasm_bindgen::prelude::*;

mod antfarm;
mod breakout;
mod hello_world;

#[wasm_bindgen(
    inline_js = "export function snippetTest() { console.log('Hello from JS FFI 1!'); }"
)]
extern "C" {
    fn snippetTest();
}

fn main() {
    set_panic_hook();
    snippetTest();

    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        window: WindowDescriptor {
            height: 200.0,
            width: 400.0,
            ..default()
        },
        ..default()
    }));

    // hello_world::main(&mut app);
    // breakout::main(&mut app);

    antfarm::main(&mut app);

    app.run();
}
