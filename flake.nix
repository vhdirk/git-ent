{
  outputs =
    {
      self,
      nixpkgs,
      flake-parts,
      rust-overlay,
      ...
    }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } (
      { ... }:
      {
        systems = nixpkgs.lib.systems.flakeExposed;

        imports = [
          inputs.treefmt-nix.flakeModule
          inputs.devenv.flakeModule
          inputs.flake-parts.flakeModules.easyOverlay
        ];

        perSystem =
          {
            config,
            pkgs,
            system,
            ...
          }:
          {
            overlayAttrs = {
              inherit (config.packages) git-ent;
            };

            packages = rec {
              default = git-ent;
              git-ent = import ./nix/package.nix {
                inherit
                  pkgs
                  inputs
                  self
                  system
                  ;
              };
            };

            treefmt = {
              programs.rustfmt.enable = true;
            };

            devenv.shells.default = {
              git-hooks = {
                enable = true;
                hooks = {
                  nix-treefmt = {
                    enable = true;
                    name = "Nix Format";

                    language = "system";
                    pass_filenames = true;

                    entry = "${self.formatter."${pkgs.stdenv.hostPlatform.system}"}/bin/treefmt";
                    args = [ "--fail-on-change" ];
                  };

                  clippy = {
                    enable = true;
                    settings = {
                      allFeatures = true;
                    };
                  };
                };
              };

              languages.rust = {
                enable = true;
                channel = "nixpkgs";
              };

              packages = with pkgs; [
                gcc
                pkg-config
                mdbook

                libgit2.dev
                openssl.dev

                cargo-tarpaulin
                cargo-expand
                # config.packages.git-ent
              ];

              env.RUST_LOG = "debug";
            };
          };
      }
    );

  nixConfig = {
    keep-outputs = true;
    keep-derivations = true;
    extra-substituters = [
      "https://nix-community.cachix.org"
    ];
    extra-trusted-public-keys = [
      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
    ];
  };

  inputs = {
    nixpkgs = {
      url = "github:nixos/nixpkgs/nixos-unstable";
    };

    flake-parts.url = "github:hercules-ci/flake-parts";
    devenv = {
      url = "github:cachix/devenv";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
}
