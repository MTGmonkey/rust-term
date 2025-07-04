{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = {
    nixpkgs,
    naersk,
    ...
  }: let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};
  in {
    packages.${system} = {
      default = pkgs.callPackage ./package.nix {
        naersk = pkgs.callPackage naersk {};
      };
    };
    #    nixosModules.${system}.default = ./module.nix;
  };
}
