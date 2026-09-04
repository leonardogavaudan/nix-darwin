{ config, lib, ... }:

# Homebrew 6 stores tap trust decisions in $XDG_CONFIG_HOME/homebrew/trust.json
# when that variable is set (as it is in the interactive shell), and in
# ~/.homebrew/trust.json otherwise. nix-darwin runs `brew bundle` via
# `sudo --preserve-env=PATH`, which drops XDG_CONFIG_HOME, so it reads the
# second location. Symlink the whole directory so `brew trust` in the shell
# is honoured by darwin-rebuild too.
#
# This must be a plain symlink to a user-owned directory: brew refuses to
# write a trust store whose resolved parent lives in the nix store, which
# rules out home.file for this.

let
  homeDir = config.home.homeDirectory;
in
{
  home.activation.linkHomebrewTrustDir = lib.hm.dag.entryAfter [ "writeBoundary" ] ''
    if [ ! -L "${homeDir}/.homebrew" ]; then
      if [ -e "${homeDir}/.homebrew" ]; then
        echo "warning: ${homeDir}/.homebrew exists and is not a symlink; leaving it alone" >&2
      else
        run mkdir -p "${homeDir}/.config/homebrew"
        run ln -sfn "${homeDir}/.config/homebrew" "${homeDir}/.homebrew"
      fi
    fi
  '';
}
