# ghostto

[![Crates.io](https://img.shields.io/crates/v/ghostto.svg)](https://crates.io/crates/ghostto)
[![Downloads](https://img.shields.io/crates/d/ghostto.svg)](https://crates.io/crates/ghostto)
[![Docs.rs](https://img.shields.io/docsrs/ghostto)](https://docs.rs/ghostto)
[![CI](https://github.com/ghosttos/ghostto/actions/workflows/ci.yml/badge.svg)](https://github.com/ghosttos/ghostto/actions)
[![License: GPL-3.0-or-later](https://img.shields.io/badge/license-GPL--3.0--or--later-blue.svg)](#license)
[![MSRV](https://img.shields.io/badge/MSRV-1.80-blue.svg)](https://blog.rust-lang.org/)

> **Status:** `v0.0.0` — name reservation only. The engine is under active development. Watch this repository or the crates.io feed for `v0.1.0`.

**ghostto** is a from-scratch, memory-safe emulation engine written in Rust, designed to run Android, Linux, and Windows apps in a browser tab via one native binary on localhost. It prioritizes speed, correctness, and freedom—no WebAssembly, no QEMU, no install, no paywalls.

---

## Table of Contents

- [What it is](#what-it-is)
- [Features](#features)
- [Architecture](#architecture)
- [Dependencies](#dependencies)
- [Installation](#installation)
- [Usage](#usage)
- [Browser UI](#browser-ui)
- [Save state](#save-state)
- [Freedom policy](#freedom-policy)
- [Roadmap](#roadmap)
- [Performance](#performance)
- [Contributing](#contributing)
- [License](#license)

---

## What it is

`ghostto` is a multi-OS browser emulator engine. It does not wrap QEMU, v86, CheerpX, or any existing emulator core. Every layer—instruction decoding, execution, memory management, devices, display, and the local signal channel—is implemented from scratch in Rust.

The engine is being built to:

- **Run Android** (ARM64) in a browser tab, for testing apps without BlueStacks
- **Run Linux** (ARM64 first, x86_64 later) in the same tab
- **Run Windows** (x86_64) in the same tab, once the x86_64 interpreter lands
- **Avoid WebAssembly entirely** — the browser is a UI client; all emulation runs in the native binary
- **Avoid QEMU** — every layer is owned by this project
- **Run as one foreground process** — `127.0.0.1:7777` (fallback `7771`), Ctrl-C to stop, no system service

The name comes from the idea that the guest OS "haunts" the browser—present and running, without being native to it.

---

## Features

- **Pure Rust core** — no GC, no hidden allocations in hot paths, ownership-based memory safety
- **Tail-call interpreter** — `become` on nightly, match-dispatch fallback on stable
- **Ghostto Auto-Chunking (GAC)** — adaptive JIT chunk sizing (seed 512, min 256, max 16384, 1–4 ms compile-time band, per-block history)
- **Cranelift tiered block JIT** — Tier 0 interpreter, Tier 1 hot-block JIT, no WebAssembly emission
- **zenjpeg MJPEG** — hybrid jpegli adaptive quantization + mozjpeg trellis; `codec_id` header keeps VP9/H.264 swappable
- **No WebAssembly, no QEMU, no daemon** — one foreground binary, `127.0.0.1` only
- **WebSocket signal channel** — browser renders frames and sends input; backend emulates
- **Resizable canvas** — fit / fill / stretch modes; guest resolution sync on Android at M4
- **Save state** — `GHST` block format, zstd-compressed RAM, mandatory copy-on-write disk overlay
- **Bring your own OS** — the project ships no guest images, firmware, or BIOS
- **GPL-3.0-or-later + DCO** — free forever, no CLA, no relicensing escape hatch

---

## Architecture

`ghostto` is organized as a Cargo workspace. Each crate has a single responsibility:

| Crate | Purpose |
|---|---|
| `ghostto` | Umbrella facade crate |
| `ghostto_cli` | `ghostto` binary entrypoint (`ghostto up`, `ghostto list`) |
| `ghostto_http` | HTTP server on `127.0.0.1:7777` (fallback `7771`) |
| `ghostto_ws` | WebSocket signal protocol at `/ws` |
| `ghostto_proto` | Shared message types (frame header, control, input, event) |
| `ghostto_cpu_arm64` | ARM64 interpreter with tail-call dispatch |
| `ghostto_cpu_x86_64` | x86_64 interpreter (later) |
| `ghostto_mm` | Guest RAM, paging, software TLB |
| `ghostto_display` | Framebuffer capture + zenjpeg MJPEG encoding |
| `ghostto_input` | Browser input → guest events |
| `ghostto_jit` | Cranelift tiered block JIT + Ghostto Auto-Chunking |
| `ghostto_snapshot` | `GHST` save-state format and copy-on-write disk overlay |
| `ghostto_android` | Android guest profile |
| `ghostto_linux` | Linux guest profile |
| `ghostto_windows` | Windows guest profile (later) |

The `web/` directory (plain HTML, JS, CSS — no build step, no wasm) is served by `ghostto_http`. The engine never links a third-party emulator core. Every layer is owned by this project.

---

## Dependencies

`ghostto` deliberately keeps its dependency tree small. The interpreter and MMU depend only on the Rust standard library.

### Runtime dependencies

| Crate | Purpose | Optional |
|---|---|---|
| `zenjpeg` | MJPEG frame encoding (jpegli AQ + mozjpeg trellis hybrid) | ❌ |
| `axum` | HTTP + WebSocket server (lighter alternatives under consideration) | ❌ |
| `tokio` | Async runtime for the server | ❌ |
| `serde` | Snapshot and protocol serialization | ❌ |
| `bincode` | Compact binary format for structured snapshot blocks | ❌ |
| `zstd` | RAM compression in snapshots | ❌ |
| `cranelift-jit` | Tiered block JIT (Tier 1) | ✅ Yes (JIT only) |
| `wgpu` | Host GPU acceleration for the virtual GPU | ✅ Yes (future) |

### Development dependencies

| Crate | Purpose |
|---|---|
| `criterion` | Benchmarks |
| `proptest` | Property-based testing of the decoder |
| `cargo-fuzz` | Decoder and protocol fuzzing |

The interpreter and memory management layers have **zero external dependencies**.

---

## Installation

`ghostto` is not yet published as a usable library. The `v0.0.0` release on crates.io is a name reservation.

Once `v0.1.0` lands:

```toml
[dependencies]
ghostto = "0.1"
```

The CLI binary will also be downloadable from GitHub Releases as a portable, no-install executable per platform.

---

## Usage

Native (once v0.1.0 is available)

```rust
use ghostto::Engine;

fn main() -> ghostto::Result<()> {
    let mut engine = Engine::builder()
        .memory(256 * 1024 * 1024)        // 256 MiB guest RAM
        .vcpus(1)
        .guest_image("android.img")       // bring your own OS
        .build()?;

    engine.run()?;

    Ok(())
}
```

CLI

```bash
# foreground process, prints the local URL, Ctrl-C to stop
ghostto up --guest android.img
# → ghostto listening on http://127.0.0.1:7777
```

No system service is installed. No installer hooks the OS. The binary you launch is the emulator and the local server.

---

## Browser UI

The browser is a dumb terminal. All emulation runs in the native `ghostto` binary on `127.0.0.1`; the tab only renders frames and forwards input.

- `ghostto up` starts the binary and prints `http://127.0.0.1:7777`
- The tab connects to `ws://127.0.0.1:7777/ws`
- Binary frames carry encoded display output (`codec_id` header selects MJPEG/VP9/H.264)
- JSON messages carry control (`boot`, `pause`, `snapshot`, `resize`) and input (`key`, `mouse`)
- No WebAssembly, no SharedArrayBuffer, no COOP/COEP headers required

Signal protocol:

| Direction | Signal | Format |
|---|---|---|
| backend → UI | `frame` | binary (header: magic, `codec_id`, `w`, `h`, `seq`) |
| backend → UI | `event` | JSON (`boot`, `ready`, `resize`, `error`) |
| UI → backend | `input.key` | JSON |
| UI → backend | `input.mouse` | JSON |
| UI → backend | `control` | JSON (`boot`, `pause`, `snapshot`, `resize`) |

The `<canvas>` is freely resizable (CSS resize + `ResizeObserver`). The backing store stays at the guest resolution; only CSS size changes, so scaling is free on the GPU. Three modes — **fit** (default), **fill**, **stretch** — are persisted in `localStorage`. Guest resolution sync (`control.resize` → Android `wm size`) arrives at M4, throttled to pointerup, max 1 per 200 ms, with UI revert on rejection.

---

## Save state

`GHST` is a versioned, block-based snapshot format. Every component implements `Snapshot::save/restore`; the orchestrator walks a fixed order with the interpreter paused.

```
[Magic: "GHST" | version u16 | flags u16 | CRC32]
[Block 0x0001: Metadata (JSON: guest name, ghostto version, timestamp)]
[Block 0x0010: CPU state (fixed size per vCPU)]
[Block 0x0020: RAM (raw or zstd-compressed)]
[Block 0x0030: MMU / page tables]
[Block 0x0040: Device block, one per device]
[Block 0x0050: Disk overlay reference]
[Block 0x00FF: Footer + checksum]
```

- **Atomic write** — `snapshot.tmp`, `fsync`, `rename`. No half-written files.
- **Checksummed** — CRC32 over the whole file; mismatch is a hard reject.
- **Deterministic state** — RNG, input latches, and instruction count are part of the snapshot.
- **Disk** — copy-on-write overlay is mandatory. Full-disk snapshots are prohibited; a 64 GB Android image makes them unusable.
- **Restore order** — memory → CPU → devices → resume.

Save state ships at **M5**, after Android boots, because building it before the device model stabilizes means rewriting serialization twice.

---

## Freedom policy

ghostto is free software, free for everyone, forever.

- **License:** GPL-3.0-or-later
- **Contributions:** DCO sign-off (`git commit -s`), never a CLA. A CLA is how open projects get bought and closed; DCO keeps copyright with contributors so no one can relicense `ghostto` closed.
- **No paywalls, time limits, feature gating, ads, telemetry, or accounts.** Ever.
- **No closed-source components** or proprietary firmware blobs.
- **No install-to-system** — portable foreground binary only.
- **Bring your own OS** — users supply guest images; the project ships no OS, firmware, BIOS, or recovery files.
- **Funding** (donations, grants, hardware) may pay developers; it may never gate features. Any PR that violates this is rejected.

macOS and iOS are **private-use only**: the code may exist, but is never enabled by default, requires a user-supplied image, and produces no shipped demos, clips, screenshots, or GIFs in the repo, website, or releases.

---

## Roadmap

The project is developed in milestones, each gated by a concrete test:

| Milestone | Target | Gate |
|-----------|--------|------|
| M0 | Binary listens on `127.0.0.1:7777`, serves `web/index.html` | Tab loads UI |
| M1 | `/ws` streams a test-pattern framebuffer to a freely resizable `<canvas>` (zenjpeg MJPEG) | Frame renders in tab |
| M2 | ARM64 tail-call interpreter passes golden-register test | Golden-register test |
| M3 | Boot minimal Linux ARM, live framebuffer in tab | BusyBox prompt in tab |
| M4 | Android APK loads and renders; `control.resize` + `wm size` sync | APK visible at chosen size |
| M5 | Keyboard + mouse reach guest; save-state ships | Input works, snapshot round-trips |
| M6 | x86_64 interpreter; Windows guest profile | Windows boots |
| M7 | macOS/iOS private feasibility study (legal-gated, not shipped) | Private-only, no clips |
| Post-1.0 | Fidelity track + native track | 125-issue catalog closed with fixtures; same-ISA direct execution and optional KVM/HVF/WHPX backend |

---

## Performance

ghostto is designed around the principle that a well-tuned interpreter plus an adaptive JIT beats a poorly-integrated one.

- **Tail-call interpreter** — `become` on nightly eliminates return-to-dispatch overhead; match-dispatch fallback on stable
- **Cranelift tiered JIT** — Tier 0 interpreter handles cold code; Tier 1 compiles hot blocks (>1000 executions)
- **Ghostto Auto-Chunking (GAC)** — chunk size adapts to measured codegen time (1–4 ms band); `history[block_hash]` warm-starts hot loops
- **zenjpeg MJPEG** — hybrid encoder chosen for both size and quality; `codec_id` keeps VP9/H.264 swappable
- **Localhost only** — bandwidth is free; MJPEG's size penalty costs nothing on `127.0.0.1`
- **Measurement-gated** — no JIT, no codec upgrade, no GPU path until a profile proves it

Realistic expectation is 5–20× slowdown versus native for CPU-bound workloads, with the exact ratio published in benchmarks once M2 is complete. The GAC benchmark (chunk-size vs codegen-time curve, GAC vs fixed-8192) is a required deliverable before the method is cited in the README.

---

## Contributing

The project is currently in private development. Issues and pull requests will open after v0.1.0 is published.

Before that point, bug reports about the placeholder v0.0.0 release are not accepted.

All contributions require a DCO sign-off (`git commit -s`). A CLA is not used and will not be accepted.

---

## License

Licensed under the **GNU General Public License v3.0 or later** ([LICENSE](LICENSE)).

This license is chosen deliberately: it prevents closed-source forks and paid forks of `ghostto`. Combined with DCO contributions, it makes the "free for everyone, forever" guarantee structural rather than promissory.

```
ghostto is free software: you can redistribute it and/or modify it
under the terms of the GNU General Public License as published by the
Free Software Foundation, either version 3 of the License, or (at your
option) any later version.
```

## Contribution

All contributions must be signed off under the Developer Certificate of Origin (DCO). Use `git commit -s`; your commit trailer is the certification. No Contributor License Agreement is used.
