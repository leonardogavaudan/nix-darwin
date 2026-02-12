{ config, pkgs, ... }:

let
  homeDir = config.home.homeDirectory;
  logDir = "${homeDir}/.local/share/resource-monitor";
  scriptsDir = "${homeDir}/.config/nix-darwin/scripts/resource-monitor";
in
{
  # Ensure launchd log target exists
  home.file.".local/share/resource-monitor/.keep".text = "";

  # ── Wrapper: resource-logger ──────────────────────────────────
  home.file.".local/bin/resource-logger" = {
    executable = true;
    text = ''
      #!/bin/bash
      set -euo pipefail

      manifest="${scriptsDir}/Cargo.toml"
      src_dir="${scriptsDir}/src"
      target_dir="${scriptsDir}/target"
      bin_path="${scriptsDir}/target/release/resource-logger"

      rebuild=0
      if [ ! -x "$bin_path" ]; then
        rebuild=1
      elif [ "$manifest" -nt "$bin_path" ]; then
        rebuild=1
      elif find "$src_dir" -type f -newer "$bin_path" -print -quit | grep -q .; then
        rebuild=1
      fi

      if [ "$rebuild" -eq 1 ]; then
        /usr/bin/env cargo build --release --manifest-path "$manifest" --target-dir "$target_dir" --bin resource-logger >/dev/null
      fi

      exec "$bin_path"
    '';
  };

  # ── Wrapper: resource-query ───────────────────────────────────
  home.file.".local/bin/resource-query" = {
    executable = true;
    text = ''
      #!/bin/bash
      set -euo pipefail

      manifest="${scriptsDir}/Cargo.toml"
      src_dir="${scriptsDir}/src"
      target_dir="${scriptsDir}/target"
      bin_path="${scriptsDir}/target/release/resource-query"

      rebuild=0
      if [ ! -x "$bin_path" ]; then
        rebuild=1
      elif [ "$manifest" -nt "$bin_path" ]; then
        rebuild=1
      elif find "$src_dir" -type f -newer "$bin_path" -print -quit | grep -q .; then
        rebuild=1
      fi

      if [ "$rebuild" -eq 1 ]; then
        /usr/bin/env cargo build --release --manifest-path "$manifest" --target-dir "$target_dir" --bin resource-query >/dev/null
      fi

      exec "$bin_path" "$@"
    '';
  };

  # ── Launchd agent ─────────────────────────────────────────────
  launchd.agents.resource-monitor = {
    enable = true;
    config = {
      Label = "com.leonardogavaudan.resource-monitor";
      ProgramArguments = [ "${homeDir}/.local/bin/resource-logger" ];
      StartInterval = 60;
      StandardOutPath = "${logDir}/launchd-stdout.log";
      StandardErrorPath = "${logDir}/launchd-stderr.log";
      EnvironmentVariables.PATH = "/run/current-system/sw/bin:/usr/bin:/bin:/usr/sbin:/sbin";
    };
  };
}
