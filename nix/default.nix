{
  pkgs,
  crane,
  system,
}:
pkgs.lib.makeScope pkgs.newScope (self: {
  craneLib = crane.lib.${system};
  papers = self.callPackage ./papers.nix {};
})
