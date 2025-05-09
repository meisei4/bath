##NEW SETTINGS THINGS:
## in order to centralize resolution settings i have introduced an optional project settings override config file, read about it to learn about it plox:
Best description of the issue i was trying to solve (caused by different scene contexts, and even different device screen sizes causing ugly behavior with the window sizes):
https://github.com/meisei4/bath/blob/main/godot/Autoloads/ResolutionManager.gd
new file:
https://github.com/meisei4/bath/blob/main/experimental_resolution_override.cfg

This project uses the Rust GDExtension (https://godot-rust.github.io/) to accelerate compute‐shader <-> physics utilities in Godot 4.4.x.
Before opening the project, make sure you have Rust and Cargo installed so that the extension can be built.
## 1. Install Rust & Cargo
### Windows (winget)
```powershell
winget install --id=Rustlang.Rustup -e
```

### Linux (any distro)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

### macOS (Homebrew)
```bash
brew install rustup
rustup-init
source "$HOME/.cargo/env"
```

## 2. Build the Rust GDExtension
1. Change into the `rust` directory (IN THIS PROJECT, like the actual rust directory in this git repo):
    ```bash
    cd rust
    ```
2. For a debug build (default):
    ```bash
    cargo build
    ```
    
> The compiled dynamic library (`.dll` / `.so` / `.dylib`) will be placed in `rust/target/{debug,release}` and is automatically referenced by `rust_bath.gdextension`.

## 3. Open in Godot
1. Launch Godot 4 (MUST BE IN FORWARD+, compatibility is not supported yet because of compute shaders)
2. Open `project.godot` at the project root.
3. The GDExtension will load the Rust library on startup.


## 4. Code Formatting
Before committing any changes, run this in the bath main project directory:
```bash
gdformat --use-spaces=4 .
```
This keeps GDScript files formatted consistently across the repo.
