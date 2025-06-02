#### Project Overview

This project uses the Rust GDExtension (via [godot-rust](https://godot-rust.github.io/)) to speed up fragment shader ↔ physics utilities in Godot 4.4.x.

#### Prerequisites

* Rust & Cargo (via [rustup](https://rustup.rs/))
* LLVM (needed for Web builds and `aubio-sys`)
* Emscripten SDK (for `wasm32-unknown-emscripten`)

#### Installing Rust & Cargo

##### Windows (PowerShell)

```powershell
winget install --id=Rustlang.Rustup -e
```

##### macOS (Homebrew or Nix)

```bash
nix profile install nixpkgs#rustup <- dumb lol
brew install rustup
rustup-init
source "$HOME/.cargo/env"
```

#### Installing LLVM (Windows)

Download and install LLVM 18.1.8:
[https://github.com/llvm/llvm-project/releases/download/llvmorg-18.1.8/LLVM-18.1.8-win64.exe](https://github.com/llvm/llvm-project/releases/download/llvmorg-18.1.8/LLVM-18.1.8-win64.exe)

#### Setup Commands

From the project root:

```bash
# Get nightly toolchain
rustup toolchain install nightly
# Add WebAssembly target
rustup target add wasm32-unknown-emscripten
```

#### Cargo Configuration

Add this to `rust/.cargo/config.toml`:

```toml
[target.wasm32-unknown-emscripten]
#linker = "emcc" <- if you installed with package manager, but package managers are dangerous with binaryen versions, see nix note
linker = "C:/Users/pl/emsdk/upstream/emscripten/emcc.bat" <- windows where ever you cloned the emsdk repo
rustflags = [
    "--verbose",
    "-C", "link-args=-g",
    "-C", "link-args=-sSIDE_MODULE=2",
    "-C", "link-args=-pthread",
    "-C", "target-feature=+atomics",
    "-Zlink-native-libraries=no",
    "-C", "link-args=-sDISABLE_EXCEPTION_CATCHING=1",
    "-C", "llvm-args=-enable-emscripten-cxx-exceptions=0"
]
```

> **Note for Nix users:**
> The Nix‐provided `emcc` (required for Godot’s Rust extension) is built against an older Binaryen (v120), which does **not** support the `--enable-bulk-memory-opt` flag needed for threads. To work around this:
>
> 1. Install a newer Binaryen (v121+) in your Nix profile (e.g. `nix profile install nixpkgs#binaryen`).
> 2. Copy the system‐wide `.emscripten` from
>    `/nix/store/...-emscripten-3.1.73/share/emscripten/.emscripten`
>    to `~/.emscripten`, and update its `BINARYEN_ROOT` to point at your v123 store path (e.g. `/nix/store/<hash>-binaryen-123`).
> 3. Before building for Web, run:
>
>    ```bash
>    export EM_CONFIG="$HOME/.emscripten"
>    ```
>
>    This forces `emcc` to use your v123 Binaryen (which understands bulk‐memory/thread flags) instead of the built‐in v120. Without this override, a release build will fail with “Unknown option ‘--enable-bulk-memory-opt’.”

#### Build Commands

##### Native (debug)
```bash
cd rust
cargo build --lib
```
##### Native (release)
```bash
cd rust
cargo build --lib --release
```
##### Rust only tests:
```bash
cd rust
cargo run --example tests --features tests-only
```

##### Web assembly/Emscripten (debug)
```bash
cd rust
cargo +nightly build -Zbuild-std --target wasm32-unknown-emscripten --lib
```
##### Web assembly/Emscripten (release) NIX NOTE MENTIONS THE EXACT VERSIONS NEEDED
```bash
cd rust
cargo +nightly build -Zbuild-std --target wasm32-unknown-emscripten --lib --release
```

The resulting `.wasm` and support files are in `rust/target/wasm32-unknown-emscripten`.

#### Formatting

Before committing, run (in the `bath/godot` directory):

```bash
gdformat --use-spaces=4 .
```
