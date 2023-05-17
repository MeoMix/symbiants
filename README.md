# Symbiants

![image](https://github.com/MeoMix/symbiants/assets/1380995/dda710c1-5aba-4d47-b1ea-685560aff254)

What is this? A project for ANTS?

Yup!

Symbiants is a slow-moving, real-time simulation of an ant colony. It's a homage to SimAnt, Tamagotchi, Progress Quest, and RimWorld. It's also mental health software.

Wait, what? Mental health software? 

Yeah! You heard me. You know how keeping a dog for a pet encourages you to go for daily walks? Yeah, it's like that, but for your brain... and with ants.

Anyway.

Symbiants is written in Rust using the amazing Bevy game engine. Its build target is WASM and runs in the browser. Desktop has first-class support but it would clearly be good to support mobile. The WASM story for mobile devices still needs to improve a bit, though, unfortunately.

The project is in its infancy, but has lofty aspirations. Ultimately, there are two goals:

  1) Create a compelling ant simulation in which the user acts as a nurturing caretaker. The colony grows over the course of real-world months and slowly takes over a planet. The colony will grow from a single queen to a full-fledged colony with thousands of worker ants digging tunnels, harvesting food, laying pheromone trails, etc.

  2) Create tactful mental health software that encourages users to spend a minute meditating and journaling each day. The user will need to engage with these tasks to gain access to the sustenance their ants need. They must take care of themselves if they want to take care of their ants.

There's a whole bunch of futuristic sci-fi lore to support these ideas. The ants are on an alien world, the user is in a satellite orbiting the planet, and the two establish a symbiotic relationship, out of necessity, as a means of terraforming the planet. You can read more about that here - https://docs.google.com/document/d/17ACH1XLCn7hkKz2dhuL1c_nxbGOTZdUY6jJPsY_xA6I/ but, fair warning, the story got quite far ahead of the code and so there's a bit of a gap between the two. All efforts are currently focused on creating a very crude, ugly, and direct MVP without supporting story or deep gameplay mechanics.

