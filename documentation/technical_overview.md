# Technical Overview

This document serves to give a high-level overview of existing features. It won't explain the specifics of code, but should illuminate why code exists. It won't elaborate on future features except where necessary to explain what currently exists. This document is not about game mechanics or worldbuilding. It's to save you, the reader, from needing to piece together an understanding of how this application is built by reading through the codebase.

# Brief Summary

The application is separated into three major crates: 

* UI
* Simulation
* Rendering

The separation is made at crate-level because, in the future, Simulation might run server-side and UI might be implemented using React. UI depends on Simulation and Rendering, Rendering depends on Simulation, and Simulation is independent.

* UI is very low priority. It's all implemented using [egui](https://www.egui.rs/) because Bevy's UI tools are still very rough to use. Bevy's UI is exceptionally verbose. In the future, it's likely egui will be dropped in favor of Bevy, but it's unclear when that will occur. In theory, because the primary goal is a WASM-based app, it would also be possible to adopt React UI layer. This would deprioritize native support and it's nice to have UI for debugging, etc.

* Simulation, as the name implies, is the simulated world. It can mostly run headless, but requires the user to start the simulation when creating a new game. It's not possible to just load the Simulation crate and run it without also having a mechanism for providing that input.

* Rendering dictates how Bevy draws the simulated world onto the screen. This requires a Camera to view where UI does not.

# UI 

There are two main sections of UI:

* Main Menu
* Simulation

The main menu UI is shown to the user on first load, but only if there isn't an existing save. It is also possible to get back to the Main Menu, without refreshing the page, by clicking the "Reset Simulation" shown within the app. In the future, the main menu will allow the user to choose between "Sandbox Mode" and "Story Mode," but currently only supports "Sandbox Mode."

The simulation UI is shown to the user once the simulation is up and running. The UI is comprised of a few menus and dialogs:

* Dialogs
    * Loading
    * Breathwork
    * Simulation Over

## Loading Dialog

Loading Dialog is sometimes shown when returning to an in-progress simulation. The simulation runs client-side and attempts to reflect real-world time. So, if the app has been closed for a while, the in-app time is de-synced from real-world time and the app must fast-forward. The loading dialog is shown to the user while this is occurring.

## Breathwork Dialog

Breathwork Dialog is a prototype of encouraging the user to perform a Box Breathing routine. If the user engages with this dialog for a sufficient amount of time then they will receive food which they can add to their simulation. The current implementation isn't very compelling, and this entire feature will likely be replaced in the future, but it serves as a means of conveying some intent behind the application.

## Simulation Over Dialog

Simulation Over Dialog is shown if the queen ant dies. This can only occur if she starves to death or if the user manually kills her through.

* Menus
    * Settings
    * Action
    * Selection

## Settings Menu

Settings Menu displays user-configurable settings such as tick rate, ant color, and time of day contorls. The settings menu is shown only in sandbox mode and mostly allows developers to apply unrestricted modifications to the simulation. Notably, the simulation tick rate is not dynamic when in Story Mode and settings like ant color are only configured once per simulation run. The time controls allow the user to configure the sun in the simulation to match where they live. The time controls are known to be awkward and unwieldly and should be revisited in the future.

## Action Menu

Action Menu allows the user to influence the world with their mouse/touch. The default action is Selection, which selects a tile and reveals the Selection Menu. Other actions modify the simulation in various ways. Similar to the Settings Menu, the Action Menu only exists in Sandbox Mode as its purpose is mostly for rapid experimentation. Selection will be supported in Story Mode, but other simulation-altering actions will have dedicated UI created.

## Selection Menu

Selection Menu is shown whenever a tile or unit is selected. It shows various data about the entity that is selected.

# Simulation

Simulation is divided into three or four main concepts, depending on how you think of things:

* Common Utility
* Common Zone
* Crater Zone
* Nest Zone

Common Utility relates to the simulation, but not a specific Zone, Common Zone are systems which apply to both Crater and Nest zones, and Crater Zone/Nest Zone are systems which apply only to their specific zones.

* Common Utility
    * Story Time
    * Settings
    * External Event
    * App State
    * Save

## Common Utility

### Story Time

