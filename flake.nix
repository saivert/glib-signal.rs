{
  description = "GObject signal bindings for Rust";
  inputs = {
    flakelib.url = "github:flakelib/fl";
    nixpkgs = { };
    rust = {
      url = "github:arcnmx/nixexprs-rust";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = { self, flakelib, nixpkgs, rust, ... }@inputs: let
    nixlib = nixpkgs.lib;
  in flakelib {
    inherit inputs;
    systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
    devShells = {
      plain = {
        mkShell, writeShellScriptBin, hostPlatform
      , pkg-config
      , glib, libiconv
      , enableRustdoc ? false
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
        RUSTDOCFLAGS = rust.lib.rustdocFlags {
          inherit (self.lib) crate;
          enableUnstableRustdoc = enableRustdoc;
          extern = {
            glib = let
              version = nixlib.versions.majorMinor self.lib.crate.dependencies.glib.version;
            in "https://gtk-rs.org/gtk-rs-core/stable/${version}/docs/";
          };
        };
      };
      stable = { rust'stable, outputs'devShells'plain }: outputs'devShells'plain.override {
        inherit (rust'stable) mkShell;
        enableRust = false;
      };
      dev = { rust'unstable, outputs'devShells'plain }: let
      in outputs'devShells'plain.override {
        inherit (rust'unstable) mkShell;
        enableRust = false;
        enableRustdoc = true;
        rustTools = [ "rust-analyzer" ];
      };
      default = { outputs'devShells }: outputs'devShells.plain;
    };
    checks = {
      rustfmt = { rust'builders, source }: rust'builders.check-rustfmt-unstable {
        src = source;
        config = ./.rustfmt.toml;
      };
      readme = { rust'builders, readme }: rust'builders.check-generate {
        expected = readme;
        src = ./src/README.md;
      };
      version = { rust'builders, source }: rust'builders.check-contents {
        src = source;
        patterns = [
          { path = "src/lib.rs"; docs'rs = {
            inherit (self.lib.crate.package) name version;
          }; }
        ];
      };
      test = { outputs'devShells'plain, rustPlatform, source }: rustPlatform.buildRustPackage {
        pname = self.lib.crate.package.name;
        inherit (self.lib.crate.package) version;
        inherit (outputs'devShells'plain.override { enableRust = false; }) buildInputs nativeBuildInputs;
        cargoLock.lockFile = ./Cargo.lock;
        src = source;
        buildType = "debug";
        meta.name = "cargo test";
      };
      docs = { docs }: docs;
      example-async = { outputs'devShells'plain, rustPlatform, source }: rustPlatform.buildRustPackage {
        pname = self.lib.crate.package.name;
        inherit (self.lib.crate.package) version;
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
      source = { rust'builders }: rust'builders.wrapSource self.lib.crate.src;

      readme = { rust'builders }: rust'builders.adoc2md {
        src = ./README.adoc;
        attributes = let
          inherit (self.lib) releaseTag;
          inherit (self.lib.crate.package) repository;
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

      docs = { rust'builders, outputs'devShells'plain, source }: let
        shell = outputs'devShells'plain.override { enableRust = false; enableRustdoc = true; };
      in rust'builders.cargoDoc {
        inherit (self.lib) crate;
        src = source;
        enableUnstableRustdoc = true;
        rustdocFlags = shell.RUSTDOCFLAGS;
        cargoDocFlags = [ "--no-deps" "--workspace" ];
        inherit (shell) buildInputs nativeBuildInputs;
      };
    } { };
    lib = with nixlib; {
      crate = rust.lib.importCargo ./Cargo.toml;
      inherit (self.lib.crate.package) version;
      releaseTag = "v${self.lib.version}";
    };
    config = rec {
      name = "glib-signal";
      packages.namespace = [ name ];
    };
  };
}
