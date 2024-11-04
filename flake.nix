{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, ... }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = function:
        nixpkgs.lib.genAttrs systems
        (system: function nixpkgs.legacyPackages.${system});
    in {
      overlays.default = final: prev: {
        rustToolchain =
          prev.rust-bin.rust.fromRustupToolchainFile ./rust-toolchain.toml;
      };

      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          packages = with pkgs; [
            cargo
            rustc
            wayland
            rust-analyzer-unwrapped
            rustfmt
            clippy
            pkg-config
            vulkan-loader
          ];
        };

        env = { RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}"; };
      });

      packages =
        forAllSystems (pkgs: { default = pkgs.callPackage ./default.nix { }; });
    };
}
