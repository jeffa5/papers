{
  description = "Papers";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
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
    crane,
    flake-utils,
  }: let
    system = "x86_64-linux";
    pkgs = import nixpkgs {inherit system;};
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
      packages = with pkgs; [
        rustc
        cargo
        rustfmt

        pkg-config
        openssl
      ];
    };
  };
}
