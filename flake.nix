{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    systems.url = "github:nix-systems/default";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      nixpkgs,
      systems,
      rust-overlay,
      ...
    }:
    let
      eachSystem = nixpkgs.lib.genAttrs (import systems);
    in
    {
      devShells = eachSystem (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };

          rustToolchain = pkgs.rust-bin.stable.latest.default.override {
            extensions = [
              "rust-src"
              "rust-analyzer"
            ];
          };
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              #  xorg.libXcursor
              #  xorg.libXi
              #  xorg.libXinerama
              #  xorg.libXrandr
              #  xorg.xinput
              rustToolchain
              SDL2
            ];

            #LD_LIBRARY_PATH =
            #  with pkgs;
            #  lib.makeLibraryPath [
            #    libGL
            #    libxkbcommon
            #    wayland
            #    xorg.libX11
            #    xorg.libXcursor
            #    xorg.libXi
            #    xorg.libXrandr
            #  ];
          };
        }
      );
    };
}
