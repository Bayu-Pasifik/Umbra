# Umbra

A Nusa-style dynamic island for the desktop, written in Rust with eframe (egui) on wgpu.

A pill-shaped notch pinned to the top-center of the screen. It shows the clock, springs open on
hover, and holds pages for music, a focus timer, a water goal, a to-do list and a file shelf.
Transient alerts (charging, headphones, volume, brightness, Caps Lock) open it briefly.

## Controls

- Hover the pill to open it; move away and it closes after 300 ms.
- Switch pages with the dots, the mouse wheel, or the left/right arrow keys.
- Drop files on the island to add them to the shelf; drag a chip out to copy the file elsewhere
  (Windows and macOS). Right-click a chip or a to-do row to remove it.
- After 10 s idle, the clock gives way to a pair of eyes.

## Build

```sh
cargo run --release
```

Requires a stable Rust toolchain. On Windows you also need the MSVC build tools. On Linux, install
the usual X11/Wayland and Vulkan runtime libraries (they are loaded at run time).

Set `UMBRA_DEMO_ALERTS=1` to cycle through sample alerts every 8 seconds.
Set `UMBRA_DEBUG_FRAMES=1` to log every frame, which helps verify that idle means zero redraws.

## Package

`.github/workflows/build.yml` checks formatting, lints and tests, then builds release binaries for
Linux, Windows and macOS and uploads each as a workflow artifact. Locally:

```sh
cargo build --release
# binary: target/release/umbra (umbra.exe on Windows)
```

The release profile is size-optimized (LTO, `opt-level = "z"`, stripped); the Windows binary is
about 4.8 MB.

## Data

State (to-do items, water count, shelf files) is stored as JSON at
`<config dir>/umbra/data.json`. Nothing leaves the machine.

## Known limitations

- Media, battery and Bluetooth sources are mocks behind the `ContentSource` and `EventSource`
  traits. Real OS backends are future work.
- Fullscreen detection is implemented for Windows only. When another app is fullscreen the
  window is parked off-screen rather than hidden, so that app never loses focus.
- Drag-out is not available on Linux: the `drag` crate only supports GTK windows.
- On Wayland, compositors may ignore the requested window position; layer-shell would be needed
  for exact top-center placement.
- Memory use is dominated by the GPU driver (about 250 MB working set with Vulkan on the test
  machine), well above the 40 MB goal.
