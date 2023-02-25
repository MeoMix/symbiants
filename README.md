This is a simple playground for learning Rust.

Currently has a devcontainer which gets a nightly Rust build going and then uses Trunk to serve and watch for changes.

Rust files are Bevy's hello world using console log instead of println. Output is WASM.

Note that devcontainer should be mounted from a Linux container for watch to work properly. I was only able to get it to work with polling when mounting devcontainer files from Windows host.
