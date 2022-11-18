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
  outputs = { self, flakelib, nixpkgs, ... }@inputs: let
    nixlib = nixpkgs.lib;
    impure = builtins ? currentSystem;
  in flakelib {
    inherit inputs;
    systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
    devShells = {
      plain = {
        mkShell, hostPlatform
      , pkg-config
      , glib, libiconv
      , enableRust ? true, cargo
      , rustTools ? [ ]
      }: mkShell {
        inherit rustTools;
        buildInputs = [ glib ]
          ++ nixlib.optionals hostPlatform.isDarwin [ libiconv ];
        nativeBuildInputs = [ pkg-config ]
          ++ nixlib.optional enableRust cargo;
      };
      stable = { rust'stable, rust'latest, outputs'devShells'plain }: let
        stable = if impure then rust'stable else rust'latest;
      in outputs'devShells'plain.override {
        inherit (stable) mkShell;
        enableRust = false;
      };
      dev = { arc'rustPlatforms'nightly, outputs'devShells'plain }: outputs'devShells'plain.override {
        inherit (arc'rustPlatforms'nightly.hostChannel) mkShell;
        enableRust = false;
        rustTools = [ "rust-analyzer" "rustfmt" ];
      };
      default = { outputs'devShells }: outputs'devShells.plain;
    };
    checks = {
      rustfmt = { rustfmt, cargo, runCommand }: runCommand "cargo-fmt-check" {
        nativeBuildInputs = [ cargo (rustfmt.override { asNightly = true; }) ];
        src = self;
        meta.name = "cargo fmt";
      } ''
        cargo fmt --check \
          --manifest-path $src/Cargo.toml
        touch $out
      '';
      test = { outputs'devShells'plain, rustPlatform }: rustPlatform.buildRustPackage {
        pname = self.lib.cargoToml.package.name;
        inherit (self.lib.cargoToml.package) version;
        inherit (outputs'devShells'plain.override { enableRust = false; }) buildInputs nativeBuildInputs;
        cargoLock.lockFile = ./Cargo.lock;
        src = self;
        buildType = "debug";
        meta.name = "cargo test";
      };
      example-async = { outputs'devShells'plain, rustPlatform }: rustPlatform.buildRustPackage {
        pname = self.lib.cargoToml.package.name;
        inherit (self.lib.cargoToml.package) version;
        inherit (outputs'devShells'plain.override { enableRust = false; }) buildInputs nativeBuildInputs;
        cargoLock.lockFile = ./Cargo.lock;
        src = self;

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
    lib = with nixlib; {
      cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
      inherit (self.lib.cargoToml.package) version;
      releaseTag = "v${self.lib.version}";
    };
    config = rec {
      name = "ddcset-rs";
      packages.namespace = [ name ];
      inputs.arc = {
        lib.namespace = [ "arc" ];
        packages.namespace = [ "arc" ];
      };
    };
  };
}
