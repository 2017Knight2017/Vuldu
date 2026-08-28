<center>

[![Vuldu Showcase](https://i.ytimg.com/vi/SZ2nbXC-9_A/maxresdefault.jpg)](http://www.youtube.com/watch?feature=player_embedded&v=SZ2nbXC-9_A)

</center>

# Vuldu

**Vul[kan] du[um]** is a Doom port written by me
completely from scratch in Rust, with Vulkan 
rendering and the ECS pattern in the game loop. The 
project's ultimate goal is to achieve maximum 
performance on large maps through multithreaded 
computations, which was fundamentally impossible in the 
original *Id Tech 1*.

## How to Run
*Note: Vuldu requires a GPU with Vulkan 1.3 support.*

First, download the latest
[release](https://github.com/2017Knight2017/Vuldu/releases),
as well as the wad you want to run (for example,
DOOM2.WAD). To run it, open the command line in the
game folder and enter:

```bash
./vuldu -i your/path/to/DOOM2.WAD 
```

You can view the application's parameters
using `./vuldu -h`

## Technology Stack
- *[winit](https://github.com/rust-windowing/winit)* 
=> Window management
- *[hecs](https://github.com/Ralith/hecs)* 
=> Implementation of the ECS world and entities
- *[rodio](https://github.com/rustaudio/rodio)* 
=> Spatial audio
- *[serde](https://github.com/serde-rs/serde)* 
=> Parsing of toml tables
- *[cxx](https://github.com/dtolnay/cxx)* 
=> FFI bridge between Rust and the C++ renderer
- *[earcut](https://github.com/georust/earcut)* algorithm 
=> Floor triangulation
---
- *[rayon](https://github.com/rayon-rs/rayon)* 
and 
*[micropool](https://github.com/DouglasDwyer/micropool)* 
=> Parallelization
of computations

It is important to note that I deliberately chose 
two libraries for multithreading, as they work 
differently and solve opposite problems:

*Rayon* is optimized for maximum performance.
It achieves this through **work stealing**: if
a thread finishes its work, it takes parts of the 
work from another thread. It works perfectly
for preparing data during loading.

However, practice shows that "work stealing"
is unstable when the game's framerate depends on it.
Therefore, I chose *micropool*, which works on the
principle of **busy waiting**: after finishing
their work, threads do not go to sleep, but wait so 
that they can start executing the next task as quickly
as possible. It works perfectly when more data needs 
to be processed without significantly increasing 
the load.

## Project Architecture
The project is designed so that the crates are loosely coupled with each other in a *strictly one-way order*:

<center><b> ↓ Renderer ↓ </b></center>
<center><b> > App < </b></center>
<center><b> ↑ Engine ↑ </b></center>
<center><b> ↑ Wad Parser ↑ </b></center>

### Renderer
The FFI bridge leads to `renderer_cpp/`, where the Vulkan
class is implemented in C++ and safely abstracted in
`src/lib.rs` for subsequent use of its methods in
Rust. The graphics pipeline is designed with
**mass resource processing** and minimization of
draw calls in mind.

Objects and UI make extensive use of **Instancing**,
allowing tens of thousands of instances to be displayed
on screen with almost no impact on FPS.

### Wad Parser
In this crate, bytes from the wad are converted into 
textures and levels. All file resource management is 
handled through the `WadManager` object, which stores 
lumps associated with their source file.

In `src/textures/`, textures from the patch format
are converted into an array of PLAYPAL color indices. 
In `src/vertices/`, sectors and the linedefs surrounding 
them are triangulated into floors and walls and turned 
into a fully-fledged 3D scene.

### Engine
This is where all ECS systems related to movement,
vision, sound propagation, etc. live. *Engine* can
modify level and entity data during gameplay.

There is more code inspired by John Carmack's 
[original source code](https://github.com/id-Software/DOOM/tree/master/linuxdoom-1.10) 
here than anywhere else in the project.

### App
The main coordination center of the project. In
`App::resumed()` you can find the window initialization 
code, while `WindowEvent::RedrawRequested` contains the 
code for redrawing it.

Overall, in *App*, all other crates are merged together
in the form of `GraphicsContext` and `GameContext`, while 
also handling audio and user input.

## Building
For a local build, you need the
[Vulkan SDK]([https://vulkan.lunarg.com/sdk/home](https://vulkan.lunarg.com/sdk/home)). 
After installing it, clone the repository to your 
computer:

```bash
cd your/local/path
git clone https://github.com/2017Knight2017/Vuldu.git
```

Place the wad you plan to run (for example, DOOM2.WAD) 
there as well. After that, compile and run the project 
using cargo:

```bash
cargo build
cargo run -- -i DOOM2.WAD
```

## License
Vuldu is licensed under 
**GNU General Public License 3.0**.
