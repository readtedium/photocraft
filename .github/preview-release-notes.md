**Unofficial preview of the frame-animation work for [PhotoCraft](https://github.com/storytold/photocraft), for people who want to try it before it is merged.** It is not an official PhotoCraft release and nobody has reviewed it yet. Expect rough edges, and don't use it as your only copy of anything.

It is built from the `frame-animation-preview` branch of this fork: PhotoCraft `main` plus the seven pull requests tracked in [issue #3263](https://github.com/storytold/photocraft/issues/3263) (#6142, #6143, #6626, #6627, #6842, #6843, #6844).

## What to try
- **Window › Timeline** now docks under the canvas. **Create Frame Animation** or **Make Frames From Layers** makes a Photoshop-style frame strip.
- Open an **animated GIF**: it opens as one layer per frame with each frame's delay. Save as `.gif` to write the animation back.
- Frames: per-frame delay and loop count, drag to reorder, select several frames (Ctrl/Cmd-click, Shift-click), copy and paste (Ctrl+C / Ctrl+V), delete.
- Keys while the pointer is over the Timeline: **Space** plays and pauses, ← → step, Home/End jump, Alt+← → move the selected frames.
- The **…** menu in the Timeline header turns on **Tween** (fade between frames) and **Onion Skin**; both are off by default.

Feedback is very welcome on [issue #3263](https://github.com/storytold/photocraft/issues/3263).

## Downloads
| Platform | File |
|---|---|
| Windows 64-bit | the `.msi` installer, or the `-portable.zip` (unzip and run `photocraft.exe`) |
| macOS, Apple Silicon (M1 and later) | the `.dmg` |
| Linux 64-bit (x86_64) | the `.AppImage` (`chmod +x` it, then run it) |

`SHA256SUMS.txt` has the checksum of every file.

## These builds are not signed
No code-signing certificates are used, so each system will warn you:
- **Windows**: SmartScreen says "Windows protected your PC". Choose **More info › Run anyway**.
- **macOS**: the app is only ad-hoc signed and not notarized, so Gatekeeper will refuse to open it at first. Drag PhotoCraft to Applications, then either right-click it and choose **Open**, or run `xattr -dr com.apple.quarantine /Applications/PhotoCraft.app` in Terminal. On recent macOS you may instead need **System Settings › Privacy & Security › Open Anyway**.
- **Linux**: needs a reasonably recent distro (built on Ubuntu 22.04, so glibc 2.35 or newer).

Built without the bundled Craft fonts (they live in a private repository), so the font list is shorter than in an official build. Some of the new interface text is machine-translated and has not been reviewed by native speakers.

PhotoCraft is licensed under MIT OR Apache-2.0; see the licence files in the repository.
