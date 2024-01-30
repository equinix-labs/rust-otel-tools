{
  inputs = {
    # maybe update .envrc (see comment in file) when devenv is bumped
    devenv.url = "github:cachix/devenv/v0.6.3";
    devenv.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };
  outputs = {
    devenv,
    flake-utils,
    nixpkgs,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: {
      devShell = nixpkgs.legacyPackages.${system}.mkShellNoCC {
        buildInputs = [
          nixpkgs.legacyPackages.${system}.git
          devenv.packages.${system}.devenv
        ];
      };
    });
}
