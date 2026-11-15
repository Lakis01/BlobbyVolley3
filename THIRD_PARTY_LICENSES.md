# Third-Party Licenses & Attributions

**Blobby Volley 3** is an unofficial Rust rewrite and remaster of **Blobby Volley 2**. It is
distributed under the GNU General Public License v3.0 or later (`GPL-3.0-or-later`); see
[`COPYING`](COPYING).

This document lists every third-party work that is included in, derived from, or redistributed
with this project, together with the license under which it is used. Where a component is
compiled into or bundled with the game binary, its license has been verified to be compatible
with `GPL-3.0-or-later`.

---

## 1. Blobby Volley 2 (upstream game)

| | |
|---|---|
| **Upstream project** | <https://github.com/danielknobe/blobbyvolley2> |
| **Upstream license** | GNU General Public License v2.0 or later (`GPL-2.0-or-later`) |
| **Our license** | `GPL-3.0-or-later` (see `COPYING`) |
| **Combination basis** | GPLv2 §9 ("GPL v2 or later"), which permits combining GPLv2-or-later code with GPLv3 code and distributing the result under GPLv3 |

### What was taken from Blobby Volley 2

- **Original game design** — the two-player, side-view beach volleyball concept, court
  dimensions, net height, scoring flow, serve handling, and the overall match structure.
- **Physics constants and simulation** — ported from `PhysicWorld.cpp` and `GameConstants.h`
  (gravity, jump impulse, ball restitution, friction, maximum speeds, net collision geometry,
  blob dimensions and movement acceleration). The Rust implementation in `rust/src/` is a
  behavioural port of that simulation; the numerical constants originate upstream.
- **Lua bot behaviours** — the AI personalities are derived from the upstream bot scripts
  `com_11.lua`, `reduced.lua`, and the bot API surface defined in `bot_api.lua`. These inform
  the five calibrated AI tiers (Novice, Amateur, Regular, Master, Pro) in `rust/src/ai.rs`.
- **Sound effects** — `rust/assets/sounds/bums.opus`, `rust/assets/sounds/pfiff.opus`, and
  `rust/assets/sounds/chat.opus` are transcoded from the upstream WAV assets in
  `data/sounds/*.wav` (ball bounce/hit, referee whistle, and chat notification). The audio
  content is unchanged; only the container/codec was converted to Opus for size.

### Original authors

Blobby Volley 2 and its assets are the work of:

- Jonathan Sieber
- Daniel Knobe
- Sven Rech
- Erik Schultheis
- Silvio Mummert
- Richard Bertrand
- and the Blobby Volley 2 contributors

Blobby Volley 3 is **not** affiliated with, endorsed by, or sponsored by any of the above
authors or the upstream project. See the trademark notice at the end of this file.

A copy of the GNU GPL v2.0 is available at <https://www.gnu.org/licenses/old-licenses/gpl-2.0.html>,
and the license for this project (`GPL-3.0-or-later`) is in [`COPYING`](COPYING).

---

## 2. Roboto Font

| | |
|---|---|
| **File** | `rust/assets/fonts/Roboto-Bold.ttf` |
| **Copyright** | Copyright Google LLC |
| **License** | Apache License, Version 2.0 (`Apache-2.0`) |
| **Upstream** | <https://github.com/googlefonts/roboto> / <https://fonts.google.com/specimen/Roboto> |

The Roboto typeface is used for the in-game HUD, menus, scoreboard, and overlays. It is
embedded unmodified (Bold weight). The Apache License 2.0 is compatible with
`GPL-3.0-or-later` distribution; a copy of the license is available at
<https://www.apache.org/licenses/LICENSE-2.0>.

---

## 3. Craft App Sponsor Icons

| | |
|---|---|
| **Files** | `rust/assets/sponsors/*.png` |
| **License** | Dual-licensed `MIT OR Apache-2.0` |
| **License source** | Each upstream repository's `assets/app-icon/LICENSE.txt`, plus the repository-root `LICENSE-MIT` and `LICENSE-APACHE` |
| **Suite** | The `storytold` Craft application suite |

The perimeter advertising boards display the app icons of the following Craft applications.
Each icon is redistributed unmodified at reduced resolution:

