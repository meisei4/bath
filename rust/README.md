lol psycho but:
```bash
ann@anns-MBP rust % cargo tree
bath v0.1.0 (/Users/ann/bath/rust)
├── aubio-rs v0.2.0
│   └── aubio-sys v0.2.1
│       [build-dependencies]
│       └── cc v1.2.27
│           ├── jobserver v0.1.33
│           │   └── libc v0.2.173
│           ├── libc v0.2.173
│           └── shlex v1.3.0
├── beat-detector v0.1.0
│   ├── cpal v0.13.5
│   │   ├── core-foundation-sys v0.8.7
│   │   ├── coreaudio-rs v0.10.0
│   │   │   ├── bitflags v1.3.2
│   │   │   └── coreaudio-sys v0.2.17
│   │   │       [build-dependencies]
│   │   │       └── bindgen v0.72.0
│   │   │           ├── bitflags v2.9.1
│   │   │           ├── cexpr v0.6.0
│   │   │           │   └── nom v7.1.3
│   │   │           │       ├── memchr v2.7.5
│   │   │           │       └── minimal-lexical v0.2.1
│   │   │           ├── clang-sys v1.8.1
│   │   │           │   ├── glob v0.3.2
│   │   │           │   ├── libc v0.2.173
│   │   │           │   └── libloading v0.8.8
│   │   │           │       └── cfg-if v1.0.1
│   │   │           │   [build-dependencies]
│   │   │           │   └── glob v0.3.2
│   │   │           ├── itertools v0.13.0
│   │   │           │   └── either v1.15.0
│   │   │           ├── proc-macro2 v1.0.95
│   │   │           │   └── unicode-ident v1.0.18
│   │   │           ├── quote v1.0.40
│   │   │           │   └── proc-macro2 v1.0.95 (*)
│   │   │           ├── regex v1.11.1
│   │   │           │   ├── regex-automata v0.4.9
│   │   │           │   │   └── regex-syntax v0.8.5
│   │   │           │   └── regex-syntax v0.8.5
│   │   │           ├── rustc-hash v2.1.1
│   │   │           ├── shlex v1.3.0
│   │   │           └── syn v2.0.103
│   │   │               ├── proc-macro2 v1.0.95 (*)
│   │   │               ├── quote v1.0.40 (*)
│   │   │               └── unicode-ident v1.0.18
│   │   ├── mach v0.3.2
│   │   │   └── libc v0.2.173
│   │   └── thiserror v1.0.69
│   │       └── thiserror-impl v1.0.69 (proc-macro)
│   │           ├── proc-macro2 v1.0.95 (*)
│   │           ├── quote v1.0.40 (*)
│   │           └── syn v2.0.103 (*)
│   ├── lowpass-filter v0.2.5
│   └── spectrum-analyzer v0.5.2
│       ├── float-cmp v0.8.0
│       │   └── num-traits v0.2.19
│       │       [build-dependencies]
│       │       └── autocfg v1.4.0
│       ├── libm v0.2.15
│       └── microfft v0.4.0
│           ├── num-complex v0.4.6
│           │   └── num-traits v0.2.19 (*)
│           └── static_assertions v1.1.0
├── godot v0.3.1
│   ├── godot-core v0.3.1
│   │   ├── glam v0.30.4
│   │   ├── godot-cell v0.3.1
│   │   └── godot-ffi v0.3.1
│   │       [build-dependencies]
│   │       ├── godot-bindings v0.3.1
│   │       │   └── gdextension-api v0.2.2
│   │       └── godot-codegen v0.3.1
│   │           ├── godot-bindings v0.3.1 (*)
│   │           ├── heck v0.5.0
│   │           ├── nanoserde v0.2.1
│   │           │   └── nanoserde-derive v0.2.1 (proc-macro)
│   │           ├── proc-macro2 v1.0.95 (*)
│   │           ├── quote v1.0.40 (*)
│   │           └── regex v1.11.1 (*)
│   │           [build-dependencies]
│   │           └── godot-bindings v0.3.1 (*)
│   │   [build-dependencies]
│   │   ├── godot-bindings v0.3.1 (*)
│   │   └── godot-codegen v0.3.1 (*)
│   └── godot-macros v0.3.1 (proc-macro)
│       ├── proc-macro2 v1.0.95 (*)
│       ├── quote v1.0.40 (*)
│       └── venial v0.6.1
│           ├── proc-macro2 v1.0.95 (*)
│           └── quote v1.0.40 (*)
│       [build-dependencies]
│       └── godot-bindings v0.3.1 (*)
├── hound v3.5.1
├── lewton v0.10.2
│   ├── byteorder v1.5.0
│   ├── ogg v0.8.0
│   │   └── byteorder v1.5.0
│   └── tinyvec v1.9.0
│       └── tinyvec_macros v0.1.1
├── midly v0.5.3
│   └── rayon v1.10.0
│       ├── either v1.15.0
│       └── rayon-core v1.12.1
│           ├── crossbeam-deque v0.8.6
│           │   ├── crossbeam-epoch v0.9.18
│           │   │   └── crossbeam-utils v0.8.21
│           │   └── crossbeam-utils v0.8.21
│           └── crossbeam-utils v0.8.21
└── rustysynth v1.3.5
[dev-dependencies]
└── raylib v5.5.1
    ├── cfg-if v1.0.1
    ├── paste v1.0.15 (proc-macro)
    ├── raylib-sys v5.5.1
    │   [build-dependencies]
    │   ├── bindgen v0.70.1
    │   │   ├── bitflags v2.9.1
    │   │   ├── cexpr v0.6.0 (*)
    │   │   ├── clang-sys v1.8.1 (*)
    │   │   ├── itertools v0.13.0 (*)
    │   │   ├── log v0.4.27
    │   │   ├── prettyplease v0.2.34
    │   │   │   ├── proc-macro2 v1.0.95 (*)
    │   │   │   └── syn v2.0.103 (*)
    │   │   ├── proc-macro2 v1.0.95 (*)
    │   │   ├── quote v1.0.40 (*)
    │   │   ├── regex v1.11.1 (*)
    │   │   ├── rustc-hash v1.1.0
    │   │   ├── shlex v1.3.0
    │   │   └── syn v2.0.103 (*)
    │   ├── cc v1.2.27 (*)
    │   └── cmake v0.1.54
    │       └── cc v1.2.27 (*)
    ├── seq-macro v0.3.6 (proc-macro)
    └── thiserror v1.0.69 (*)
ann@anns-MBP rust % cargo build
   Compiling proc-macro2 v1.0.95
   Compiling unicode-ident v1.0.18
   Compiling regex-syntax v0.8.5
   Compiling libc v0.2.173
   Compiling godot-bindings v0.3.1
   Compiling gdextension-api v0.2.2
   Compiling quote v1.0.40
   Compiling shlex v1.3.0
   Compiling glob v0.3.2
   Compiling regex-automata v0.4.9
   Compiling either v1.15.0
   Compiling clang-sys v1.8.1
   Compiling godot-codegen v0.3.1
   Compiling syn v2.0.103
   Compiling cfg-if v1.0.1
   Compiling minimal-lexical v0.2.1
   Compiling nanoserde-derive v0.2.1
   Compiling memchr v2.7.5
   Compiling nom v7.1.3
   Compiling regex v1.11.1
   Compiling nanoserde v0.2.1
   Compiling libloading v0.8.8
   Compiling heck v0.5.0
   Compiling autocfg v1.4.0
   Compiling bindgen v0.72.0
   Compiling cexpr v0.6.0
   Compiling num-traits v0.2.19
   Compiling itertools v0.13.0
   Compiling bitflags v2.9.1
   Compiling crossbeam-utils v0.8.21
   Compiling rustc-hash v2.1.1
   Compiling jobserver v0.1.33
   Compiling cc v1.2.27
   Compiling godot-ffi v0.3.1
   Compiling crossbeam-epoch v0.9.18
   Compiling rayon-core v1.12.1
   Compiling thiserror v1.0.69
   Compiling libm v0.2.15
   Compiling crossbeam-deque v0.8.6
   Compiling aubio-sys v0.2.1
   Compiling num-complex v0.4.6
   Compiling godot-core v0.3.1
   Compiling thiserror-impl v1.0.69
   Compiling godot-macros v0.3.1
   Compiling cpal v0.13.5
   Compiling bitflags v1.3.2
   Compiling static_assertions v1.1.0
   Compiling microfft v0.4.0
   Compiling mach v0.3.2
   Compiling coreaudio-sys v0.2.17
   Compiling float-cmp v0.8.0
   Compiling venial v0.6.1
   Compiling byteorder v1.5.0
   Compiling godot-cell v0.3.1
   Compiling glam v0.30.4
   Compiling core-foundation-sys v0.8.7
   Compiling tinyvec_macros v0.1.1
   Compiling tinyvec v1.9.0
   Compiling ogg v0.8.0
   Compiling spectrum-analyzer v0.5.2
   Compiling rayon v1.10.0
   Compiling lowpass-filter v0.2.5
   Compiling aubio-rs v0.2.0
   Compiling coreaudio-rs v0.10.0
   Compiling lewton v0.10.2
   Compiling beat-detector v0.1.0
   Compiling hound v3.5.1
   Compiling rustysynth v1.3.5
   Compiling midly v0.5.3
   Compiling godot v0.3.1
   Compiling bath v0.1.0 (/Users/ann/bath/rust)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4m 13s
ann@anns-MBP rust % printenv
COMMAND_MODE=unix2003
TERM_SESSION_ID=8b8dc58b-154f-457e-833d-8765e905ca16
LC_CTYPE=UTF-8
SHELL=/bin/zsh
__CFBundleIdentifier=com.jetbrains.rustrover
TMPDIR=/var/folders/0m/b455psh97113ml9p6gkl4hfm0000gn/T/
HOME=/Users/ann
SSH_AUTH_SOCK=/private/tmp/com.apple.launchd.e0FhcZNJuf/Listeners
PATH=/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:/Users/ann/.nix-profile/bin:/nix/var/nix/profiles/default/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:/Library/Apple/usr/bin
XPC_SERVICE_NAME=0
TERM=xterm-256color
LOGNAME=ann
USER=ann
XPC_FLAGS=0x0
__CF_USER_TEXT_ENCODING=0x1F5:0x0:0x0
TERMINAL_EMULATOR=JetBrains-JediTerm
SHLVL=1
PWD=/Users/ann/bath/rust
OLDPWD=/Users/ann/bath/rust
__ETC_PROFILE_NIX_SOURCED=1
NIX_PROFILES=/nix/var/nix/profiles/default /Users/ann/.nix-profile
XDG_DATA_DIRS=/usr/local/share:/usr/share:/Users/ann/.nix-profile/share:/nix/var/nix/profiles/default/share
NIX_SSL_CERT_FILE=/nix/var/nix/profiles/default/etc/ssl/certs/ca-bundle.crt
EM_CACHE=/Users/ann/.emscripten_cache
EM_CONFIG=/Users/ann/.emscripten
PKG_CONFIG_PATH=/usr/local/lib/pkgconfig:
RAYLIB_SYS_USE_PKG_CONFIG=1
GLFW_COCOA_RETINA_FRAMEBUFFER=0
_=/usr/bin/printenv
ann@anns-MBP rust % which cc
/usr/bin/cc
ann@anns-MBP rust % which ar
/usr/bin/ar
ann@anns-MBP rust % which ld
/usr/bin/ld
ann@anns-MBP rust % which pkg-config
/usr/local/bin/pkg-config
ann@anns-MBP rust % which cmake     
/usr/local/bin/cmake
ann@anns-MBP rust % brew list
==> Formulae
cmake   pkgconf raylib

==> Casks
midikeys        polyphone       qgis            wine-stable
ann@anns-MBP rust % which pkgconf
/usr/local/bin/pkgconf
ann@anns-MBP rust % which zig
/Users/ann/.nix-profile/bin/zig
ann@anns-MBP rust % pkg-config --modversion raylib
5.5.0
ann@anns-MBP rust % nix profile list
Name:               binaryen
Flake attribute:    legacyPackages.x86_64-darwin.binaryen
Original flake URL: flake:nixpkgs
Locked flake URL:   github:NixOS/nixpkgs/fe51d34885f7b5e3e7b59572796e1bcb427eccb1?narHash=sha256-qmmFCrfBwSHoWw7cVK4Aj%2Bfns%2Bc54EBP8cGqp/yK410%3D
Store paths:        /nix/store/jwvf9bgxziibaa6nmi5zmd1d1s54lj9s-binaryen-123

Name:               emscripten
Flake attribute:    legacyPackages.x86_64-darwin.emscripten
Original flake URL: flake:nixpkgs/21808d22b1cda1898b71cf1a1beb524a97add2c4
Locked flake URL:   github:NixOS/nixpkgs/21808d22b1cda1898b71cf1a1beb524a97add2c4?narHash=sha256-j4HeaLw1LZxkCvuOxdO1xTnPYLSOQuzOjGEuCK80X2w%3D
Store paths:        /nix/store/zgad5m5blynzq79fsci4mplp6mw9yyg0-emscripten-3.1.73

Name:               ffmpeg
Flake attribute:    legacyPackages.x86_64-darwin.ffmpeg
Original flake URL: flake:nixpkgs
Locked flake URL:   github:NixOS/nixpkgs/8c441601c43232976179eac52dde704c8bdf81ed?narHash=sha256-q2PmaOxyR3zqOF54a3E1Cj1gh0sDu8APX9b%2BOkX4J5s%3D
Store paths:        /nix/store/1f6dxlw9fnms9zgbw3xz96dxkzvm5x6v-ffmpeg-7.1.1-bin /nix/store/b50yaby83nfmiiqpgajbakfwi8b5l3h0-ffmpeg-7.1.1-man

Name:               fluidsynth
Flake attribute:    legacyPackages.x86_64-darwin.fluidsynth
Original flake URL: flake:nixpkgs
Locked flake URL:   github:NixOS/nixpkgs/8c441601c43232976179eac52dde704c8bdf81ed?narHash=sha256-q2PmaOxyR3zqOF54a3E1Cj1gh0sDu8APX9b%2BOkX4J5s%3D
Store paths:        /nix/store/flbj5lmjqfzdg6arzim3asw6l50jzl0y-fluidsynth-2.4.4 /nix/store/zwar80kgi3bqpzjwhbnh3bs1zw9xgy19-fluidsynth-2.4.4-man

Name:               gcc
Flake attribute:    legacyPackages.x86_64-darwin.gcc
Original flake URL: flake:nixpkgs
Locked flake URL:   github:NixOS/nixpkgs/8c441601c43232976179eac52dde704c8bdf81ed?narHash=sha256-q2PmaOxyR3zqOF54a3E1Cj1gh0sDu8APX9b%2BOkX4J5s%3D
Store paths:        /nix/store/kpk57xjwgb2kkbidrvhj4mk0zifj7jkp-gcc-wrapper-14.2.1.20250322 /nix/store/wdrsjsi6020c883n5bd4j0ih3wqf65da-gcc-wrapper-14.2.1.20250322-man

Name:               gdtoolkit_4
Flake attribute:    legacyPackages.x86_64-darwin.gdtoolkit_4
Original flake URL: flake:nixpkgs
Locked flake URL:   github:NixOS/nixpkgs/8c441601c43232976179eac52dde704c8bdf81ed?narHash=sha256-q2PmaOxyR3zqOF54a3E1Cj1gh0sDu8APX9b%2BOkX4J5s%3D
Store paths:        /nix/store/sp1yk45ax1iwmwpq35mxf4dyd4dnq7db-gdtoolkit-4.3.3

Name:               git
Flake attribute:    legacyPackages.x86_64-darwin.git
Original flake URL: flake:nixpkgs
Locked flake URL:   github:NixOS/nixpkgs/8c441601c43232976179eac52dde704c8bdf81ed?narHash=sha256-q2PmaOxyR3zqOF54a3E1Cj1gh0sDu8APX9b%2BOkX4J5s%3D
Store paths:        /nix/store/38kaic6dax0nkj4xwnyysmlwzdlrfdfr-git-2.49.0

Name:               mpv
Flake attribute:    legacyPackages.x86_64-darwin.mpv
Original flake URL: flake:nixpkgs
Locked flake URL:   github:NixOS/nixpkgs/8c441601c43232976179eac52dde704c8bdf81ed?narHash=sha256-q2PmaOxyR3zqOF54a3E1Cj1gh0sDu8APX9b%2BOkX4J5s%3D
Store paths:        /nix/store/nqakrq5xigsi5zbxnrvn6as8rxwfq424-mpv-with-scripts-0.40.0

Name:               neofetch
Flake attribute:    legacyPackages.x86_64-darwin.neofetch
Original flake URL: flake:nixpkgs
Locked flake URL:   github:NixOS/nixpkgs/41da1e3ea8e23e094e5e3eeb1e6b830468a7399e?narHash=sha256-jp0D4vzBcRKwNZwfY4BcWHemLGUs4JrS3X9w5k/JYDA%3D
Store paths:        /nix/store/nh89kq3a75mjqdlnq8mnm73jp783dhlg-neofetch-unstable-2021-12-10 /nix/store/s5vs1yfq34xwycpwkq0cj4yzgspivwna-neofetch-unstable-2021-12-10-man

Name:               qemu
Flake attribute:    legacyPackages.x86_64-darwin.qemu
Original flake URL: flake:nixpkgs
Locked flake URL:   github:NixOS/nixpkgs/e314d5c6d3b3a0f40ec5bcbc007b0cbe412f48ae?narHash=sha256-IlAuXnIi%2BZmyS89tt1YOFDCv7FKs9bNBHd3MXMp8PxE%3D
Store paths:        /nix/store/rkp7480jvg15nnp08396ndk72r0q9sn3-qemu-9.2.3

Name:               rectangle
Flake attribute:    legacyPackages.x86_64-darwin.rectangle
Original flake URL: flake:nixpkgs
Locked flake URL:   github:NixOS/nixpkgs/8c441601c43232976179eac52dde704c8bdf81ed?narHash=sha256-q2PmaOxyR3zqOF54a3E1Cj1gh0sDu8APX9b%2BOkX4J5s%3D
Store paths:        /nix/store/25g8xwrfqqjzqg0ps6034f175f8yyldw-rectangle-0.87

Name:               rustup
Flake attribute:    legacyPackages.x86_64-darwin.rustup
Original flake URL: flake:nixpkgs
Locked flake URL:   github:NixOS/nixpkgs/a16efe5d2fc7455d7328a01f4692bfec152965b3?narHash=sha256-rSuxACdwx5Ndr2thpjqcG89fj8mSSp96CFoCt0yrdkY%3D
Store paths:        /nix/store/3ybgkqizd7h1z3v1m1q3y8acs309c78n-rustup-1.28.2

Name:               tree
Flake attribute:    legacyPackages.x86_64-darwin.tree
Original flake URL: flake:nixpkgs
Locked flake URL:   github:NixOS/nixpkgs/8c441601c43232976179eac52dde704c8bdf81ed?narHash=sha256-q2PmaOxyR3zqOF54a3E1Cj1gh0sDu8APX9b%2BOkX4J5s%3D
Store paths:        /nix/store/hz7yr86j6j1dxa4kfbskaksrbcyhdcxm-tree-2.2.1

Name:               wget
Flake attribute:    legacyPackages.x86_64-darwin.wget
Original flake URL: flake:nixpkgs
Locked flake URL:   github:NixOS/nixpkgs/41da1e3ea8e23e094e5e3eeb1e6b830468a7399e?narHash=sha256-jp0D4vzBcRKwNZwfY4BcWHemLGUs4JrS3X9w5k/JYDA%3D
Store paths:        /nix/store/vi8028rmawlzww76dc9k1fx8iyvk3nhb-wget-1.25.0

Name:               yt-dlp
Flake attribute:    legacyPackages.x86_64-darwin.yt-dlp
Original flake URL: flake:nixpkgs
Locked flake URL:   github:NixOS/nixpkgs/8c441601c43232976179eac52dde704c8bdf81ed?narHash=sha256-q2PmaOxyR3zqOF54a3E1Cj1gh0sDu8APX9b%2BOkX4J5s%3D
Store paths:        /nix/store/0f2kirqd18gzn56jsvvnjgh862flrdq3-yt-dlp-2025.4.30

Name:               zig
Flake attribute:    legacyPackages.x86_64-darwin.zig
Original flake URL: flake:nixpkgs
Locked flake URL:   github:NixOS/nixpkgs/c539ae8d21e49776966d714f82fba33b1fca78bc?narHash=sha256-zcGClfkXh4pckf4aGOZ18GFv73n1xHbdMWl17cPLouE%3D
Store paths:        /nix/store/n00cnc0aly9ymd3v9wn3q3g918bg9w6l-zig-0.14.1
ann@anns-MBP rust % zig env
{
 "zig_exe": "/nix/store/n00cnc0aly9ymd3v9wn3q3g918bg9w6l-zig-0.14.1/bin/zig",
 "lib_dir": "/nix/store/n00cnc0aly9ymd3v9wn3q3g918bg9w6l-zig-0.14.1/lib/zig",
 "std_dir": "/nix/store/n00cnc0aly9ymd3v9wn3q3g918bg9w6l-zig-0.14.1/lib/zig/std",
 "global_cache_dir": "/Users/ann/.cache/zig",
 "version": "0.14.1",
 "target": "x86_64-macos.12.7.6...12.7.6-none",
 "env": {
  "ZIG_GLOBAL_CACHE_DIR": null,
  "ZIG_LOCAL_CACHE_DIR": null,
  "ZIG_LIB_DIR": null,
  "ZIG_LIBC": null,
  "ZIG_BUILD_RUNNER": null,
  "ZIG_VERBOSE_LINK": null,
  "ZIG_VERBOSE_CC": null,
  "ZIG_BTRFS_WORKAROUND": null,
  "ZIG_DEBUG_CMD": null,
  "CC": null,
  "NO_COLOR": null,
  "CLICOLOR_FORCE": null,
  "XDG_CACHE_HOME": null,
  "HOME": "/Users/ann"
 }
}
```