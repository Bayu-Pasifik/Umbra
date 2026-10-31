# STYLE: Visual Guidelines

## Theme
Dark only. The pill sits flush against the top screen edge, like a hardware notch.

## Shape
- Collapsed: 200×32 px, fully rounded (radius = height / 2).
- Expanded: up to 420×180 px, corner radius 28 px.
- Width, height and radius interpolate together, so no hard corners appear mid-animation.

## Color Palette
| Role | Value |
|---|---|
| Background | #000000 |
| Primary text | #FFFFFF |
| Secondary text | #FFFFFF at 60% alpha |
| Accent / active state | #0A84FF |
| Tracks / disabled | #FFFFFF at 15% alpha |

## Motion
- Spring physics, not fixed easing: stiffness 300, damping 26, mass 1. This gives a slight overshoot of about 3%.
- Content fades in after 60% of the expansion and fades out before the collapse starts.
- Alerts hold for 2 s.

## Typography
- System-ui sans-serif for labels.
- Monospace (tabular digits) for the clock and timers.
- Sizes: 13, 15 and 22 px.
- Minimal text: at most one title and one subtitle per view.

## Icons and Assets
- All procedural: vector paths drawn with egui shapes (play, pause, skip, drop, battery, headphones, eyes).
- No image files.
- 1.5 px strokes in white or accent.

## Assumptions
The reference publishes no exact values. The numbers above are chosen defaults.
