{
  description = "Tmux session switcher";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        rustToolchain = pkgs.rust-bin.stable."1.86.0".default.override {
          extensions = [ "rust-src" "rustfmt" "clippy" "rust-analyzer" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            pkg-config
          ];
          
          shellHook = ''
            echo "Tmux session switcher development environment"
            echo "Rust toolchain: $(rustc --version)"
          '';
        };

        packages.default = pkgs.rustPlatform.buildRustPackage rec {
          pname = "tmux-session-switcher";
          version = "0.1.0";
          
          src = ./.;
          
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          
          nativeBuildInputs = with pkgs; [
            pkg-config
          ];
          
          meta = with pkgs.lib; {
            description = "Tmux session switcher";
            homepage = "https://github.com/bh/tmux-session-switcher";
            license = licenses.mit;
            maintainers = [ ];
          };
        };
      });
}