{ config, channels, lib, ... }: with channels.nixpkgs; with lib; let
  importShell = writeText "shell.nix" ''
    import ${builtins.unsafeDiscardStringContext config.shell.drvPath}
  '';
  cargo = name: command: ci.command {
    name = "cargo-${name}";
    command = ''
      nix-shell ${importShell} --run ${escapeShellArg ("cargo " + command)}
    '';
    impure = true;
  };
in {
  config = {
    name = "glib-signal.rs";
    ci.gh-actions.enable = true;
    cache.cachix.arc.enable = true;
    channels = {
      nixpkgs = "21.11";
      rust = "master";
    };
    environment = {
      test = {
        inherit (config.rustChannel.buildChannel) cargo;
      };
    };
    tasks = {
      build.inputs = [
        (cargo "build" "build")
        (cargo "build-futures" "build -F futures")
        (cargo "test" "test --workspace")
        (cargo "example-async" "run -p examples --bin async")
      ];
    };
  };

  options = {
    rustChannel = mkOption {
      type = types.unspecified;
      default = channels.rust.stable;
    };
    shell = mkOption {
      type = types.unspecified;
      default = config.rustChannel.mkShell {
        buildInputs = [ glib ];
        nativeBuildInputs = [ pkg-config ];
      };
    };
  };
}
