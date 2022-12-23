{
  craneLib,
  pkg-config,
  openssl,
  lib,
  sqlite,
}: let
  migrationsFilter = path: _type: builtins.match ".*/migrations/.*$" path != null;
  cargoFilter = craneLib.filterCargoSources;
  srcFilter = path: type: builtins.any (f: f path type) [cargoFilter migrationsFilter];
  src = lib.cleanSourceWith {
    src = ./..;
    filter = srcFilter;
  };
  deps = craneLib.buildDepsOnly {
    inherit src;
    buildInputs = [sqlite pkg-config openssl];
  };
in
  craneLib.buildPackage {
    inherit src;
    cargoArtifacts = deps;
    buildInputs = [sqlite];
  }