Story Time is focused on tracking the elapsed ticks of the simulation and how those ticks relate to elapsed time. The goal is to have a simulation whose time mirrors real-world time by relying on Bevy's FixedUpdate schedule to advance the simulation one tick per increment of real-world time. It is possible for simulation time to desync from real-world time in a few scenarios: the user closes the app, the browser tab loses focus, the app is lagging due to low PC performance, or the user changes the tick rate.

When the user closes the app, the current time is recorded into the save file. When the user opens the app, the delta time missed is computed, translated into a number of missed simulation ticks, and the simulation begins fast-forwarding. While fast-forwarding, the simulation runs at a significantly increased tick rate and does not render every simulation tick. It runs in this mode until it its time is resynced. This process also occurs when the browser tab loses focus because the app pauses when not visible.

There isn't a great recovery mechanism if the app is not able to run sufficiently quick due to PC performance issues, but it should be possible to catch-up in some scenarios by skipping rendering and fast-forwarding. The simulation will stutter, though.

The user is able to change the tick rate of the simulation when in Sandbox Mode. Changing the tick rate will cause the simulation time to desync. This is why there is a "Use Real Time" checkbox in Sandbox Mode. The user is allowed to sync the initial time of the simulation with real-world time, but this will diverge, even if "Use Real Time" is checked, if the tick rate is adjusted. This UX is poor and should be revisited.

Story Time also tracks some cursory details about real-world sunrise/sunset. This is to influence the sky/lighting of the simulation to make it feel more immersive. The intent here is to have the simulation's sunrise/sunset synced with the user's real-world sunrise/sunset. This seemed important because the goal of the app is to establish a daily check-in ritual, which invites the user to do so as they wake/sleep, and it would feel weird if the simulated world was midday when the user is waking up.

### Settings

Settings is a slew of constants which influence the world. Some of these are just defaults that can be overriden by the user, such as ant color, and others are hardcoded and not exposed to the user, such as grid size. There is also a set of probabilities which are used to provide dynamic flair in the world. It's worth taking a moment to scan through the list of settings and familiarize yourself with them.

### External Event

External Event is an abstraction of a user's input device. This is desirable to prevent Simulation from needing awareness of the Rendering crate. The user is able to spawn/despawn ants, influence ant state, and spawn/despawn elements. It should be possible to spawn/despawn pheromones, too, but this isn't implemented yet.

It's not expected that External Event will be used in Story Mode. It's mostly used for rapid iteration when developing the app and to entertain curious users exploring Sandbox Mode.

### App State

App State represents the states the app goes through which require dedicated UI communication to the user. This rule dictates the granularity of the states. Currently, App State encompasses both Rendering and Simulation state, but it should be split into two states so that Simulation isn't aware of "MainMenu" state.

### Save

Saving occurs automatically and periodically. Saving hasn't been implemented for native apps, but it shouldn't be hard to support, and there have been some technical concessions made with the web implementation as it is more difficult to fully support.

Ideally, the simulation would save whenever the user closes the tab. This is difficult to support because there isn't a clear path to querying the world state, synchronously, from the JavaScript event loop. It's possible to do it asynchronously, but then JavaScript's `onbeforeunload` will have passed and the tab may have closed. The problem is that world state is only accessible from within Bevy systems and those systems are ran by Bevy's scheduler. They cannot be manually executed from JavaScript.

As such, a workaround has been implemented. Periodically, every few seconds, a snapshot of the world is taken and written into a global variable. This global is trivial to access from JavaScript, but is expensive to create and will always contain slightly stale information. When the browser tab closes, the snapshot is serialized and written to local storage.

The current save algorithm is very naive. The entire world is persisted, which is many MBs of text data, even if no changes have been applied to the world. It would be better to persist the delta of the initial seed and the current state. All of the model data (i.e. everything in the Simulation crate) is persisted as well as Settings and Story Time.

## Common Simulation

Common Simulation contains features which apply to both Crater and Nest zones. This includes the Grid which each Zone relies upon as well as Ant, Element, and Pheromone features.

