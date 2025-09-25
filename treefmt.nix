{ lib, ... }:
{
  # Used to find the project root
  projectRootFile = "flake.nix";

  programs.nixfmt.enable = true;

  programs.rustfmt = {
    enable = true;
    edition = (lib.importTOML ./Cargo.toml).package.edition;
  };
}
