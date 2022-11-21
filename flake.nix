{
  description = "GObject signal bindings for Rust";
  inputs = {
    flakelib.url = "github:flakelib/fl";
    nixpkgs = { };
    rust = {
      url = "github:arcnmx/nixexprs-rust";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    arc = {
      url = "github:arcnmx/nixexprs";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = { self, flakelib, nixpkgs, rust, ... }@inputs: let
    nixlib = nixpkgs.lib;
    impure = builtins ? currentSystem;
  in flakelib {
    inherit inputs;
    systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
    devShells = {
      plain = {
        mkShell, writeShellScriptBin, hostPlatform
      , pkg-config
      , glib, libiconv
      , enableRust ? true, cargo
      , rustTools ? [ ]
      }: mkShell {
        inherit rustTools;
        buildInputs = [ glib ]
          ++ nixlib.optionals hostPlatform.isDarwin [ libiconv ];
        nativeBuildInputs = [
          pkg-config
          (writeShellScriptBin "generate" ''nix run .#generate "$@"'')
        ] ++ nixlib.optional enableRust cargo;
      };
      stable = { rust'stable, outputs'devShells'plain }: outputs'devShells'plain.override {
        inherit (rust'stable) mkShell;
        enableRust = false;
      };
      dev = { arc'rustPlatforms'nightly, rust'distChannel, outputs'devShells'plain }: let
        channel = rust'distChannel {
          inherit (arc'rustPlatforms'nightly) channel date manifestPath;
        };
      in outputs'devShells'plain.override {
        inherit (channel) mkShell;
        enableRust = false;
        rustTools = [ "rust-analyzer" ];
      };
      default = { outputs'devShells }: outputs'devShells.plain;
    };
    checks = {
      rustfmt = { rust'builders, source }: rust'builders.check-rustfmt-unstable {
        src = source;
      };
      readme = { rust'builders, readme }: rust'builders.check-generate {
        expected = readme;
        src = ./src/README.md;
      };
      test = { outputs'devShells'plain, rustPlatform, source }: rustPlatform.buildRustPackage {
        pname = self.lib.cargoToml.package.name;
        inherit (self.lib.cargoToml.package) version;
        inherit (outputs'devShells'plain.override { enableRust = false; }) buildInputs nativeBuildInputs;
        cargoLock.lockFile = ./Cargo.lock;
        src = source;
        buildType = "debug";
        meta.name = "cargo test";
      };
      example-async = { outputs'devShells'plain, rustPlatform, source }: rustPlatform.buildRustPackage {
        pname = self.lib.cargoToml.package.name;
        inherit (self.lib.cargoToml.package) version;
        inherit (outputs'devShells'plain.override { enableRust = false; }) buildInputs nativeBuildInputs;
        cargoLock.lockFile = ./Cargo.lock;
        src = source;

        cargoBuildFlags = "--workspace --bin async";
        cargoTestFlags = "--workspace";
        buildType = "debug";

        doInstallCheck = true;
        installCheckPhase = ''
          $out/bin/async
        '';
        meta.name = "cargo test --workspace && cargo run -p examples --bin async";
      };
    };
    legacyPackages = { callPackageSet }: callPackageSet {
      source = { rust'builders }: rust'builders.wrapSource self.lib.cargoToml.src;

      readme = { rust'builders }: rust'builders.adoc2md {
        src = ./README.adoc;
        attributes = let
          inherit (self.lib) releaseTag;
          inherit (self.lib.cargoToml.package) repository;
        in {
          relative-tree = "${repository}/tree/${releaseTag}/";
          relative-blob = "${repository}/blob/${releaseTag}/";
        };
      };

      generate = { rust'builders, readme }: rust'builders.generateFiles {
        paths = {
          "src/README.md" = readme;
        };
      };
    } { };
    lib = with nixlib; {
      cargoToml = rust.lib.importCargo ./Cargo.toml;
      inherit (self.lib.cargoToml.package) version;
      releaseTag = "v${self.lib.version}";
      path = ./.;
      srcs = filesystem.listFilesRecursive ./src
      ++ filesystem.listFilesRecursive ./examples
      ++ [
        ./Cargo.toml
        ./Cargo.lock
        ./README.adoc
        ./.rustfmt.toml
      ];
    };
    config = rec {
      name = "glib-signal";
      packages.namespace = [ name ];
    };
  };
}