Custom commands are used to manage Ant, Element, and Pheromone entities. Custom commands run in what is called an "Exclusive System" which is a Bevy construct. Exclusive systems are slow because they receive mutable access to the entire world and, as such, cannot be parallelized. Custom commands are used here because it makes the code easier to reason about. The caches which track which entities exist at which positions are able to stay synchronized with the world's `QueryState` because the world is modified, and the cache updated, in a single operation. In the future, these custom commands may be identified as a key performance bottleneck and be removed. If this occurs, code will need to be rewritten on the assumption the cache may be stale and desynced from executed queries.

### Grid

Grid is a simple, 2D representation of a set of square tiles. It's not guaranteed that both zones have the same size grid, but they do currently. Each tile has a size of 1 unit and its location is able to be referenced via `Position`. For simulation logic, the x-axis increases to the right and the y-axis increases towards the bottom. Discovering which `Entity` is at a given `Position` is a common need and so there are caches throughout the app which map `Position` to `Entity` outside of the traditional ECS architecture. If pure ECS were relied upon then it would cost O(n) to discover an entity at a given position. In the future, it might be desirable to adopt a kd-tree or other structure, but performance bottlenecks haven't appeared in this way just yet.

### Ant

Ants are able to roam between zones and thus have behaviors which apply in both zones: 

* initative
* hunger
* digestion
* death

Initative tracks whether an ant has moved and/or acted recently. It ensures an ant doesn't move more than one tile per simulation tick or take multiple actions. Initiative runs on a timer which takes a few ticks to replenish and without the timer ants would necessarily move as quickly as elements fall through the sky - which feels unnatural. One drawback of this approach, though, is that the simulation needs to advance many ticks to move ants a small amount and this draws out the delay when fast-forwarding.

Hunger and Digestion are closely related. Ants get hungry and must eat food to survive. There's no penalty for getting hungry - just instant death from starvation. Ants won't eat food if they're not hungry, or if they are hungry but have food that they're still digesting. Ants can engage in trophallaxis thereby feeding adjacent, hungry ants. It seemed important to introduce the concept of digestion because, without it, the queen was able to give birth to an ant, it wasn't born starving, and thus it could perform trophallaxis on the queen. This resulted in an infinite food glitch which was undesirable. It seems possible, and desirable, to eliminate the concept of digestion by introducing an egg/larvae/pupae lifecycle. Digestion isn't an especially compelling game mechanic and isn't something the player would necessarily care to have simulated.

Death occurs to ants who starve. It's possible there should be more scenarios which kill ants, such as old age or being crushed by debris, but nothing has been implemented. Alternatively, perhaps death should be removed because death isn't a great concept for a mental health app. Still, if a colony is to grow to hundreds of ants, it seems reasonable to focus on the status of the colony, rather than individual ants, in which case having ants die is less of an issue.

### Element

Elements exist at every tile of the simulation as well as potentially being held in an Ant's inventory.  There are four elements, but more will be introduced: 

* Air
* Dirt
* Sand
* Food

It's important to consider the performance implications of writing queries which work against elements due to how many exist. The code makes heavy use of "marker components," which are types known at compile time, to enable queries to be written efficiently. This is why there is both an `Element` enum and `Air`, `Dirt`, `Sand`, and `Food` marker components.

Air represents the absence of another element. It's possible that it would be better to represent this concept with `None`, but it's nice to be able to rely on densely populated grids.

Dirt is only represented in the Nest. It is undisturbed ground. If it shares an edge with another solid material, and is stable, then gravity does not pull it downward.

Sand is only represented in the Nest. It is disturbed/loose ground and is generated when an ant digs dirt. Sand, when unstable, falls when subjected to gravity and may fall straight down or diagonally. Ants will intentionally remove sand from underground and drop it on the surface and this behavior creates an ant hill.

Food is represented in both the Crater and Nest. It's similar to Sand in that it falls when unstable, but it can also be eaten by ants. Ants will store food underground and will attempt to group food near other food.

### Pheromone

Pheromones sparsely populate the grid, but there may be multiple, distinct pheromones at a given position. The implementation of crater pheromones and Nest pheromones function similarly, but not identically. There are two crater pheromones, `Food` and `Nest`, two nest pheromones, `Tunnel` and `Chamber`.

Pheromones are applied with a variable amount of `PheromoneStrength` and this strength decays over time. When strength reaches zero the pheromone despawns.

## Nest Simulation

## Crater Simulation

# Rendering

## Common Rendering

## Nest Rendering

## Simulation Rendering
