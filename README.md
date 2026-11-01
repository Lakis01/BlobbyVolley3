# Blobby Volley 3 (Remastered)

A modern, high-performance port and complete visual remaster of the legendary **Blobby Volley** head-to-head multiplayer volleyball game, rewritten in **Rust** for Linux and modern platforms.

---

## Highlights of the 2026 Remaster

- **Authentic Physics Model**:
  - Deterministic 75 Hz simulation perfectly preserving the iconic Blobby jump curve (`BLOBBY_JUMP_ACCELERATION = -15.1`), double-sphere blobby collision, ball rebound velocities, net reactions, and squash tolerance.
- **2026 Visual Overhaul**:
  - **Procedural Organic Jelly Blobbies**: Smooth volume-preserving squash-and-stretch spring dynamics on jumping, landing, and ball strikes. Translucent gelatin shading with specular gloss highlights and contact sand shadows.
  - **Animated Expressive Eyes**: Real-time pupil tracking that watches the volleyball wherever it flies, plus natural blinking and reaction expressions.
  - **3D-Shaded Volleyball**: Smooth continuous rotation with authentic multi-color panels, dynamic ground shadow scaling with height, and fiery / electric motion trail ribbons when spiked.
  - **3 Remastered Arenas**:
    1. **Copacabana Sunset Beach**: Multi-layer sunset sky gradient, giant glowing sun with lens bloom, parallax drifting clouds, animated ocean waves with sunlight caustics and cresting foam, and swaying tropical palm trees.
    2. **Neo Tokyo Cyber Stadium**: Synthwave wireframe sun, cyberpunk skyline, perspective neon grid floor, and pulsating laser net.
    3. **Classic Beach HD**: Ultra-crisp modern tribute to the nostalgic Blobby Volley 1/2 sunny beach.
  - **Visual Juice & VFX**: Sand puff particles on landing, shockwave impact rings, spark bursts on spikes, celebratory confetti, and subtle screen shake.
  - **Widescreen & High-DPI**: Native scaling for 16:10 and 16:9 displays (optimized for 2560x1600).
- **Audio Engine**:
  - Embedded sound effects (bums, referee whistle, score chime, chat ping) with velocity-modulated impact volume.
- **AI & Game Modes**:
  - 1-Player vs Bot (Novice, Pro [Reduced bot], Master [Combot 1.1]).
  - 2-Player Local Versus (Same keyboard or gamepad).
  - Online & LAN Network Multiplayer (Authoritative host, low-latency TCP socket engine on port 1234, real-time in-game chat).
  - CPU vs CPU Exhibition mode.
  - Rule Modes: **Modern (Rally Points)** or **Classic (Side-Out)**.
  - Score limits: First to 5, 10, 15, or 21 points.

---

## Authentic AI Alignment
- **Master (Combot 1.1)**: Exact port of the legendary BV2 `com_11.lua`. Executes touch-0 ground setups, high-jump spike attacks (`sprungattacke`) with randomized attack power (30–55) scaled to court depth, and net tape roller recovery.
- **Pro (Reduced Bot)**: Exact port of `reduced.lua`. Features proactive high-play jump spikes (`highPlayPos = target - 50`), disciplined ground head receives, and emergency diving saves (`panic jump`).
- **Novice**: Accessible entry-level bot with relaxed reaction time, ground returns, and gentle pop-ups.

---

## Online & LAN Multiplayer
- **Host Match**: Select *Host Online Game* from the main menu. The game automatically displays your detected local LAN IP address and listens on port 1234.
- **Join Match**: Select *Join Online Game* and enter the Host IP address (`<IP>:<PORT>` or `<IP>`).
- **In-Game Chat**: Press `T` or `Enter` to open chat, type your message, and hit `Enter` to send. Received messages play the authentic `chat.wav` audio ping.

---

## Controls

| Player | Left | Right | Jump / Spike |
|---|---|---|---|
| **Player 1** | `A` or `Left Arrow` (in 1P) | `D` or `Right Arrow` (in 1P) | `W`, `Space`, or `Up Arrow` (in 1P) |
| **Player 2** | `Left Arrow` or `J` | `Right Arrow` or `L` | `Up Arrow`, `I`, `Enter`, or `Right Shift` |

- **In-Game Chat**: `T` or `Enter` (press `Enter` to send, `Escape` to cancel)
- **Pause / In-Game Menu**: `Escape` or `P`
- **Toggle Audio Mute**: `M`
- **Menu Navigation**: `Up`/`Down` (or `W`/`S`), change options with `Left`/`Right` (or `A`/`D`), select with `Space` or `Enter`.

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

### Direct Binary
The optimized release binary is located at:
```bash
./target/release/blobby
```

