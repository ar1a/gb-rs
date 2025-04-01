{
  pkgs ? import <nixpkgs> { },
}:
let
  dlopenLibraries = with pkgs; [
    libxkbcommon
    wayland
  ];
in
pkgs.mkShell {
  env.RUSTFLAGS = "-C link-arg=-Wl,-rpath,${pkgs.lib.makeLibraryPath dlopenLibraries}";
}
