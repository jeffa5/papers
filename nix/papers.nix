{
  craneLib,
  pkg-config,
  openssl,
}: let
  src = craneLib.cleanCargoSource ./..;
  deps = craneLib.buildDepsOnly {
    inherit src;
    buildInputs = [pkg-config openssl];
  };
in
  craneLib.buildPackage {
    inherit src;
    cargoArtifacts = deps;
  }
