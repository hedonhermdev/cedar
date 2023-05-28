{
  description = "Cedar: Simple in memory embeddings in Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, crane, ... }:
    let
      # System types to support.
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" ];

      # Rust nightly version.
      nightlyVersion = "2022-12-04";
    in
    flake-utils.lib.eachSystem supportedSystems (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
		  config = {
		  	allowUnfree = true;
			cudaSupport = true;
		  };
        };

        rustNightly = pkgs.rust-bin.nightly.${nightlyVersion}.default.override {
          extensions = [ "rust-src" "rust-analyzer-preview" ];
          targets = [ "x86_64-unknown-linux-gnu" ];
        };
        craneLib = (crane.mkLib pkgs).overrideToolchain rustNightly;

        cedarPackage = craneLib.buildPackage {
          inherit src;
          nativeBuildInputs = with pkgs; [
            rustNightly
            pkg-config
            openssl
            python310Packages.torch
            duckdb
          ];
          doCheck = true;
        };

        LD_LIBRARY_PATH = "${pkgs.python310Packages.torch}/lib/python3.10/site-packages/torch";
        src = ./.;
      in
      {
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            rustNightly
            pkg-config
            openssl
            python310Packages.torch
            duckdb
			cudatoolkit
          ];
          LD_LIBRARY_PATH = "${pkgs.python310Packages.torch}/lib/python3.10/site-packages/torch";
        };

        packages = {
          default = cedarPackage;
          cedar = cedarPackage;
        };
      });
}
