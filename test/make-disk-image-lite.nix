/**
  Create a bootable disk image.

  `makeDiskImage` builds a disk image from a NixOS configuration. It is the
  whole-file example: the file evaluates to a single anonymous function and
  this header is its documentation.

  # Partitioning

  Supports `legacy`, `efi`, and `none` partition tables.
*/
{
  pkgs,
  lib,

  # The NixOS configuration to install.
  config,

  # Disk size in MiB, or "auto".
  diskSize ? "auto",
}:
pkgs.vmTools.runInLinuxVM (
  pkgs.runCommand "nixos-disk-image" { } ''
    touch $out
  ''
)
