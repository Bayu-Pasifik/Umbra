# TASKS: Implementation Checklist

Total estimate: about 54 hours, in order.

- [ ] 1. Scaffold the cargo project with eframe (wgpu backend), fmt and clippy (1h)
- [ ] 2. Create a borderless, transparent, always-on-top window at top-center (3h)
- [ ] 3. Render the pill as a rounded rect and verify DPI scaling (2h)
- [ ] 4. Implement a spring struct (value, velocity, target) with fixed-step integration (3h)
- [ ] 5. Animate width, height and radius with springs, driven by an expand/collapse state machine (3h)
- [ ] 6. Expand on hover; collapse after a 300 ms leave delay (2h)
- [ ] 7. Make clicks pass through outside the pill, via window resize or an input region (3h)
- [ ] 8. Define a `Widget` trait (size, draw, input) and a page stack (3h)
- [ ] 9. Build the Clock, Music (mocked), Pomodoro, Water and To-do widgets (8h)
- [ ] 10. Add page dots plus wheel and arrow-key navigation with crossfade (3h)
- [ ] 11. Build a priority alert queue with auto-expand and a 2 s hold (3h)
- [ ] 12. Draw procedural icons and the idle eyes mascot (4h)
- [ ] 13. Build the file shelf: drag-in via winit, drag-out via a per-OS crate (5h)
- [ ] 14. Persist data as serde JSON in the config directory (2h)
- [ ] 15. Performance: repaint only while springs are active, then verify 0% idle CPU and memory use (3h)
- [ ] 16. Edge cases: detect fullscreen apps and hide without stealing focus or minimizing them; multiple monitors, DPI changes, rapid hover toggling, alerts arriving while expanded (4h)
- [ ] 17. Package builds for all three OSes (2h)

## Assumptions
- Always-on-top and click-through behavior differ per OS.
- Wayland may require layer-shell, or accepting a positioning limitation.
