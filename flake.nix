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
    personalConfiguration = { pkgs, ... }: {
      environment.systemPackages = [
        # CLI tools
        pkgs.ast-grep
        pkgs.awscli2
        pkgs.btop
        pkgs.bun
        pkgs.difftastic
        pkgs.duckdb
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
        pkgs.imagemagick
        pkgs.inetutils
        pkgs.mas
        pkgs.miller
        pkgs.luarocks
        pkgs.netlify-cli
        pkgs.nodejs
        pkgs.ocaml
        pkgs.pandoc
        pkgs.tectonic
        pkgs.postgresql_14
        pkgs.pup
        pkgs.ripgrep
        pkgs.ruff
        pkgs.cargo
        pkgs.rustc
        pkgs.shellcheck
        pkgs.sox
        pkgs.stylua
        pkgs.terraform
        pkgs.tmux
        pkgs.tree
        pkgs.turso-cli
        pkgs.watchman
        pkgs.wget
        pkgs.yq-go
        pkgs.zlib

        # GUI apps
        pkgs.anki
        pkgs.brave
        pkgs.iterm2
        pkgs.numi
        pkgs.obsidian
        pkgs.rectangle
      ];

      homebrew = {
        enable = true;
        onActivation.autoUpdate = true;
        onActivation.upgrade = true;
        onActivation.cleanup = "uninstall";

        brews = [
          "mole"
          "opencode"
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
          "warp"
        ];
      };

      nixpkgs.config.allowUnfree = true;

      # Clean Homebrew cache after each rebuild.
      system.activationScripts.postActivation.text = ''
        echo "Cleaning Homebrew cache..."
        /opt/homebrew/bin/brew cleanup --prune=all 2>/dev/null || true
      '';

      nix.enable = false;
      nix.settings.experimental-features = "nix-command flakes";

      system.configurationRevision = self.rev or self.dirtyRev or null;
      system.stateVersion = 6;

      networking.hostName = "Leonardos-MacBook-Pro";
      nixpkgs.hostPlatform = "aarch64-darwin";
      system.primaryUser = "leonardogavaudan";
      users.users.leonardogavaudan.home = "/Users/leonardogavaudan";
    };

    workConfiguration = { pkgs, ... }: {
      environment.systemPackages = [
        pkgs.vim
        pkgs.neovim
        pkgs.yarn
        pkgs.ripgrep
        pkgs.fd
        pkgs.jq
        pkgs.mariadb.client
        pkgs.tree
        pkgs.hyperfine
        pkgs.htop
        pkgs.btop
        pkgs.go
        pkgs.golangci-lint
        pkgs.graphviz
        pkgs.grpcurl
        pkgs.cue
        pkgs.vale
        pkgs.findutils
        (pkgs.python3.withPackages (ps: [ ps.pyyaml ]))
        pkgs.python310
        pkgs.circleci-cli
        pkgs.uv
        pkgs.google-cloud-sql-proxy

        pkgs.playwright-test
        (pkgs.writeShellScriptBin "rustup" ''
          exec ${pkgs.rustup}/bin/rustup "$@"
        '')
        pkgs.rustc
        pkgs.cargo
        pkgs.clippy
        pkgs.rustfmt
        pkgs.gleam
        pkgs.erlang
        pkgs.rebar3
        pkgs.jujutsu
        pkgs.yq-go
        pkgs.miller
        pkgs.gron
      ];

      nix.enable = false;
      nixpkgs.config.allowUnfree = true;

      homebrew = {
        enable = true;
        onActivation = {
          cleanup = "zap";
          autoUpdate = true;
          upgrade = true;
        };
        taps = [
          "golangci/tap"
          "hashicorp/tap"
          "puma/puma"
        ];
        brews = [
          "git-xargs"
          "hashicorp/tap/terraform"
          "hashicorp/tap/vault"
          "ast-grep"
          "nushell"
          "nvm"
          "pup"
          "opencode"
          "postgresql@14"
          "poetry"
          "puma-dev"
          "rbenv"
          "sql-migrate"
        ];
        casks = [
          "beeper"
          "brave-browser"
          "codex"
          "cursor"
          "dbeaver-community"
          "docker-desktop"
          "figma"
          "font-fira-code-nerd-font"
          "font-iosevka-nerd-font"
          "font-jetbrains-mono-nerd-font"
          "gcloud-cli"
          "ghostty"
          "insomnia"
          "notion-calendar"
          "obsidian"
          "opencode-desktop"
          "openvpn-connect"
          "raycast"
          "rectangle"
          "spotify"
          "tableplus"
          "temurin"
          "visual-studio-code"
          "zed"
        ];
      };

      system.configurationRevision = self.rev or self.dirtyRev or null;
      system.stateVersion = 6;

      networking.hostName = "PAR-M4P-LGavaudan";
      nixpkgs.hostPlatform = "aarch64-darwin";
      system.primaryUser = "leonardo.gavaudan";
      users.users."leonardo.gavaudan".home = "/Users/leonardo.gavaudan";
    };
  in
  {
    darwinConfigurations."Leonardos-MacBook-Pro" = nix-darwin.lib.darwinSystem {
      modules = [
        personalConfiguration
        home-manager.darwinModules.home-manager
        {
          home-manager.useGlobalPkgs = true;
          home-manager.useUserPackages = true;
          home-manager.backupFileExtension = "backup";
          home-manager.users.leonardogavaudan = import ./home.nix;
        }
      ];
    };

    darwinConfigurations."PAR-M4P-LGavaudan" = nix-darwin.lib.darwinSystem {
      modules = [
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
