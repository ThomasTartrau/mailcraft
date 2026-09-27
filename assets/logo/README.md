# mailcraft logo

A postage stamp struck with an `m`: an email only leaves once it is franked, and mailcraft franks it
at compile time. The name is always lowercase: `mailcraft`.

## Files

| File | What it is |
|---|---|
| `mailcraft-logo-light.svg` | Horizontal logo for light backgrounds (green stamp, ink wordmark) |
| `mailcraft-logo-dark.svg` | Horizontal logo for dark backgrounds (green stamp, paper wordmark, drawn a touch thinner) |
| `mailcraft-banner.png` | 2560 x 640 banner with its own dark background, readable on any theme |
| `mailcraft-symbol.svg` | The stamp alone, transparent (256 x 256 viewBox); the `m` is a real cut-out |
| `mailcraft-favicon.svg` | Small-size cut of the stamp for 16-32 px: 3 larger perforations per side, heavier `m` |
| `mailcraft-favicon.ico` | 16, 32 and 48 px favicon rendered from `mailcraft-favicon.svg` |
| `mailcraft-icon.svg` | Square icon: green stamp on a rounded ink tile (200 x 200 viewBox) |
| `mailcraft-avatar.png` | 512 x 512 avatar: green stamp on a full ink square, safe for circular crops |
| `mailcraft-social.png` | 1280 x 640 card: logo, tagline, `#[derive(EmailTemplate)]` |

## What to use where

### README

The root `README.md` already uses the header below, with paths relative to the repository root.

```html
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="assets/logo/mailcraft-logo-dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="assets/logo/mailcraft-logo-light.svg">
  <img alt="mailcraft" src="assets/logo/mailcraft-banner.png" width="560">
</picture>
```

Renderers that keep `<picture>` (GitHub, for instance) pick the light or dark SVG from the viewer's
theme. GitLab strips `<picture>` and `<source>`, so it shows the `<img>` fallback, the banner, which
reads on both GitLab themes.

### docs.rs

Both `lib.rs` files set `#![doc(html_logo_url = ..., html_favicon_url = ...)]`: the sidebar shows
`mailcraft-symbol.svg` and the tab shows `mailcraft-favicon.ico`. The cut-out `m` takes the colour
of the page, so the same file works on the light, dark and ayu themes.

### Portfolio (tartrau.fr project page)

- Project logo: `mailcraft-icon.svg`, copied to the site as `/logos/mailcraft.svg`. It follows the
  same template as the other project logos (200 viewBox, 180 tile, radius 40) and works in the light
  and dark themes without a variant.
- Project image: `mailcraft-social.png`, also usable as the Open Graph image.

### LinkedIn

- Project section of the profile, or a post about mailcraft: `mailcraft-social.png` as the media.
- LinkedIn page logo, if a page is ever created: `mailcraft-avatar.png`.

### GitLab

- Project avatar: `mailcraft-avatar.png`, in Settings > General > Project avatar (GitLab
  recommends 192 x 192 and 200 KB at most; it resizes the file).

### GitHub

- A repository has no icon. Set `mailcraft-social.png` in Settings > General > Social preview; it
  shows in link previews on LinkedIn, Slack, X and others.

## Colours

| Name | HEX | RGB | Use |
|---|---|---|---|
| Stamp green | `#178F52` | 23, 143, 82 | The stamp only |
| Ink | `#0E0F12` | 14, 15, 18 | Wordmark on light backgrounds, dark tiles |
| Paper | `#F4F3EF` | 244, 243, 239 | Wordmark on dark backgrounds |

The green is a nod to the French "lettre verte" stamp and to a green build. Contrast: 4.1:1 on white,
3.7:1 on paper, 4.6:1 on ink.

## Rules

- Clear space around the logo: at least the height of the `a` of the wordmark.
- Minimum size: 120 px wide for the horizontal logo; 32 px for `mailcraft-symbol.svg`. Below 32 px,
  use `mailcraft-favicon.svg`.
- One colour: the stamp can be set in ink or paper when green is not available; keep the `m` as a
  cut-out.
- Don't stretch, rotate, recolour, add effects or shadows, fill the `m` with another colour, change
  the number of perforations, or retype the wordmark in a font. The wordmark is drawn as paths,
  not set in a typeface.
