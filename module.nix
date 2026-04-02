self:
{ config, lib, pkgs, ... }:
with lib;
let
  cfg = config.services.prometheus-gpsd-exporter;
  name = "gpsd";
  package = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
in
{
  options.services.prometheus-gpsd-exporter = with types; mkOption {
    type = types.submodule {
      options = {
        enable = mkEnableOption "the prometheus ${name} exporter";
        enableLocalScraping = mkEnableOption "scraping by local prometheus";
        port = mkOption {
          type = types.port;
          default = 9015;
          description = "Port to listen on.";
        };
        listenAddress = mkOption {
          type = types.str;
          default = "127.0.0.1";
          description = "Address to listen on.";
        };
        gpsdHost = mkOption {
          type = types.str;
          default = "localhost";
          description = "Hostname or IP of the gpsd instance to connect to.";
        };
        gpsdPort = mkOption {
          type = types.port;
          default = 2947;
          description = "Port of the gpsd instance.";
        };
        timeout = mkOption {
          type = types.int;
          default = 10;
          description = "Connection timeout in seconds.";
        };
        retryDelay = mkOption {
          type = types.int;
          default = 10;
          description = "Initial retry delay in seconds.";
        };
        maxRetryDelay = mkOption {
          type = types.int;
          default = 300;
          description = "Maximum retry delay in seconds.";
        };
        disableMonitorSatellites = mkOption {
          type = types.bool;
          default = false;
          description = "Disable per-satellite monitoring.";
        };
        ppsHistogram = mkOption {
          type = types.bool;
          default = false;
          description = "Enable PPS clock offset histogram.";
        };
        ppsBucketSize = mkOption {
          type = types.int;
          default = 250;
          description = "PPS histogram bucket size in nanoseconds.";
        };
        ppsBucketCount = mkOption {
          type = types.int;
          default = 40;
          description = "PPS histogram bucket count.";
        };
        ppsTime1 = mkOption {
          type = types.float;
          default = 0.0;
          description = "PPS time1 offset correction.";
        };
        offsetFromGeopoint = mkOption {
          type = types.bool;
          default = false;
          description = "Enable geo-offset tracking from a fixed reference point.";
        };
        geopointLat = mkOption {
          type = types.float;
          default = 0.0;
          description = "Latitude of fixed reference point.";
        };
        geopointLon = mkOption {
          type = types.float;
          default = 0.0;
          description = "Longitude of fixed reference point.";
        };
        geoBucketSize = mkOption {
          type = types.float;
          default = 0.5;
          description = "Geo histogram bucket size in meters.";
        };
        geoBucketCount = mkOption {
          type = types.int;
          default = 40;
          description = "Geo histogram bucket count.";
        };
        user = mkOption {
          type = types.str;
          default = "${name}-exporter";
          description = "User name under which the ${name} exporter shall be run.";
        };
        group = mkOption {
          type = types.str;
          default = "${name}-exporter";
          description = "Group under which the ${name} exporter shall be run.";
        };
      };
    };
    default = { };
  };

  config = mkIf cfg.enable {
    users.users."${cfg.user}" = {
      description = "Prometheus ${name} exporter service user";
      isSystemUser = true;
      group = "${cfg.group}";
    };
    users.groups."${cfg.group}" = { };

    systemd.services."prometheus-${name}-exporter" =
      let
        wrapper = pkgs.writeShellScript "prometheus-${name}-exporter" ''
          exec ${getBin package}/bin/prometheus-gpsd-exporter \
            --hostname ${cfg.gpsdHost} \
            --port ${toString cfg.gpsdPort} \
            --exporter-port ${toString cfg.port} \
            --listen-address ${cfg.listenAddress} \
            --timeout ${toString cfg.timeout} \
            --retry-delay ${toString cfg.retryDelay} \
            --max-retry-delay ${toString cfg.maxRetryDelay} \
            ${optionalString cfg.disableMonitorSatellites "--disable-monitor-satellites"} \
            ${optionalString cfg.ppsHistogram "--pps-histogram"} \
            --pps-bucket-size ${toString cfg.ppsBucketSize} \
            --pps-bucket-count ${toString cfg.ppsBucketCount} \
            --pps-time1 ${toString cfg.ppsTime1} \
            ${optionalString cfg.offsetFromGeopoint "--offset-from-geopoint"} \
            --geopoint-lat ${toString cfg.geopointLat} \
            --geopoint-lon ${toString cfg.geopointLon} \
            --geo-bucket-size ${toString cfg.geoBucketSize} \
            --geo-bucket-count ${toString cfg.geoBucketCount}
        '';
      in
      {
        wantedBy = [ "multi-user.target" ];
        after = [ "network.target" ];
        serviceConfig = {
          Restart = "always";
          PrivateTmp = true;
          WorkingDirectory = "/tmp";
          DynamicUser = false;
          User = cfg.user;
          Group = cfg.group;
          ExecStart = toString wrapper;
        };
      };

    services.prometheus.scrapeConfigs = mkIf cfg.enableLocalScraping [
      {
        job_name = "${name}";
        honor_labels = true;
        static_configs = [{
          targets = [ "127.0.0.1:${toString cfg.port}" ];
        }];
      }
    ];
  };
}
