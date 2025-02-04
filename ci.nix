{ config, channels, lib, ... }: with channels.nixpkgs; with lib; let
  inherit (import ./. { inherit pkgs; }) checks;
in {
  config = {
    name = "glib-signal.rs";
    ci.gh-actions.enable = true;
    cache.cachix = {
      ci.signingKey = "";
      arc.enable = true;
    };
    channels = {
      nixpkgs = "22.11";
    };
    tasks = {
      rustfmt.inputs = singleton checks.rustfmt;
      readme.inputs = singleton checks.readme;
      version.inputs = singleton checks.version;
      build.inputs = singleton checks.test;
      example.inputs = [
        checks.example-async
      ];
      docs.inputs = [
        checks.docs
      ];
    };
  };
}
