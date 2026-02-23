{ ... }:

{
  programs.zoxide = {
    enable = true;
    enableZshIntegration = true;
  };

  programs.eza = {
    enable = true;
    enableZshIntegration = true;
    icons = "auto";
    git = true;
    extraOptions = [ "--group-directories-first" ];
  };

  programs.bat.enable = true;

  programs.starship = {
    enable = true;
    enableZshIntegration = true;
    settings = {
      buf.disabled = true;
      gcloud.disabled = true;
      docker_context.disabled = true;
      package.disabled = true;
      cmd_duration.min_time = 3000;
      directory.truncation_length = 3;
    };
  };

  programs.gh = {
    enable = true;
    settings = {
      git_protocol = "https";
    };
  };

  programs.git = {
    enable = true;
    ignores = [
      "**/.claude/settings.local.json"
      "CLAUDE.local.md"
      ".local"
    ];
    settings = {
      user.name = "Leonardo Gavaudan";
      user.email = "leonardogavaudan@gmail.com";
      pull.rebase = true;
      rerere.enabled = true;
      rerere.autoupdate = true;
    };
  };
}

