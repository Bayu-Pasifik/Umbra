# PRD: Umbra

A Nusa-style dynamic island for the desktop, written in Rust.

## Overview
Umbra is a pill-shaped notch pinned to the top-center of the desktop. It expands to show music, timers, reminders, files and system alerts, then contracts when idle. Reference: [Nusa](https://rama2555.github.io/nusa/).

## Audience
A single developer, who both builds and uses it.

## Target Platform
- Desktop Linux, Windows and macOS.
- Built on eframe (egui) over winit and wgpu.
- Borderless, transparent, always-on-top window.

## Must-Have Features
1. Collapsed pill shows the clock. It springs open on hover and closes when the pointer leaves.
2. Click-to-interact controls inside the expanded panel.
3. Content pages:
    - Music: title, artist, progress, play/pause/skip.
    - Focus timer: 25:00 Pomodoro.
    - Water reminder: 1.5 L daily goal.
    - File shelf: drag files in and out.
    - To-do list.
4. Transient alerts (charging, headphones, volume/brightness, Caps Lock) auto-expand briefly, then collapse.
5. Page switching via page dots, mouse wheel and ←/→ keys.
6. Idle state: the clock fades and a procedural "eyes" mascot appears.
7. Fullscreen safety: while another app runs fullscreen, the island hides and never crashes, minimizes or steals focus from that app.

## Nice-to-Have
- Synced lyrics in a split pill.
- Minimal-notch mode (thin line when closed).
- Tray menu and settings window.
- Volume slider on the Music page.
- To-do due-time reminders with snooze.
- Shelf multi-select and text cards.
- Wi-Fi and keyboard-language alerts.
- Adjustable notch size; clock follows the OS 12/24-hour format.

## Technical Constraints
- Pure Rust. No JavaScript and no web engine.
- GPU rendering via wgpu.
- Redraw only while animating, with 0% CPU when idle.
- Targets: ≤40 MB RAM and a binary under 5 MB.
- All data stored locally. No network access except optional lyrics.

## Assumptions
- The reference publishes no dimensions or timings, so all values come from STYLE.md.
- The reference is Windows-only. OS integrations (media session, battery, Bluetooth) are mocked behind a `ContentSource` trait in v1.
