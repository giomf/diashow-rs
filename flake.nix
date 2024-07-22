{
  description = "A very basic rust flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    flake-utils.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { nixpkgs, flake-utils, ...}:
    flake-utils.lib.eachDefaultSystem (system:
      let pkgs = nixpkgs.legacyPackages.${system};
      in {
        devShells.default = pkgs.mkShell rec {
          buildInputs = with pkgs; [
            cargo
            rustc
            rust-analyzer
            rustfmt
            
            libxkbcommon
            libGL

            # WINIT_UNIX_BACKEND=wayland
            wayland

            # WINIT_UNIX_BACKEND=x11
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi
            xorg.libX11
           ];
          env = {
            LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
          };
        };
      });
}
