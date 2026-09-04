{ config, pkgs, ... }:

# Daily syncs of Withings (body composition) and Oura (sleep/activity) data
# into the Obsidian vault under "Health & Cosmetics". Each sync script reads
# its API tokens from the login keychain, so these must run as user agents.
#
# StartCalendarInterval (unlike cron) fires a missed run at next wake, so a
# sleeping laptop at 09:15 does not skip the day.

let
  homeDir = config.home.homeDirectory;
  logDir = "${homeDir}/.local/share/vault-sync";
  devDir = "${homeDir}/dev";
  path = "/run/current-system/sw/bin:/usr/bin:/bin:/usr/sbin:/sbin";

  mkSync = { name, minute }: {
    enable = true;
    config = {
      Label = "com.leonardogavaudan.vault-sync-${name}";
      ProgramArguments = [
        "${pkgs.bun}/bin/bun"
        "${devDir}/${name}-mcp/src/sync-vault.ts"
      ];
      WorkingDirectory = "${devDir}/${name}-mcp";
      StartCalendarInterval = [ { Hour = 9; Minute = minute; } ];
      StandardOutPath = "${logDir}/${name}.log";
      StandardErrorPath = "${logDir}/${name}.log";
      EnvironmentVariables.PATH = path;
    };
  };
in
{
  home.file.".local/share/vault-sync/.keep".text = "";

  launchd.agents.vault-sync-withings = mkSync { name = "withings"; minute = 15; };
  launchd.agents.vault-sync-oura = mkSync { name = "oura"; minute = 20; };
}
