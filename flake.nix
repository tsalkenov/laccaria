{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      with pkgs; {
        devShells.default = mkShell rec {
          packages = [
            pkg-config
          ];
        };

        shellHook = ''
          export LD_LIBRARY_PATH = ${stdenv.cc.cc.lib}/lib
        '';
      }
    );
}
