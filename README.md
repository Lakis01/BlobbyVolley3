# Blobby Volley 3 (Remastered)

A modern, high-performance port and complete visual remaster of the legendary **Blobby Volley** head-to-head multiplayer volleyball game, rewritten in **Rust** for Linux and modern platforms.

---

## Copyright & Attribution

Blobby Volley 3 (Remastered) is a modern Rust port and remaster of **Blobby Volley 2**, originally created by Daniel Knobe and the Blobby Volley Community ([github.com/danielknobe/blobbyvolley2](https://github.com/danielknobe/blobbyvolley2)).

- **License**: GNU General Public License v2.0 ([GPL-2.0](COPYING))
- **Original Authors & Heritage**: Daniel Knobe, Silvio Mummert, and the Blobby Volley 2 Contributors.
- All original game assets, physics constants, sound effects (encoded in Opus 100 kbps), and tournament rules are preserved and remastered in full compliance with the GNU GPL v2.0.

---

## 2026 Remaster Highlights

### 1. Visual Overhaul & Arenas
- **6 Hand-Crafted Arenas**:
  1. **Copacabana Sunset Beach**: Multi-layer sunset sky gradient, giant glowing sun with lens bloom, parallax drifting clouds, animated ocean waves with sunlight caustics and cresting foam, and swaying tropical palm trees.
  2. **Neo Cyber Stadium**: Synthwave wireframe sun, cyberpunk skyline, perspective neon grid floor, and pulsating laser net.
  3. **Classic Beach HD**: Ultra-crisp modern tribute to the nostalgic Blobby Volley 1/2 sunny beach.
  4. **Alpine Peaks (Snow)**: Majestic snow-capped mountain peaks, pine tree silhouettes, and drifting procedural snowflakes.
  5. **Magma Caldera (Volcano)**: Glowing lava lakes, volcanic crater horizon, rising heat shimmer, and floating ember particles.
  6. **Orbital Station (Space)**: Deep space starfield, glowing blue Earth curvature, orbital solar arrays, and high-tech containment court.
- **Open-Source Perimeter Advertising Boards**:
  - In the spirit of the original game's Linux distro banners, the court perimeter boards showcase prominent open-source software and Linux projects: **Arch Linux**, **Debian**, **Fedora**, **CachyOS**, **KDE Plasma**, **Rust**, **Godot Engine**, **Blender**, **FFmpeg**, **VLC**, **Git**, and **Neovim**.
- **Procedural Organic Jelly Blobbies**:
  - Volume-preserving squash-and-stretch spring dynamics on jumping, landing, and ball strikes. Translucent gelatin shading with specular gloss highlights and contact sand shadows.
- **Animated Expressive Eyes**:
  - Real-time pupil tracking that dynamically watches the volleyball wherever it flies across the net, plus natural blinking and reaction expressions.
- **3D-Shaded Volleyball & VFX**:
  - Smooth continuous rotation with authentic multi-color panels, dynamic ground shadow scaling with height, fiery/electric motion trail ribbons when spiked, sand puffs on impact, shockwave rings, and celebratory confetti.

---

### 2. 5-Tier AI Difficulty System
The AI opponent engine has been expanded from legacy scripts into 5 carefully calibrated tiers:
1. **Novice**: Accessible entry-level bot with relaxed reaction time, ground returns, and gentle pop-ups.
2. **Amateur**: Intermediate opponent with balanced court coverage, proactive ball positioning, and gentle jump touches.
3. **Regular**: Classic competitive baseline (formerly Master / Combot 1.1). Executes touch-0 ground setups, high-jump spike attacks (`sprungattacke`), and net tape roller recovery.
4. **Master**: Advanced tournament-grade bot featuring defensive recovery, predictive trajectory intercepts, and targeted corner spikes.
5. **Pro**: Legendary peak difficulty (formerly Reduced Bot). Features aggressive high-play spikes (`highPlayPos = target - 50`), disciplined ground head receives, and emergency diving saves (`panic jump`).

---

### 3. Settings Menu & Input Handling
- **Value Cycling**: Change any setting's value via `Left`/`Right` arrow keys when highlighted, OR by clicking directly on the `<` and `>` arrow buttons next to the setting—even if that row is not currently focused.
- **Confirm & Quick Start**: Pressing `Enter` or `Space`, or clicking anywhere on the center label/body of a setting row, confirms the current configuration and immediately starts the game (without accidentally cycling the value).
- **Fullscreen Support**: Fullscreen toggle option in the settings menu, as well as `F11` and `Alt+Enter` shortcuts anytime. Aspect ratio is preserved with pristine letterboxing for widescreen (16:10, 16:9, and high-DPI displays).
- **Player Profiles & Identity**: Persistent player display names stored in `~/.config/blobbyvolley3/config.json`. Displays your custom name in local and online matches, HUD, and victory screen. In 2-Player mode, prompts Player 2 to customize their name (defaulting to "Player 2").

---

### 4. Multiplayer & Matchmaking
- **Public Server Browser**: Built-in matchmaking browser that queries the official master list at `blobbyvolley.de` and lists dedicated community servers.
- **Zero-Cost P2P & LAN Multiplayer**: Host or join matches with authoritative server simulation, low-latency TCP packet networking, and in-game chat (`T` to chat).

---

## Controls

| Action | Player 1 | Player 2 |
|---|---|---|
| **Move Left** | `A` or `Left Arrow` (in 1P) | `Left Arrow` or `J` |
| **Move Right** | `D` or `Right Arrow` (in 1P) | `Right Arrow` or `L` |
| **Jump / Spike** | `W`, `Space`, or `Up Arrow` (in 1P) | `Up Arrow`, `I`, `Enter`, or `Right Shift` |

- **In-Game Chat**: `T` or `C` (press `Enter` to send, `Escape` to cancel)
- **Pause / In-Game Menu**: `Escape` or `P`
- **Toggle Audio Mute**: `M`
- **Toggle Fullscreen**: `F11` or `Alt+Enter`
- **Menu Navigation**: `Up`/`Down` (or `W`/`S`), cycle values with `Left`/`Right` or click `<` / `>`, confirm with `Enter` or row click.

---

## How to Build & Run

### Quick Run
```bash
cargo run --release
```

### Run Tests
```bash
cargo test
```

### Build Release Artifacts
```bash
cargo build --release
```
The optimized release binary is located at:
```bash
./target/release/blobby
```
