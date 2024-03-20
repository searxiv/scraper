{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-23.11";
    flake-utils.url = "github:numtide/flake-utils";
    naersk = {
      url = "github:nix-community/naersk";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { self, nixpkgs, flake-utils, naersk, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustToolchain = pkgs.rust-bin.stable.latest.complete;
        buildInputs = with pkgs; [
          rustToolchain
          pkg-config
          poppler
        ];
        naersk' = pkgs.callPackage naersk { };
      in
      rec {
        packages.default = naersk'.buildPackage {
          inherit buildInputs;
          src = ./.;
        };

        devShells.default = pkgs.mkShell { inherit buildInputs; };
      });
}
