#### Project Overview

This project uses the Rust GDExtension (via [godot-rust](https://godot-rust.github.io/)) to speed up fragment shader ↔ physics utilities in Godot 4.4.x.

#### New Settings

For editor experimentation only: an optional override config (`experimental_resolution_override.cfg`) lets you test different resolutions in the Godot editor without affecting runtime builds.

* **Manager script**: `godot/Autoloads/ResolutionManager.gd`

  * [https://github.com/meisei4/bath/blob/main/godot/Autoloads/ResolutionManager.gd](https://github.com/meisei4/bath/blob/main/godot/Autoloads/ResolutionManager.gd)
* **Override file**: `experimental_resolution_override.cfg`

  * [https://github.com/meisei4/bath/blob/main/experimental\_resolution\_override.cfg](https://github.com/meisei4/bath/blob/main/experimental_resolution_override.cfg)

This setup helps resolve inconsistent window sizes across scenes and devices during development.

#### Prerequisites

* Rust & Cargo (via [rustup](https://rustup.rs/))
* LLVM (needed for Web builds and `aubio-sys`)
* Emscripten SDK (for `wasm32-unknown-emscripten`)

#### Installing Rust & Cargo

##### Windows (PowerShell)

```powershell
winget install --id=Rustlang.Rustup -e
```

##### macOS (Homebrew)

```bash
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
linker = "emcc"
rustflags = [
  "-C", "link-args=-sSIDE_MODULE=2",
  "-Zlink-native-libraries=no",
  "-Cllvm-args=-enable-emscripten-cxx-exceptions=0",
]
```

#### Build Commands

##### Native (debug)

```bash
cd rust
cargo build --lib
```

##### Web (Emscripten)

```bash
cd rust
cargo +nightly build -Zbuild-std --target wasm32-unknown-emscripten --lib
```

The `.wasm` and support files are in `rust/target/wasm32-unknown-emscripten/debug`.

#### Formatting

Before committing, run (in the `bath/godot` directory:

```bash
gdformat --use-spaces=4 .
```

### NOTE
- non of the release build features of rust are working due to emcc linker issues, i will solve this much later, debug builds are perfectly fine for now
