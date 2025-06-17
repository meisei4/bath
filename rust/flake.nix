{
  description = "raylib dev env";

  inputs = {
    nixpkgs.url     = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            raylib
            pkg-config
            cmake
            rustc
            cargo
            # this doesnt actually work
            darwin.CarbonHeaders
            darwin.apple_sdk.frameworks.Cocoa
          ];

          shellHook = ''
            export RAYLIB_SYS_USE_PKG_CONFIG=1
            export PKG_CONFIG_PATH=${pkgs.raylib}/lib/pkgconfig:$PKG_CONFIG_PATH
            export GLFW_COCOA_RETINA_FRAMEBUFFER=0
          '';
        };
      });
}
