{ pkgs, inputs, ... }:
let
  toolchain = inputs.fenix.packages.${pkgs.system}.combine [
    inputs.fenix.packages.${pkgs.system}.stable.cargo
    inputs.fenix.packages.${pkgs.system}.stable.rustc
    inputs.fenix.packages.${pkgs.system}.stable.clippy
    inputs.fenix.packages.${pkgs.system}.stable.rustfmt
    inputs.fenix.packages.${pkgs.system}.stable.rust-src
  ];
in
pkgs.mkShell {
  packages = [
    toolchain
    pkgs.rust-analyzer
  ];
}
