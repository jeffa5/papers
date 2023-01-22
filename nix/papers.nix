{
  craneLib,
  pkg-config,
  openssl,
  lib,
  sqlite,
  installShellFiles,
}: let
  migrationsFilter = path: _type: builtins.match ".*/migrations/.*$" path != null;
  cargoFilter = craneLib.filterCargoSources;
  srcFilter = path: type: builtins.any (f: f path type) [cargoFilter migrationsFilter];
  src = lib.cleanSourceWith {
    src = ./..;
    filter = srcFilter;
  };
  pname = "papers";
  deps = craneLib.buildDepsOnly {
    inherit src pname;
    buildInputs = [sqlite pkg-config openssl];
  };
in
  craneLib.buildPackage {
    inherit src pname;
    cargoArtifacts = deps;
    buildInputs = [sqlite installShellFiles];
    installPhaseCommand = ''
      if [ -n "$cargoBuildLog" -a -f "$cargoBuildLog" ]; then
        installFromCargoBuildLog "$out" "$cargoBuildLog"
        installShellCompletion target/release/build/papers-*/out/share/papers.{bash,fish}
        installShellCompletion --zsh target/release/build/papers-*/out/share/_papers
      else
        false
      fi
    '';
  }