| Icon file | Application |
|---|---|
| `photocraft.png` | PhotoCraft |
| `pdfcraft.png` | PdfCraft |
| `filmcraft.png` | FilmCraft |
| `wordcraft.png` | WordCraft |
| `lightcraft.png` | LightCraft |
| `vectorcraft.png` | VectorCraft |
| `effectcraft.png` | EffectCraft |
| `cadcraft.png` | CADCraft |
| `gridcraft.png` | GridCraft |
| `designcraft.png` | DesignCraft |
| `figcraft.png` | FigCraft |
| `deckcraft.png` | DeckCraft |
| `soundcraft.png` | SoundCraft |

Per-icon attribution and the full dual-license texts are preserved in each upstream
repository. `MIT OR Apache-2.0` is compatible with `GPL-3.0-or-later` redistribution.

---

## 4. Rust Crates

Rust dependencies are pinned in `Cargo.lock`. All transitive dependencies use
permissive licenses that are compatible with `GPL-3.0-or-later`:

- **MIT**
- **Apache-2.0**
- **BSD-3-Clause**
- **Zlib**
- **0BSD**
- **Unlicense**

Notable dependencies with direct relevance to the game:

| Crate | Version | License | Purpose |
|---|---|---|---|
| `hound` | 3.5.1 | `Apache-2.0` | WAV decoding of the upstream Blobby Volley 2 sound assets |
| `opus-pure` | 0.2.2 | `BSD-3-Clause` | Pure-Rust Opus decoder for the bundled `.opus` sound effects |
| `macroquad` | 0.4.16 | `MIT OR Apache-2.0` | Windowing, rendering, input, and audio backend |
| `glam` | 0.27.0 | `MIT OR Apache-2.0` | Vector and matrix math |
| `image` | 0.24.9 | `MIT OR Apache-2.0` | PNG decoding for icons, arenas, and sponsor boards |
| `fontdue` | 0.9.5 | `MIT OR Apache-2.0` | Font rasterization for the Roboto face |
| `serde` / `serde_json` | 1.0.x | `MIT OR Apache-2.0` | Configuration, profiles, and JSON protocol payloads |
| `ureq` | 3.5.2 | `MIT OR Apache-2.0` | HTTPS client for the optional matchmaking registry |
| `rustls` | 0.23.x | `Apache-2.0 OR ISC OR MIT` | TLS for registry communication |

To regenerate the full dependency inventory:

```bash
cargo install cargo-license      # once
cargo license --all-features     # prints crate name, version, license
```

Or read `Cargo.lock` directly; each `[[package]]` entry records the exact version in use.

---

## 5. Trademarks & Non-Endorsement

All product names, project names, application names, logos, and icons referenced in this
repository, in the game, or displayed on the in-game perimeter sponsor boards are the
property of their respective owners.

The perimeter boards show the names and marks of **PhotoCraft, PdfCraft, FilmCraft,
WordCraft, LightCraft, VectorCraft, EffectCraft, CADCraft, GridCraft, DesignCraft, FigCraft,
DeckCraft, SoundCraft, Arch Linux, Debian, Fedora, CachyOS, KDE Plasma, Rust, Godot, Blender,
FFmpeg, mpv, Git, OpenCode, Navidrome, and Neovim**, in the same spirit as the original
Blobby Volley's Linux distribution banners.

These marks are used descriptively, for identification and attribution only. Their inclusion
does **not** imply any affiliation with, endorsement of, or sponsorship by those projects, and
does **not** imply that those projects endorse Blobby Volley 3. Blobby Volley 3 is an
independent, unofficial fan project.

Blobby Volley 3 is likewise not affiliated with or endorsed by the Blobby Volley 2 authors or
contributors.

---

## 6. Summary Table

| Component | Path | License |
|---|---|---|
| Blobby Volley 3 (this project) | repository root | `GPL-3.0-or-later` (`COPYING`) |
| Blobby Volley 2 heritage | design, physics, bots, sounds | `GPL-2.0-or-later`, redistributed under `GPL-3.0-or-later` |
| Roboto Bold | `rust/assets/fonts/Roboto-Bold.ttf` | `Apache-2.0` |
| Craft app icons | `rust/assets/sponsors/*.png` | `MIT OR Apache-2.0` |
| Matchmaking Worker | `server/worker.js` | `MIT OR Apache-2.0` |
| Rust crates | `Cargo.lock` | MIT / Apache-2.0 / BSD-3-Clause / Zlib / 0BSD / Unlicense |
