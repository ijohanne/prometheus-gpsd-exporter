# prometheus-gpsd-exporter

Prometheus exporter for [gpsd](https://gpsd.io/). Connects to gpsd's JSON protocol over TCP and exposes GPS/GNSS metrics for Prometheus scraping.

## Features

- **Streaming** — reads the gpsd JSON stream in real-time, not polling
- **All message types** — TPV, SKY, PPS, GST, TOFF, OSC, VERSION, DEVICES
- **Per-satellite metrics** — signal strength, azimuth, elevation, health with full labels (PRN, svid, gnssid, used, sigid, freqid)
- **PPS histogram** — clock offset distribution with configurable buckets and time1 correction
- **Geo-offset histograms** — distance and bearing from a fixed reference point
- **Automatic reconnect** — exponential backoff on connection loss, never crashes on bad data
- **NixOS module** — included for declarative deployment

## Usage

```
prometheus-gpsd-exporter [OPTIONS]
```

### Options

| Flag | Default | Description |
|------|---------|-------------|
| `-H, --hostname` | `localhost` | gpsd host |
| `-p, --port` | `2947` | gpsd port |
| `-E, --exporter-port` | `9015` | Metrics HTTP port |
| `-L, --listen-address` | `::` | Metrics listen address |
| `-t, --timeout` | `10` | Connection timeout (seconds) |
| `--retry-delay` | `10` | Initial retry delay (seconds) |
| `--max-retry-delay` | `300` | Maximum retry delay (seconds) |
| `-S, --disable-monitor-satellites` | | Disable per-satellite metrics |
| `--pps-histogram` | | Enable PPS clock offset histogram |
| `--pps-bucket-size` | `250` | PPS histogram bucket size (ns) |
| `--pps-bucket-count` | `40` | PPS histogram bucket count |
| `--pps-time1` | `0` | PPS time1 offset correction |
| `--offset-from-geopoint` | | Enable geo-offset tracking |
| `--geopoint-lat` | `0` | Reference latitude |
| `--geopoint-lon` | `0` | Reference longitude |
| `--geo-bucket-size` | `0.5` | Geo histogram bucket size (meters) |
| `--geo-bucket-count` | `40` | Geo histogram bucket count |

### Example

```bash
# Basic usage — connect to remote gpsd
prometheus-gpsd-exporter -H 10.0.0.5

# With PPS histogram and geo-offset tracking
prometheus-gpsd-exporter -H 10.0.0.5 \
  --pps-histogram \
  --offset-from-geopoint --geopoint-lat 51.5074 --geopoint-lon -0.1278
```

## Metrics

### TPV (Time-Position-Velocity)

`gpsd_lat`, `gpsd_long`, `gpsd_altHAE`, `gpsd_altMSL`, `gpsd_mode`, `gpsd_status`, `gpsd_leapseconds`, `gpsd_magvar`, `gpsd_speed`, `gpsd_track`, `gpsd_magtrack`, `gpsd_climb`, `gpsd_ept`, `gpsd_epx`, `gpsd_epy`, `gpsd_epv`, `gpsd_eps`, `gpsd_epc`, `gpsd_epd`, `gpsd_eph`, `gpsd_sep`, `gpsd_geoidSep`, `gpsd_ecefx`, `gpsd_ecefy`, `gpsd_ecefz`, `gpsd_ecefvx`, `gpsd_ecefvy`, `gpsd_ecefvz`, `gpsd_ecefpAcc`, `gpsd_ecefvAcc`, `gpsd_velN`, `gpsd_velE`, `gpsd_velD`, `gpsd_depth`, `gpsd_dgpsAge`, `gpsd_dgpsSta`, `gpsd_relN`, `gpsd_relE`, `gpsd_relD`

### SKY (Satellite View)

`gpsd_gdop`, `gpsd_hdop`, `gpsd_pdop`, `gpsd_tdop`, `gpsd_vdop`, `gpsd_xdop`, `gpsd_ydop`, `gpsd_nSat`, `gpsd_uSat`, `gpsd_prRes`, `gpsd_qual`, `gpsd_sat_used`, `gpsd_sat_seen`

### Per-Satellite (labels: PRN, svid, gnssid, used)

`gpsd_sat_ss`, `gpsd_sat_az`, `gpsd_sat_el`, `gpsd_used`, `gpsd_health`, `gpsd_sat_sigid`, `gpsd_sat_freqid`

### GST (Pseudorange Noise)

`gpsd_gst_rms`, `gpsd_gst_major`, `gpsd_gst_minor`, `gpsd_gst_orient`, `gpsd_gst_lat`, `gpsd_gst_lon`, `gpsd_gst_alt`

### TOFF (Time Offset)

`gpsd_toff_real_sec`, `gpsd_toff_real_nsec`, `gpsd_toff_clock_sec`, `gpsd_toff_clock_nsec`

### PPS (Pulse Per Second)

`gpsd_pps_real_sec`, `gpsd_pps_real_nsec`, `gpsd_pps_clock_sec`, `gpsd_pps_clock_nsec`, `gpsd_pps_precision`, `gpsd_pps_qErr`

With `--pps-histogram`: `gpsd_pps_histogram_bucket{device}`

### OSC (Oscillator)

`gpsd_osc_running`, `gpsd_osc_reference`, `gpsd_osc_disciplined`, `gpsd_osc_delta`

### Geo-offset (with `--offset-from-geopoint`)

`gpsd_geo_offset_m_histogram_bucket`, `gpsd_geo_bearing_x_histogram_bucket`, `gpsd_geo_bearing_y_histogram_bucket`

### Info

`gpsd_version_info{release, rev, proto_major, proto_minor}`, `gpsd_devices_info{device, driver, subtype, ...}`

## Building

### Cargo

```bash
cargo build --release
```

### Nix

```bash
nix build
```

## NixOS Module

```nix
{
  services.prometheus-gpsd-exporter = {
    enable = true;
    enableLocalScraping = true;
    gpsdHost = "10.0.0.5";
    ppsHistogram = true;
    offsetFromGeopoint = true;
    geopointLat = 51.5074;
    geopointLon = -0.1278;
  };
}
```

## License

MIT
