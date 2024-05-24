{
  description = "A Nix-flake-based development environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = { self , nixpkgs, ... }: let
    system = "x86_64-linux";
  in {
    devShells."${system}".default = let
      pkgs = import nixpkgs {
        inherit system;
      };
    in pkgs.mkShell {
      packages = with pkgs; [
        python311
        poetry
        rustc
        cargo
        rustfmt
        rustPackages.clippy
      ];

      shellHook = ''
        export POETRY_VIRTUALENVS_IN_PROJECT=true
        source .venv/bin/activate
        exec zsh
      '';
    };
  };
}