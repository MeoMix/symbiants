use bevy::prelude::*;
use console_error_panic_hook::set_once as set_panic_hook;
use wasm_bindgen::prelude::*;

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

    hello_world::main(&mut app);
    breakout::main(&mut app);

    app.run();
}
