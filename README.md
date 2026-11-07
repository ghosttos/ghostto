# ghostto

[![Crates.io](https://img.shields.io/crates/v/ghostto.svg)](https://crates.io/crates/ghostto)
[![Downloads](https://img.shields.io/crates/d/ghostto.svg)](https://crates.io/crates/ghostto)
[![Docs.rs](https://img.shields.io/docsrs/ghostto)](https://docs.rs/ghostto)
[![CI](https://github.com/ghosttos/ghostto/actions/workflows/ci.yml/badge.svg)](https://github.com/ghosttos/ghostto/actions)
[![License](https://img.shields.io/crates/l/ghostto.svg)](#license)
[![MSRV](https://img.shields.io/badge/MSRV-1.80-blue.svg)](https://blog.rust-lang.org/)
[![Wasm](https://img.shields.io/badge/target-wasm32--unknown--unknown-orange.svg)](#webassembly)

> **Status:** `v0.0.0` — name reservation only. The engine is under active development. Watch this repository or the crates.io feed for `v0.1.0`.

**ghostto** is a from-scratch, memory-safe emulation engine written in Rust, designed to run full operating systems inside the browser via WebAssembly. It prioritizes speed, deterministic allocation, and a clean interpreter-first architecture—with optional GPU acceleration through `wgpu`.

---

## Table of Contents

- [What it is](#what-it-is)
- [Features](#features)
- [Architecture](#architecture)
- [Dependencies](#dependencies)
- [Installation](#installation)
- [Usage](#usage)
- [WebAssembly](#webassembly)
- [Roadmap](#roadmap)
- [Performance](#performance)
- [Contributing](#contributing)
- [License](#license)

---

## What it is

`ghostto` is a multi-OS browser emulator engine. It does not wrap QEMU, v86, CheerpX, or any existing emulator core. Every layer—instruction decoding, execution, memory management, devices, and display—is implemented from scratch in Rust.

The engine is being built to:

- **Run Linux** (x86_64, full-system) inside the browser
- **Run legacy Windows** (95–XP) via a custom minimal BIOS and legacy device set
- **Target modern hardware** through `wgpu`, which maps to Vulkan, Metal, and DX12 on native platforms and to WebGPU in the browser
- **Stay fast** by using a highly optimized pure interpreter, with an optional tiered Wasm JIT added only when measurement proves its worth

The name comes from the idea that the guest OS "haunts" the browser—present and running, without being native to it.

---

## Features

- **Pure Rust core** — no GC, no hidden allocations in hot paths, ownership-based memory safety
- **Interpreter-first** — a monolithic execution loop with zero heap allocation in the vCPU path
- **Deterministic allocation** — arena and slab allocators for the translation pipeline; zero `malloc` calls in steady-state execution
- **Optional tiered JIT** — emits WebAssembly modules for hot basic blocks only when profiling justifies it
- **`wgpu`-based GPU** — one backend targeting Vulkan, Metal, DX12, and browser WebGPU
- **SMP-ready design** — vCPU workers share guest RAM via `SharedArrayBuffer` and communicate through atomics
- **Memory-safe FFI** — all device models sit behind a documented C ABI; `unsafe` is confined to two audited modules
- **Wasm-native** — first-class `wasm32-unknown-unknown` target with `wasm-bindgen` bindings

---

## Architecture

`ghostto` is organized as a Cargo workspace. Each crate has a single responsibility:

| Crate | Purpose |
|---|---|
| `ghostto` | Umbrella facade crate |
| `ghostto_core` | Engine runtime: scheduler, block cache, vCPU orchestration |
| `ghostto_cpu` | x86_64 instruction decoder and interpreter |
| `ghostto_mm` | Guest RAM, software TLB, paging, page-table walkers |
| `ghostto_devices` | virtio-blk/net/console, IDE, VGA, PS/2, PCI |
| `ghostto_gpu` | `wgpu`-based virtual GPU backend |
| `ghostto_firmware` | Minimal BIOS / UEFI implementation |
| `ghostto_web` | `wasm32` entrypoint with `wasm-bindgen` bindings |
| `ghostto_cli` | Native command-line binary |

The engine never links a third-party emulator core. Every layer is owned by this project.

---

## Dependencies

`ghostto` deliberately keeps its dependency tree small. The core interpreter and MMU depend only on the Rust standard library.

### Runtime dependencies

| Crate | Purpose | Optional |
|---|---|---|
| `wgpu` | Cross-platform GPU backend (Vulkan/Metal/DX12/WebGPU) | ✅ Yes |
| `wasm-bindgen` | Browser bindings for the `wasm32` target | ✅ Yes (web only) |
| `wasm-encoder` | JIT emission of WebAssembly modules | ✅ Yes (JIT only) |

### Development dependencies

| Crate | Purpose |
|---|---|
| `criterion` | Benchmarks |
| `proptest` | Property-based testing of the decoder |
| `cargo-fuzz` | FFI and decoder fuzzing |

The interpreter and memory management layers have **zero external dependencies**.

---

## Installation

`ghostto` is not yet published as a usable library. The `v0.0.0` release on crates.io is a name reservation.

Once `v0.1.0` lands:

```toml
[dependencies]
ghostto = "0.1"
```

For the browser bundle, install the wasm package:

```bash
npm install @ghostto/engine
```

---

## Usage

Native (once v0.1.0 is available)

```rust
use ghostto::Engine;

fn main() -> ghostto::Result<()> {
    let mut engine = Engine::builder()
        .memory(256 * 1024 * 1024)   // 256 MiB guest RAM
        .vcpus(1)
        .build()?;

    engine.load_kernel("vmlinuz")?;
    engine.run()?;

    Ok(())
}
```

## Browser

```html
<script type="module">
  import init, { Engine } from "./ghostto_web.js";

  await init();
  const engine = new Engine({ memoryMb: 256 });
  await engine.loadKernel(fetch("vmlinuz").then(r => r.arrayBuffer()));
  await engine.run();
</script>
```

Browser deployments must set the following headers to enable SharedArrayBuffer:

```
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Embedder-Policy: require-corp
```

---

## WebAssembly

ghostto targets wasm32-unknown-unknown natively. The ghostto_web crate exposes an idiomatic JavaScript API through wasm-bindgen:

· Engine — the VM instance
· Engine.loadKernel(bytes) — load a Linux kernel image
· Engine.loadInitrd(bytes) — load an initramfs
· Engine.run() — start execution
· Engine.pause() / Engine.resume() — control flow
· Engine.snapshot() — serialize full VM state

The engine uses Web Workers for SMP vCPU execution and SharedArrayBuffer for guest RAM, which requires the COOP/COEP headers shown above.

---

## Roadmap

The project is developed in milestones, each gated by a concrete test:

| Milestone | Target | Gate |
|-----------|---------|------|
| M1 | Decode and execute 5 instructions (nop, hlt, mov imm64, mov r64, jmp) | Passing golden-register test |
| M2 | Run a static Linux user-mode binary | hello world from interpreter
| M3 | wasm32 target with wasm-bindgen bindings | Same binary runs in a browser
| M4 | Software MMU + TLB + page-table walkers | mmap-heavy static binary runs
| M5 | Full-system Linux: direct bzImage boot + virtio | BusyBox shell prompt
| M6 | SMP: 2 vCPUs via SharedArrayBuffer + atomics | nproc returns 2
| M7 | Interpreter optimization: inline TLB, lazy | EFLAGS MIPS vs M5 baseline measured
| M8 | Optional tiered Wasm JIT (measurement-gated) | Speedup proven against M7
| M9 Legacy Windows / classic Mac OS | Separate platform work

---

## Performance

ghostto is designed around the principle that a well-optimized interpreter can beat a poorly-integrated JIT in the browser. The project's performance strategy:

- Zero allocation in the vCPU loop, TLB fill, block dispatch, or device IRQ path
- Flat direct-mapped TLB with an inline fast path emitted into JIT output
- Lazy x86 EFLAGS computation—materialized only on branch or read
- Cache-friendly structure-of-arrays layouts throughout
- Paravirtualized devices (virtio) where the guest supports them

Realistic expectation is 5–20× slowdown versus native for CPU-bound workloads, with the exact ratio published in benchmarks once M2 is complete.

---

## Contributing

The project is currently in private development. Issues and pull requests will open after v0.1.0 is published.

Before that point, bug reports about the placeholder v0.0.0 release are not accepted.

---

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in ghostto by you, as defined in the Apache-2.0 license, shall be dual-licensed as above, without any additional terms or conditions.
