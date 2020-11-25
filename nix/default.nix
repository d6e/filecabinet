{ sources ? import ./sources.nix
}:

let
  # default nixpkgs
  pkgs = import sources.nixpkgs {};

  # gitignore.nix
  gitignoreSource = (import sources."gitignore.nix" { inherit (pkgs) lib; }).gitignoreSource;

  src = gitignoreSource ./..;
in
{
  inherit pkgs src;

  # provided by shell.nix
  devTools = {
    inherit (pkgs) niv;
  };

  # to be built by github actions
  ci = {
    pre-commit-check = (import sources."pre-commit-hooks.nix").run {
      inherit src;
      hooks = {
      };
      # generated files
      excludes = [ "^nix/sources\.nix$" ];
    };
  };
}
