<p align="center">
    <img src="https://img.shields.io/github/languages/top/wiremann/wiremann?style=for-the-badge" alt="languages"/>
    <img src="https://img.shields.io/github/commit-activity/m/wiremann/wiremann?style=for-the-badge" alt="commits"/>
    <img src="https://img.shields.io/github/stars/wiremann/wiremann?style=for-the-badge" alt="stars"/>
    <img src="https://img.shields.io/github/watchers/wiremann/wiremann.svg?style=for-the-badge" alt="watchers"/>
    <img src="https://img.shields.io/github/license/wiremann/wiremann.svg?style=for-the-badge" alt="license"/>
</p>

# Wiremann

![Main Showcase.png](assets/screenshots/Main%20Showcase.png)

A fast, no-bullshit music player built in Rust. Just pure, native speed.

---

## What & Why

Most music players pick a side.
They either look good and run like trash, or they’re fast and feel like they’re from 2005.

Wiremann doesn’t play that game.

Built from the ground up in Rust with a fully native, GPU-accelerated UI. No Electron, no webviews, no hidden garbage
eating your RAM.

The goal is simple:

* instant response
* low resource usage
* clean UI
* no unnecessary layers

If it can be faster, it should be.

---

## Features

* **Rust core** – no compromises
* **Fully native UI** – built with [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui)
* **GPU accelerated rendering** – smooth 120 FPS
* **Multithreaded** – actually uses your CPU
* **Low idle usage** – ~1–2% CPU
* **Memory efficient** – <80MB with 500+ tracks
* **Audio backend** – `rodio` for now

    * planned: `symphonia` + `cpal`
* **Metadata decoding** – `lofty`

    * planned: switch to `symphonia` (v0.2, big perf gains)
* **Binary data storage** – no SQL, no queries, just fast I/O

---

## Installation

### From source

```bash
git clone https://github.com/wiremann/wiremann
cd wiremann
cargo run
```

---

### Prebuilt binaries

Grab them from the Releases page:
[https://github.com/wiremann/wiremann/releases](https://github.com/wiremann/wiremann/releases)

Download. Run. Done.

---

## License

GPL-3.0
[https://www.gnu.org/licenses/gpl-3.0.en.html](https://www.gnu.org/licenses/gpl-3.0.en.html)
