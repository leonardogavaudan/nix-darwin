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
    commonConfiguration = { pkgs, ... }: {
      environment.systemPackages = [
        pkgs.ast-grep
        pkgs.btop
        pkgs.bun
        pkgs.cargo
        pkgs.difftastic
        pkgs.duckdb
        pkgs.erlang
        pkgs.fd
        pkgs.gh
        pkgs.gleam
        pkgs.go
        pkgs.google-cloud-sql-proxy
        pkgs.gron
        pkgs.htop
        pkgs.hyperfine
        pkgs.jq
        pkgs.jujutsu
        pkgs.mariadb.client
        pkgs.miller
        pkgs.neovim
        pkgs.pup
        pkgs.ripgrep
        pkgs.rustc
        pkgs.rustfmt
        pkgs.shellcheck
        pkgs.stylua
        pkgs.tmux
        pkgs.tree
        pkgs.uv
        pkgs.vim
        pkgs.yarn
        pkgs.yq-go
      ];

      homebrew = {
        enable = true;
        onActivation.autoUpdate = true;
        onActivation.upgrade = true;

        taps = [
          "hashicorp/tap"
        ];

        brews = [
          "hashicorp/tap/terraform"
          "mole"
          "nushell"
          "nvm"
          "opencode"
          "poetry"
          "postgresql@14"
        ];

        casks = [
          "beeper"
          "brave-browser"
          "codex"
          "cursor"
          "font-commit-mono-nerd-font"
          "font-droid-sans-mono-nerd-font"
          "font-fira-code-nerd-font"
          "font-inconsolata-nerd-font"
          "font-iosevka-nerd-font"
          "font-jetbrains-mono-nerd-font"
          "gcloud-cli"
          "ghostty"
          "notion-calendar"
          "obsidian"
          "raycast"
          "rectangle"
          "spotify"
          "tailscale-app"
          "tableplus"
          "visual-studio-code"
        ];
      };

      nix.enable = false;
      nixpkgs.config.allowUnfree = true;

      system.configurationRevision = self.rev or self.dirtyRev or null;
      system.stateVersion = 6;

      nixpkgs.hostPlatform = "aarch64-darwin";
    };

    personalConfiguration = { pkgs, ... }: {
      environment.systemPackages = [
        # CLI tools
        pkgs.awscli2
        pkgs.duti
        pkgs.podman-compose
        pkgs.eza
        pkgs.ffmpeg
        pkgs.cabal-install
        pkgs.ghc
        pkgs.google-cloud-sdk
        pkgs.imagemagick
        pkgs.inetutils
        pkgs.mas
        pkgs.luarocks
        pkgs.netlify-cli
        pkgs.nodejs
        pkgs.ocaml
        pkgs.pandoc
        pkgs.tectonic
        pkgs.ruff
        pkgs.sox
        pkgs.turso-cli
        pkgs.watchman
        pkgs.wget
        pkgs.zlib

        # GUI apps
        pkgs.anki
        pkgs.iterm2
        pkgs.numi
      ];

      homebrew = {
        onActivation.cleanup = "uninstall";

        brews = [
          "podman"
          "cloudflare-wrangler"
        ];

        casks = [
          "chromium"
          "gimp"
          "google-drive"
          "messenger"
          "signal"
          "warp"
        ];
      };

      # Clean Homebrew cache after each rebuild.
      system.activationScripts.postActivation.text = ''
        echo "Cleaning Homebrew cache..."
        /opt/homebrew/bin/brew cleanup --prune=all 2>/dev/null || true
      '';

      nix.settings.experimental-features = "nix-command flakes";

      networking.hostName = "Leonardos-MacBook-Pro";
      system.primaryUser = "leonardogavaudan";
      users.users.leonardogavaudan.home = "/Users/leonardogavaudan";
    };

    workConfiguration = { pkgs, ... }: {
      environment.systemPackages = [
        pkgs.golangci-lint
        pkgs.graphviz
        pkgs.grpcurl
        pkgs.cue
        pkgs.vale
        pkgs.findutils
        (pkgs.python3.withPackages (ps: [ ps.pyyaml ]))
        pkgs.python310
        pkgs.circleci-cli
        pkgs.playwright-test
        (pkgs.writeShellScriptBin "rustup" ''
          exec ${pkgs.rustup}/bin/rustup "$@"
        '')
        pkgs.clippy
        pkgs.rebar3
      ];

      homebrew = {
        onActivation.cleanup = "zap";

        taps = [
          "golangci/tap"
          "puma/puma"
        ];

        brews = [
          "git-xargs"
          "hashicorp/tap/vault"
          "puma-dev"
          "rbenv"
          "sql-migrate"
        ];

        casks = [
          "docker-desktop"
          "figma"
          "insomnia"
          "openvpn-connect"
          "temurin"
        ];
      };

      networking.hostName = "PAR-M4P-LGavaudan";
      system.primaryUser = "leonardo.gavaudan";
      users.users."leonardo.gavaudan".home = "/Users/leonardo.gavaudan";
    };
  in
  {
    darwinConfigurations."Leonardos-MacBook-Pro" = nix-darwin.lib.darwinSystem {
      modules = [
        commonConfiguration
        personalConfiguration
        home-manager.darwinModules.home-manager
        {
          home-manager.useGlobalPkgs = true;
          home-manager.useUserPackages = true;
          home-manager.backupFileExtension = "backup";
          home-manager.users.leonardogavaudan = import ./home-personal.nix;
        }
      ];
    };

    darwinConfigurations."PAR-M4P-LGavaudan" = nix-darwin.lib.darwinSystem {
      modules = [
        commonConfiguration
        workConfiguration
        home-manager.darwinModules.home-manager
        {
          home-manager.useGlobalPkgs = true;
          home-manager.useUserPackages = true;
          home-manager.users."leonardo.gavaudan" = import ./home-work.nix;
        }
      ];
    };

    legacyPackages.aarch64-darwin =
      self.darwinConfigurations."Leonardos-MacBook-Pro".pkgs;
  };
}
