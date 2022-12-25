{ pkgs ? import <nixpkgs> { } }:
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    libvirt
    libvirt-glib.dev
  ];
  RUST_LOG="debug";
}
