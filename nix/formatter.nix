{ pkgs, inputs, ... }:
(inputs.treefmt-nix.lib.evalModule pkgs {
  projectRootFile = "flake.nix";

  programs.deadnix.enable = true;
  programs.deadnix.no-lambda-pattern-names = true;
  settings.formatter.deadnix.priority = 1;

  programs.statix.enable = true;
  settings.formatter.statix.priority = 2;

  programs.nixfmt.enable = true;
  settings.formatter.nixfmt.priority = 3;

  programs.rustfmt.enable = true;
  programs.taplo.enable = true;
}).config.build.wrapper
