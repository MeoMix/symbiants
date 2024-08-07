# Symbiants

![MIT/Apache 2.0](https://img.shields.io/github/license/MeoMix/symbiants)
![Discord](https://img.shields.io/discord/1047934512773996604)

![image](https://github.com/MeoMix/symbiants/assets/1380995/394ac75d-6695-4492-8a99-46539bc91f40)


What is this? A project for ANTS?

Yup!

Symbiants is a slow-moving, real-time simulation of an ant colony. It's a homage to SimAnt, Tamagotchi, Progress Quest, and RimWorld. It's also mental health software.

Wait, what? Mental health software? 

Yeah! You heard me. You know how keeping a dog for a pet encourages you to go for daily walks? Yeah, it's like that, but for your brain... and with ants.

Anyway.

Symbiants is written in Rust using the amazing Bevy game engine. Its build target is WASM and runs in the browser. Desktop has first-class support, but mobile is a little rougher. iOS Safari/Chrome is supported. Android Firefox works, but Android Chrome is broken due to a downstream bug. This should be resolved in Bevy v0.13 (Q1 '24)

The project is in its infancy, but has lofty aspirations. Ultimately, there are two goals:

  1) Create a compelling ant simulation in which the user acts as a nurturing caretaker. The colony grows over the course of real-world months and slowly takes over a planet. The colony will grow from a single queen to a full-fledged colony with thousands of worker ants digging tunnels, harvesting food, laying pheromone trails, etc.

  2) Create tactful mental health software that encourages users to spend a minute meditating and journaling each day. The user will need to engage with these tasks to gain access to the sustenance their ants need. They must take care of themselves if they want to take care of their ants.

There's a whole bunch of futuristic sci-fi lore to support these ideas. The ants are on an alien world, the user is in a satellite orbiting the planet, and the two establish a symbiotic relationship, out of necessity, as a means of terraforming the planet. You can read more about that here - https://docs.google.com/document/d/17ACH1XLCn7hkKz2dhuL1c_nxbGOTZdUY6jJPsY_xA6I/ but, fair warning, the story got quite far ahead of the code and so there's a bit of a gap between the two. All efforts are currently focused on creating a very crude, ugly, and direct MVP without supporting story or deep gameplay mechanics.

# Development

This project expects to be developed within a VSCode devcontainer (Docker) with code running on WSL2 (Ubuntu). If you have that then you're good to go, otherwise you'll need to ensure your environment is configured with Rust Nightly. You can just mirror the bits declared in Dockerfile.

Release builds only generate WASM artifacts - this project intends to only be played in the browser. However, it's pretty challenging to have a good development workflow when working strictly in a WASM environment due to lack of incremental recompiles and lack of breakpoint debugging. So, the primary compilation target is Linux, but it's important to double-check the WASM target every once in a while.

Use the commands `cargo build`, `cargo run`, and `cargo watch -x run` when doing native development. `cargo watch` will watch files for changes and auto-reload the app.
Use the commands `trunk build` and `trunk serve` when doing WASM development. `trunk serve` will watch files for changes and auto-reload the app.

Currently, native development only supports x11 not Wayland and only provides hardware acceleration for Nvidia/DirectX. 

You should use Ubuntu 22 to ensure GPU acceleration works well. You'll need to install an XServer on your host machine. I use VcXsrv(https://sourceforge.net/projects/vcxsrv/). Be sure to add an exception in your firewall for communication and to start VcSrv with "Disable access control" enabled. You do not need to make any changes to the devcontainer to enable x11 forwarding. To triage, `ECHO $DISPLAY` from within the devcontainer should emit `:0` and `xclock` should open a clock on the host machine.

If you have any questions - please feel comfortable reaching out on Discord.
