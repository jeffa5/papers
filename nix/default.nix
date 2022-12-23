{
  pkgs,
  crane,
  system,
}:
pkgs.lib.makeScope pkgs.newScope (self: let
  inherit (self) callPackage;
in rec {
  craneLib = crane.lib.${system};
  papers = self.callPackage ./papers.nix {};
})
