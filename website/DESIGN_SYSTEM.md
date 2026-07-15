# Design System — OpenInstall Catalog

## Philosophy

Minimalism. Almost asceticism. Every pixel must have a purpose.

## Colors

ONLY TWO:
- Black: `#000000`
- White: `#FFFFFF`

NO gray, NO blue, NO red, NO green, NO accent colors, NO transparent colors, NO gradients, NO shadows, NO blur, NO glassmorphism.

## Geometry

ONLY rectangles.

FORBIDDEN:
- Circles, ovals, capsules, pills
- Any border-radius
- Decorative lines, decorative figures
- Icons with circular geometry

ALL elements: `border-radius: 0`

## Borders

Every interactive element has a border. Borders communicate state.

| State | Border |
|-------|--------|
| Default | `1px solid #000000` |
| Active/Focus | `2px solid #000000` |
| Disabled | `1px solid #000000` at 50% opacity |
| Selected | Inversion (black bg, white text) |

## Typography

- Font: Inter (Google Fonts)
- Weights: 300, 400, 500, 600, 700, 900
- Headings: `font-weight: 900`, `letter-spacing: -0.05em`
- Body: `font-weight: 400`, `line-height: 1.6`
- Giant headings: `font-size: clamp(3rem, 10vw, 12rem)`

## Spacing (8px base)

| Token | Value |
|-------|-------|
| xs | 4px |
| sm | 8px |
| md | 16px |
| lg | 24px |
| xl | 32px |
| 2xl | 48px |
| 3xl | 64px |
| 4xl | 96px |
| 5xl | 128px |

## Animations

ONLY soft fade-ins: `opacity 0 → 1`, `transition: 200-250ms ease-out`.

FORBIDDEN: scale, bounce, rotate, spring, spinning, zoom, flashy effects, parallax.

Elements appear from darkness, materialize from the interface.

## Components

### Button Primary
```css
background: #000000;
color: #FFFFFF;
border: none;
padding: 12px 24px;
font-weight: 600;
cursor: pointer;
transition: opacity 200ms;
```
Hover: `opacity: 0.8`

### Button Secondary
```css
background: #FFFFFF;
color: #000000;
border: 1px solid #000000;
padding: 12px 24px;
font-weight: 600;
cursor: pointer;
transition: all 200ms;
```
Hover: `background: #000000; color: #FFFFFF`

### Input
```css
background: #FFFFFF;
border: 1px solid #000000;
padding: 12px 16px;
font-size: 16px;
border-radius: 0;
outline: none;
```
Focus: `border-width: 2px; padding: 11px 15px`

### Card
```css
background: #FFFFFF;
border: 1px solid #000000;
padding: 24px;
```

### Navbar
```css
background: #FFFFFF;
border-bottom: 1px solid #000000;
height: 64px;
position: fixed;
top: 0;
width: 100%;
```

## Layout

- Max-width: 1200px
- Margin: 0 auto
- Padding: 0 24px
- Grid-based structure
- Large whitespace — empty space is part of the design

## UX Rules

User understands through:
- Borders → what is clickable
- Inversion → what is active/selected
- Border thickness → what is important
- Composition → what is disabled

Never use color alone to communicate state.
