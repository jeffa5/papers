{
  description = "Papers";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.inputs.flake-utils.follows = "flake-utils";
    crane.url = "github:ipetkov/crane";
    crane.inputs.rust-overlay.follows = "rust-overlay";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    crane.inputs.flake-utils.follows = "flake-utils";
    flake-utils.url = "github:numtide/flake-utils";
  };

  nixConfig = {
    extra-substituters = [
      "https://papers.cachix.org"
    ];
    extra-trusted-public-keys = ["papers.cachix.org-1:XjQOqL1skswC0FgUn2xVE7Iu1fPr69ugIOdgKksu8eI="];
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    crane,
    flake-utils,
  }: let
    system = "x86_64-linux";
    pkgs = import nixpkgs {
      inherit system;
      overlays = [rust-overlay.overlays.default];
    };
    nix = import ./nix {inherit pkgs crane system;};
  in {
    packages.${system} =
      flake-utils.lib.filterPackages system nix // {default = self.packages.${system}.papers;};

    overlays.default = _final: _prev: self.packages.${system};

    apps.${system} = {
      papers = {
        type = "app";
        program = "${nix.papers}/bin/papers";
      };
      default = self.apps.${system}.papers;
    };

    checks.${system} = {
      papers = self.packages.${system}.papers;
    };

    formatter.${system} = pkgs.alejandra;

    devShells.${system}.default = pkgs.mkShell {
      buildInputs = with pkgs; [
        (rust-bin.stable.latest.default.override {
          extensions = ["rust-src"];
          targets = ["wasm32-unknown-unknown"];
        })
        cargo-edit
        cargo-fuzz
        cargo-make
        diesel-cli
        cargo-watch
        wasm-pack
        pkgconfig
        openssl

        sqlite
      ];
    };
  };
}
