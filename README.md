You need rust for this project to work:

```bash
# Linux (distro-independent)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows
winget install --id=Rustlang.Rustup -e

# macOS
brew install rustup
```

before opening godot, please install rust/rustup, and run the following in the projects `rust` directory:

```bash
cargo build
```

Before commiting please run:
```bash
gdformat --use-spaces=4 .
```

just for consistency between updates...



