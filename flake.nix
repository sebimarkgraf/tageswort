{
  description = "Flake for building the tageswort Rust binary with crane2nix";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane2nix.url = "github:ipetkov/crane";
  };

  outputs = { self, nixpkgs, flake-utils, crane2nix, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        craneLib = crane2nix.mkLib pkgs;

        commonArgs = {
          src = craneLib.cleanCargoSource ./.;
          buildInputs = [ pkgs.openssl ];
          nativeBuildInputs = [ pkgs.pkg-config ];
        };
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        tageswort = craneLib.buildPackage (commonArgs // {
          pname = "tageswort";
          version = "0.1.0";
          inherit cargoArtifacts;
        });

        checks = {
          fmt = craneLib.cargoFmt commonArgs;

          clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoExtraArgs = "--locked --all-features";
            cargoClippyExtraArgs = "--all-targets -- -D warnings";
          });

          test = craneLib.cargoTest (commonArgs // {
            inherit cargoArtifacts;
            cargoTestExtraArgs = "--all-targets";
          });
        };
      in
      {
        packages.default = tageswort;

        apps.default = flake-utils.lib.mkApp {
          drv = tageswort;
        };

        devShells.default = craneLib.devShell {
          checks = checks;
        };

        inherit checks;
      });
}
