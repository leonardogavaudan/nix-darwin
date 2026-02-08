{
  description = "Leonardo's Mac configuration";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nix-darwin.url = "github:LnL7/nix-darwin";
    nix-darwin.inputs.nixpkgs.follows = "nixpkgs";
    home-manager.url = "github:nix-community/home-manager";
    home-manager.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs@{ self, nix-darwin, nixpkgs, home-manager }:
  let
    configuration = { pkgs, ... }: {

      # ============================================================
      # PACKAGES (managed by Nix)
      # ============================================================

      environment.systemPackages = [
        # CLI tools
        pkgs.awscli2
        pkgs.btop
        pkgs.bun
        pkgs.duti
        pkgs.podman-compose
        pkgs.eza
        pkgs.fd
        pkgs.ffmpeg
        pkgs.gh
        pkgs.cabal-install
        pkgs.ghc
        pkgs.go
        pkgs.google-cloud-sdk
        pkgs.inetutils        # telnet
        pkgs.mas
        pkgs.luarocks
        pkgs.neovim
        pkgs.netlify-cli
        pkgs.nodejs
        pkgs.ocaml
        pkgs.opencode
        pkgs.postgresql_14
        pkgs.ripgrep
        pkgs.ruff
        pkgs.sox
        pkgs.stylua
        pkgs.terraform
        pkgs.tmux
        pkgs.tree
        pkgs.turso-cli
        pkgs.watchman
        pkgs.wget
        pkgs.zlib

        # GUI apps
        pkgs.anki
        pkgs.brave
        pkgs.iterm2
        pkgs.numi
        pkgs.obsidian
        pkgs.rectangle
      ];

      # ============================================================
      # HOMEBREW (only for apps not in Nix)
      # ============================================================

      homebrew = {
        enable = true;
        onActivation.autoUpdate = true;
        onActivation.upgrade = true;
        onActivation.cleanup = "uninstall";

        brews = [
          "mole"
          "podman"
          "cloudflare-wrangler"
        ];

        casks = [
          "beeper"
          "chromium"
          "codex"
          "cursor"
          "font-commit-mono-nerd-font"
          "font-droid-sans-mono-nerd-font"
          "font-fira-code-nerd-font"
          "font-inconsolata-nerd-font"
          "font-iosevka-nerd-font"
          "ghostty"
          "gimp"
          "google-drive"
          "messenger"
          "raycast"
          "signal"
          "spotify"
          "tailscale"
          "warp"
        ];
      };

      # ============================================================
      # SYSTEM
      # ============================================================

      nixpkgs.config.allowUnfree = true;

      # Clean Homebrew cache after each rebuild
      system.activationScripts.postActivation.text = ''
        echo "Cleaning Homebrew cache..."
        /opt/homebrew/bin/brew cleanup --prune=all 2>/dev/null || true
      '';

      nix.enable = false;
      nix.settings.experimental-features = "nix-command flakes";

      system.stateVersion = 6;

      networking.hostName = "Leonardos-MacBook-Pro";
      nixpkgs.hostPlatform = "aarch64-darwin";
      system.primaryUser = "leonardogavaudan";
      users.users.leonardogavaudan.home = "/Users/leonardogavaudan";
    };
  in
  {
    darwinConfigurations."Leonardos-MacBook-Pro" = nix-darwin.lib.darwinSystem {
      modules = [
        configuration
        home-manager.darwinModules.home-manager
        {
          home-manager.useGlobalPkgs = true;
          home-manager.useUserPackages = true;
          home-manager.backupFileExtension = "backup";
          home-manager.users.leonardogavaudan = import ./home.nix;
        }
      ];
    };

    darwinPackages = self.darwinConfigurations."Leonardos-MacBook-Pro".pkgs;
  };
}
