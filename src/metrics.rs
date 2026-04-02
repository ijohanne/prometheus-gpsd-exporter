use prometheus::{GaugeVec, Histogram, HistogramOpts, HistogramVec, Opts, Registry};

use crate::gpsd::{GpsdMessage, Satellite};

const NSEC: f64 = 1_000_000_000.0;
const EARTH_RADIUS_M: f64 = 6_371_000.0;

macro_rules! gauge {
    ($registry:expr, $name:expr, $help:expr) => {{
        let g = prometheus::Gauge::new($name, $help).expect("metric can be created");
        $registry.register(Box::new(g.clone())).expect("collector can be registered");
        g
    }};
}

macro_rules! gauge_vec {
    ($registry:expr, $name:expr, $help:expr, $labels:expr) => {{
        let g = GaugeVec::new(Opts::new($name, $help), $labels).expect("metric can be created");
        $registry
            .register(Box::new(g.clone()))
            .expect("collector can be registered");
        g
    }};
}

#[derive(Clone, Debug)]
pub struct MetricsConfig {
    pub monitor_satellites: bool,
    pub pps_histogram: bool,
    pub pps_bucket_size: i64,
    pub pps_bucket_count: i64,
    pub pps_time1: f64,
    pub geo_offset: bool,
    pub geo_lat: f64,
    pub geo_lon: f64,
    pub geo_bucket_size: f64,
    pub geo_bucket_count: i64,
}

pub struct Metrics {
    pub registry: Registry,
    pub config: MetricsConfig,

    // TPV
    pub lat: prometheus::Gauge,
    pub lon: prometheus::Gauge,
    pub alt_hae: prometheus::Gauge,
    pub alt_msl: prometheus::Gauge,
    pub mode: prometheus::Gauge,
    pub status: prometheus::Gauge,
    pub leapseconds: prometheus::Gauge,
    pub magvar: prometheus::Gauge,
    pub ept: prometheus::Gauge,
    pub epx: prometheus::Gauge,
    pub epy: prometheus::Gauge,
    pub epv: prometheus::Gauge,
    pub eps: prometheus::Gauge,
    pub epc: prometheus::Gauge,
    pub epd: prometheus::Gauge,
    pub eph: prometheus::Gauge,
    pub sep: prometheus::Gauge,
    pub geoid_sep: prometheus::Gauge,
    pub ecefx: prometheus::Gauge,
    pub ecefy: prometheus::Gauge,
    pub ecefz: prometheus::Gauge,
    pub ecefvx: prometheus::Gauge,
    pub ecefvy: prometheus::Gauge,
    pub ecefvz: prometheus::Gauge,
    pub ecefp_acc: prometheus::Gauge,
    pub ecefv_acc: prometheus::Gauge,
    pub vel_n: prometheus::Gauge,
    pub vel_e: prometheus::Gauge,
    pub vel_d: prometheus::Gauge,
    pub speed: prometheus::Gauge,
    pub track: prometheus::Gauge,
    pub magtrack: prometheus::Gauge,
    pub climb: prometheus::Gauge,
    pub depth: prometheus::Gauge,
    pub dgps_age: prometheus::Gauge,
    pub dgps_sta: prometheus::Gauge,
    pub rel_n: prometheus::Gauge,
    pub rel_e: prometheus::Gauge,
    pub rel_d: prometheus::Gauge,

    // SKY
    pub gdop: prometheus::Gauge,
    pub hdop: prometheus::Gauge,
    pub pdop: prometheus::Gauge,
    pub tdop: prometheus::Gauge,
    pub vdop: prometheus::Gauge,
    pub xdop: prometheus::Gauge,
    pub ydop: prometheus::Gauge,
    pub n_sat: prometheus::Gauge,
    pub u_sat: prometheus::Gauge,
    pub pr_res: prometheus::Gauge,
    pub qual: prometheus::Gauge,
    pub sat_used: prometheus::Gauge,
    pub sat_seen: prometheus::Gauge,

    // Per-satellite (None if disabled)
    pub sat_ss: Option<GaugeVec>,
    pub sat_az: Option<GaugeVec>,
    pub sat_el: Option<GaugeVec>,
    pub sat_is_used: Option<GaugeVec>,
    pub sat_health: Option<GaugeVec>,
    pub sat_sigid: Option<GaugeVec>,
    pub sat_freqid: Option<GaugeVec>,

    // Version / Devices
    pub version_info: GaugeVec,
    pub devices_info: GaugeVec,

    // GST
    pub gst_rms: prometheus::Gauge,
    pub gst_major: prometheus::Gauge,
    pub gst_minor: prometheus::Gauge,
    pub gst_orient: prometheus::Gauge,
    pub gst_lat: prometheus::Gauge,
    pub gst_lon: prometheus::Gauge,
    pub gst_alt: prometheus::Gauge,

    // TOFF
    pub toff_real_sec: prometheus::Gauge,
    pub toff_real_nsec: prometheus::Gauge,
    pub toff_clock_sec: prometheus::Gauge,
    pub toff_clock_nsec: prometheus::Gauge,

    // PPS gauges (always)
    pub pps_real_sec: prometheus::Gauge,
    pub pps_real_nsec: prometheus::Gauge,
    pub pps_clock_sec: prometheus::Gauge,
    pub pps_clock_nsec: prometheus::Gauge,
    pub pps_precision: prometheus::Gauge,
    pub pps_q_err: prometheus::Gauge,

    // PPS histogram (optional)
    pub pps_hist: Option<HistogramVec>,

    // Geo-offset histograms (optional)
    pub geo_offset_hist: Option<Histogram>,
    pub geo_bearing_x_hist: Option<Histogram>,
    pub geo_bearing_y_hist: Option<Histogram>,

    // OSC
    pub osc_running: prometheus::Gauge,
    pub osc_reference: prometheus::Gauge,
    pub osc_disciplined: prometheus::Gauge,
    pub osc_delta: prometheus::Gauge,
}

fn make_pps_buckets(bucket_size: i64, bucket_count: i64) -> Vec<f64> {
    let half = bucket_count / 2;
    let mut buckets: Vec<f64> = Vec::new();
    for i in -half..=half {
        buckets.push((i * bucket_size) as f64);
    }
    buckets
}

fn make_geo_offset_buckets(bucket_size: f64, bucket_count: i64) -> Vec<f64> {
    (1..bucket_count).map(|i| i as f64 * bucket_size).collect()
}

fn make_geo_yx_buckets(bucket_size: f64, bucket_count: i64) -> Vec<f64> {
    let half = bucket_count / 2;
    (-half..=half).map(|i| i as f64 * bucket_size).collect()
}

impl Metrics {
    #[allow(clippy::too_many_lines)]
    pub fn new(config: MetricsConfig) -> Self {
        let registry = Registry::new();

        let sat_labels: &[&str] = &["PRN", "svid", "gnssid", "used"];

        let (sat_ss, sat_az, sat_el, sat_is_used, sat_health, sat_sigid, sat_freqid) =
            if config.monitor_satellites {
                (
                    Some(gauge_vec!(registry, "gpsd_sat_ss", "Signal to noise ratio in dBHz", sat_labels)),
                    Some(gauge_vec!(registry, "gpsd_sat_az", "Azimuth, degrees from true north", sat_labels)),
                    Some(gauge_vec!(registry, "gpsd_sat_el", "Elevation in degrees", sat_labels)),
                    Some(gauge_vec!(registry, "gpsd_used", "Used satellite", sat_labels)),
                    Some(gauge_vec!(registry, "gpsd_health", "Satellite health: 0=unknown, 1=OK, 2=unhealthy", sat_labels)),
                    Some(gauge_vec!(registry, "gpsd_sat_sigid", "Signal ID", sat_labels)),
                    Some(gauge_vec!(registry, "gpsd_sat_freqid", "GLONASS frequency ID", sat_labels)),
                )
            } else {
                (None, None, None, None, None, None, None)
            };

        let pps_hist = if config.pps_histogram {
            let buckets = make_pps_buckets(config.pps_bucket_size, config.pps_bucket_count);
            let h = HistogramVec::new(
                HistogramOpts::new("gpsd_pps_histogram", "PPS clock offset in nanoseconds")
                    .buckets(buckets),
                &["device"],
            )
            .expect("histogram can be created");
            registry.register(Box::new(h.clone())).expect("collector can be registered");
            Some(h)
        } else {
            None
        };

        let (geo_offset_hist, geo_bearing_x_hist, geo_bearing_y_hist) = if config.geo_offset {
            let offset_buckets = make_geo_offset_buckets(config.geo_bucket_size, config.geo_bucket_count);
            let yx_buckets = make_geo_yx_buckets(config.geo_bucket_size, config.geo_bucket_count);

            let offset = Histogram::with_opts(
                HistogramOpts::new("gpsd_geo_offset_m_histogram", "Geo offset from reference point in meters")
                    .buckets(offset_buckets),
            )
            .expect("histogram can be created");
            registry.register(Box::new(offset.clone())).expect("collector can be registered");

            let bx = Histogram::with_opts(
                HistogramOpts::new("gpsd_geo_bearing_x_histogram", "X offset in meters from static geo point")
                    .buckets(yx_buckets.clone()),
            )
            .expect("histogram can be created");
            registry.register(Box::new(bx.clone())).expect("collector can be registered");

            let by = Histogram::with_opts(
                HistogramOpts::new("gpsd_geo_bearing_y_histogram", "Y offset in meters from static geo point")
                    .buckets(yx_buckets),
            )
            .expect("histogram can be created");
            registry.register(Box::new(by.clone())).expect("collector can be registered");

            (Some(offset), Some(bx), Some(by))
        } else {
            (None, None, None)
        };

        Self {
            config,

            // TPV
            lat: gauge!(registry, "gpsd_lat", "Latitude in degrees: +/- signifies North/South"),
            lon: gauge!(registry, "gpsd_long", "Longitude in degrees: +/- signifies East/West"),
            alt_hae: gauge!(registry, "gpsd_altHAE", "Altitude, height above ellipsoid, in meters"),
            alt_msl: gauge!(registry, "gpsd_altMSL", "MSL Altitude in meters"),
            mode: gauge!(registry, "gpsd_mode", "NMEA mode: 0=unknown, 1=no fix, 2=2D, 3=3D"),
            status: gauge!(registry, "gpsd_status", "GPS fix status: 2=DGPS, 3=RTK Fixed, 4=RTK Float"),
            leapseconds: gauge!(registry, "gpsd_leapseconds", "Current leap seconds"),
            magvar: gauge!(registry, "gpsd_magvar", "Magnetic variation, degrees"),
            ept: gauge!(registry, "gpsd_ept", "Estimated timestamp error in seconds"),
            epx: gauge!(registry, "gpsd_epx", "Longitude error estimate in meters"),
            epy: gauge!(registry, "gpsd_epy", "Latitude error estimate in meters"),
            epv: gauge!(registry, "gpsd_epv", "Estimated vertical error in meters"),
            eps: gauge!(registry, "gpsd_eps", "Estimated speed error in meters per second"),
            epc: gauge!(registry, "gpsd_epc", "Estimated climb error in meters per second"),
            epd: gauge!(registry, "gpsd_epd", "Estimated track direction error in degrees"),
            eph: gauge!(registry, "gpsd_eph", "Estimated horizontal position error in meters"),
            sep: gauge!(registry, "gpsd_sep", "Estimated spherical position error in meters"),
            geoid_sep: gauge!(registry, "gpsd_geoidSep", "Geoid separation in meters"),
            ecefx: gauge!(registry, "gpsd_ecefx", "ECEF X position in meters"),
            ecefy: gauge!(registry, "gpsd_ecefy", "ECEF Y position in meters"),
            ecefz: gauge!(registry, "gpsd_ecefz", "ECEF Z position in meters"),
            ecefvx: gauge!(registry, "gpsd_ecefvx", "ECEF X velocity in meters per second"),
            ecefvy: gauge!(registry, "gpsd_ecefvy", "ECEF Y velocity in meters per second"),
            ecefvz: gauge!(registry, "gpsd_ecefvz", "ECEF Z velocity in meters per second"),
            ecefp_acc: gauge!(registry, "gpsd_ecefpAcc", "ECEF position error in meters"),
            ecefv_acc: gauge!(registry, "gpsd_ecefvAcc", "ECEF velocity error in meters per second"),
            vel_n: gauge!(registry, "gpsd_velN", "North velocity component in meters"),
            vel_e: gauge!(registry, "gpsd_velE", "East velocity component in meters"),
            vel_d: gauge!(registry, "gpsd_velD", "Down velocity component in meters"),
            speed: gauge!(registry, "gpsd_speed", "Speed over ground, meters per second"),
            track: gauge!(registry, "gpsd_track", "Course over ground, degrees from true north"),
            magtrack: gauge!(registry, "gpsd_magtrack", "Course over ground, degrees magnetic"),
            climb: gauge!(registry, "gpsd_climb", "Climb rate, meters per second"),
            depth: gauge!(registry, "gpsd_depth", "Depth in meters"),
            dgps_age: gauge!(registry, "gpsd_dgpsAge", "Age of DGPS data in seconds"),
            dgps_sta: gauge!(registry, "gpsd_dgpsSta", "Station of DGPS data"),
            rel_n: gauge!(registry, "gpsd_relN", "North component of relative position vector"),
            rel_e: gauge!(registry, "gpsd_relE", "East component of relative position vector"),
            rel_d: gauge!(registry, "gpsd_relD", "Down component of relative position vector"),

            // SKY
            gdop: gauge!(registry, "gpsd_gdop", "Geometric dilution of precision"),
            hdop: gauge!(registry, "gpsd_hdop", "Horizontal dilution of precision"),
            pdop: gauge!(registry, "gpsd_pdop", "Position dilution of precision"),
            tdop: gauge!(registry, "gpsd_tdop", "Time dilution of precision"),
            vdop: gauge!(registry, "gpsd_vdop", "Vertical dilution of precision"),
            xdop: gauge!(registry, "gpsd_xdop", "Latitudinal dilution of precision"),
            ydop: gauge!(registry, "gpsd_ydop", "Longitudinal dilution of precision"),
            n_sat: gauge!(registry, "gpsd_nSat", "Number of satellite objects in skyview"),
            u_sat: gauge!(registry, "gpsd_uSat", "Number of satellites used in navigation solution"),
            pr_res: gauge!(registry, "gpsd_prRes", "Pseudorange residue in meters"),
            qual: gauge!(registry, "gpsd_qual", "Quality indicator"),
            sat_used: gauge!(registry, "gpsd_sat_used", "Satellites used in current solution"),
            sat_seen: gauge!(registry, "gpsd_sat_seen", "Satellites seen in current solution"),

            sat_ss, sat_az, sat_el, sat_is_used, sat_health, sat_sigid, sat_freqid,

            // Version / Devices
            version_info: gauge_vec!(registry, "gpsd_version_info", "GPSD version details", &["release", "rev", "proto_major", "proto_minor"]),
            devices_info: gauge_vec!(registry, "gpsd_devices_info", "GPSD device details", &["device", "driver", "subtype", "subtype1", "activated", "flags", "native", "bps", "parity", "stopbits", "cycle", "mincycle"]),

            // GST
            gst_rms: gauge!(registry, "gpsd_gst_rms", "Standard deviation of range inputs to navigation"),
            gst_major: gauge!(registry, "gpsd_gst_major", "Standard deviation of semi-major axis of error ellipse in meters"),
            gst_minor: gauge!(registry, "gpsd_gst_minor", "Standard deviation of semi-minor axis of error ellipse in meters"),
            gst_orient: gauge!(registry, "gpsd_gst_orient", "Orientation of semi-major axis of error ellipse in degrees"),
            gst_lat: gauge!(registry, "gpsd_gst_lat", "Standard deviation of latitude error in meters"),
            gst_lon: gauge!(registry, "gpsd_gst_lon", "Standard deviation of longitude error in meters"),
            gst_alt: gauge!(registry, "gpsd_gst_alt", "Standard deviation of altitude error in meters"),

            // TOFF
            toff_real_sec: gauge!(registry, "gpsd_toff_real_sec", "Seconds from the GPS clock"),
            toff_real_nsec: gauge!(registry, "gpsd_toff_real_nsec", "Nanoseconds from the GPS clock"),
            toff_clock_sec: gauge!(registry, "gpsd_toff_clock_sec", "Seconds from the system clock"),
            toff_clock_nsec: gauge!(registry, "gpsd_toff_clock_nsec", "Nanoseconds from the system clock"),

            // PPS gauges
            pps_real_sec: gauge!(registry, "gpsd_pps_real_sec", "Seconds from the PPS source"),
            pps_real_nsec: gauge!(registry, "gpsd_pps_real_nsec", "Nanoseconds from the PPS source"),
            pps_clock_sec: gauge!(registry, "gpsd_pps_clock_sec", "Seconds from the system clock"),
            pps_clock_nsec: gauge!(registry, "gpsd_pps_clock_nsec", "Nanoseconds from the system clock"),
            pps_precision: gauge!(registry, "gpsd_pps_precision", "NTP style estimate of PPS precision"),
            pps_q_err: gauge!(registry, "gpsd_pps_qErr", "Quantization error of PPS in picoseconds"),

            pps_hist,
            geo_offset_hist,
            geo_bearing_x_hist,
            geo_bearing_y_hist,

            // OSC
            osc_running: gauge!(registry, "gpsd_osc_running", "Oscillator is currently running"),
            osc_reference: gauge!(registry, "gpsd_osc_reference", "Oscillator is receiving GPS PPS signal"),
            osc_disciplined: gauge!(registry, "gpsd_osc_disciplined", "GPS PPS is disciplining local oscillator"),
            osc_delta: gauge!(registry, "gpsd_osc_delta", "Time difference in nanoseconds between oscillator PPS and GPS PPS"),

            registry,
        }
    }

    pub fn process(&self, msg: &GpsdMessage) {
        match msg {
            GpsdMessage::TPV(tpv) => self.process_tpv(tpv),
            GpsdMessage::SKY(sky) => self.process_sky(sky),
            GpsdMessage::PPS(pps) => self.process_pps(pps),
            GpsdMessage::GST(gst) => self.process_gst(gst),
            GpsdMessage::TOFF(toff) => self.process_toff(toff),
            GpsdMessage::OSC(osc) => self.process_osc(osc),
            GpsdMessage::VERSION(v) => self.process_version(v),
            GpsdMessage::DEVICES(d) => self.process_devices(d),
            GpsdMessage::WATCH(_) => {}
        }
    }

    fn set_if_some(gauge: &prometheus::Gauge, value: Option<f64>) {
        if let Some(v) = value {
            gauge.set(v);
        }
    }

    #[allow(clippy::too_many_lines)]
    fn process_tpv(&self, tpv: &crate::gpsd::Tpv) {
        Self::set_if_some(&self.lat, tpv.lat);
        Self::set_if_some(&self.lon, tpv.lon);
        Self::set_if_some(&self.alt_hae, tpv.altHAE);
        Self::set_if_some(&self.alt_msl, tpv.altMSL);
        Self::set_if_some(&self.mode, tpv.mode);
        Self::set_if_some(&self.status, tpv.status);
        Self::set_if_some(&self.leapseconds, tpv.leapseconds);
        Self::set_if_some(&self.magvar, tpv.magvar);
        Self::set_if_some(&self.ept, tpv.ept);
        Self::set_if_some(&self.epx, tpv.epx);
        Self::set_if_some(&self.epy, tpv.epy);
        Self::set_if_some(&self.epv, tpv.epv);
        Self::set_if_some(&self.eps, tpv.eps);
        Self::set_if_some(&self.epc, tpv.epc);
        Self::set_if_some(&self.epd, tpv.epd);
        Self::set_if_some(&self.eph, tpv.eph);
        Self::set_if_some(&self.sep, tpv.sep);
        Self::set_if_some(&self.geoid_sep, tpv.geoidSep);
        Self::set_if_some(&self.ecefx, tpv.ecefx);
        Self::set_if_some(&self.ecefy, tpv.ecefy);
        Self::set_if_some(&self.ecefz, tpv.ecefz);
        Self::set_if_some(&self.ecefvx, tpv.ecefvx);
        Self::set_if_some(&self.ecefvy, tpv.ecefvy);
        Self::set_if_some(&self.ecefvz, tpv.ecefvz);
        Self::set_if_some(&self.ecefp_acc, tpv.ecefpAcc);
        Self::set_if_some(&self.ecefv_acc, tpv.ecefvAcc);
        Self::set_if_some(&self.vel_n, tpv.velN);
        Self::set_if_some(&self.vel_e, tpv.velE);
        Self::set_if_some(&self.vel_d, tpv.velD);
        Self::set_if_some(&self.speed, tpv.speed);
        Self::set_if_some(&self.track, tpv.track);
        Self::set_if_some(&self.magtrack, tpv.magtrack);
        Self::set_if_some(&self.climb, tpv.climb);
        Self::set_if_some(&self.depth, tpv.depth);
        Self::set_if_some(&self.dgps_age, tpv.dgpsAge);
        Self::set_if_some(&self.dgps_sta, tpv.dgpsSta);
        Self::set_if_some(&self.rel_n, tpv.relN);
        Self::set_if_some(&self.rel_e, tpv.relE);
        Self::set_if_some(&self.rel_d, tpv.relD);

        if self.config.geo_offset {
            if let (Some(lat), Some(lon)) = (tpv.lat, tpv.lon) {
                let (dx, dy) = meter_offset_small(self.config.geo_lat, self.config.geo_lon, lat, lon);
                let distance = earth_distance_small(lat, lon, self.config.geo_lat, self.config.geo_lon);
                if let Some(ref h) = self.geo_offset_hist {
                    h.observe(distance);
                }
                if let Some(ref h) = self.geo_bearing_x_hist {
                    h.observe(dx);
                }
                if let Some(ref h) = self.geo_bearing_y_hist {
                    h.observe(dy);
                }
            }
        }
    }

    fn process_sky(&self, sky: &crate::gpsd::Sky) {
        Self::set_if_some(&self.gdop, sky.gdop);
        Self::set_if_some(&self.hdop, sky.hdop);
        Self::set_if_some(&self.pdop, sky.pdop);
        Self::set_if_some(&self.tdop, sky.tdop);
        Self::set_if_some(&self.vdop, sky.vdop);
        Self::set_if_some(&self.xdop, sky.xdop);
        Self::set_if_some(&self.ydop, sky.ydop);
        Self::set_if_some(&self.n_sat, sky.nSat);
        Self::set_if_some(&self.u_sat, sky.uSat);
        Self::set_if_some(&self.pr_res, sky.prRes);
        Self::set_if_some(&self.qual, sky.qual);

        if let Some(ref sats) = sky.satellites {
            let mut seen: f64 = 0.0;
            let mut used: f64 = 0.0;
            for sat in sats {
                seen += 1.0;
                if sat.used.unwrap_or(false) {
                    used += 1.0;
                }
            }
            self.sat_seen.set(seen);
            self.sat_used.set(used);

            if self.config.monitor_satellites {
                self.reset_satellite_vecs();
                for sat in sats {
                    self.process_satellite(sat);
                }
            }
        }
    }

    fn reset_satellite_vecs(&self) {
        if let Some(ref v) = self.sat_ss { v.reset(); }
        if let Some(ref v) = self.sat_az { v.reset(); }
        if let Some(ref v) = self.sat_el { v.reset(); }
        if let Some(ref v) = self.sat_is_used { v.reset(); }
        if let Some(ref v) = self.sat_health { v.reset(); }
        if let Some(ref v) = self.sat_sigid { v.reset(); }
        if let Some(ref v) = self.sat_freqid { v.reset(); }
    }

    fn process_satellite(&self, sat: &Satellite) {
        let prn = sat.PRN.map_or_else(|| "?".to_string(), |v| format!("{}", v as i64));
        let svid = sat.svid.unwrap_or(sat.PRN.unwrap_or(0.0));
        let svid_s = format!("{}", svid as i64);
        let gnssid = sat.gnssid.map_or_else(|| "0".to_string(), |v| format!("{}", v as i64));
        let used = sat.used.map_or("False", |u| if u { "True" } else { "False" });
        let labels = [prn.as_str(), svid_s.as_str(), gnssid.as_str(), used];

        if let (Some(ref gv), Some(v)) = (&self.sat_ss, sat.ss) {
            gv.with_label_values(&labels).set(v);
        }
        if let (Some(ref gv), Some(v)) = (&self.sat_az, sat.az) {
            gv.with_label_values(&labels).set(v);
        }
        if let (Some(ref gv), Some(v)) = (&self.sat_el, sat.el) {
            gv.with_label_values(&labels).set(v);
        }
        if let Some(ref gv) = self.sat_is_used {
            gv.with_label_values(&labels)
                .set(if sat.used.unwrap_or(false) { 1.0 } else { 0.0 });
        }
        if let (Some(ref gv), Some(v)) = (&self.sat_health, sat.health) {
            gv.with_label_values(&labels).set(v);
        }
        if let (Some(ref gv), Some(v)) = (&self.sat_sigid, sat.sigid) {
            gv.with_label_values(&labels).set(v);
        }
        if let (Some(ref gv), Some(v)) = (&self.sat_freqid, sat.freqid) {
            gv.with_label_values(&labels).set(v);
        }
    }

    fn process_pps(&self, pps: &crate::gpsd::Pps) {
        Self::set_if_some(&self.pps_real_sec, pps.real_sec);
        Self::set_if_some(&self.pps_real_nsec, pps.real_nsec);
        Self::set_if_some(&self.pps_clock_sec, pps.clock_sec);
        Self::set_if_some(&self.pps_clock_nsec, pps.clock_nsec);
        Self::set_if_some(&self.pps_precision, pps.precision);
        Self::set_if_some(&self.pps_q_err, pps.q_err);

        if let Some(ref hist) = self.pps_hist {
            if let Some(clock_nsec) = pps.clock_nsec {
                let corr = self.config.pps_time1 * NSEC;
                let mut value = clock_nsec - corr;
                if value > NSEC / 2.0 {
                    value -= NSEC;
                }
                let device = pps.device.as_deref().unwrap_or("unknown");
                hist.with_label_values(&[device]).observe(value);
            }
        }
    }

    fn process_gst(&self, gst: &crate::gpsd::Gst) {
        Self::set_if_some(&self.gst_rms, gst.rms);
        Self::set_if_some(&self.gst_major, gst.major);
        Self::set_if_some(&self.gst_minor, gst.minor);
        Self::set_if_some(&self.gst_orient, gst.orient);
        Self::set_if_some(&self.gst_lat, gst.lat);
        Self::set_if_some(&self.gst_lon, gst.lon);
        Self::set_if_some(&self.gst_alt, gst.alt);
    }

    fn process_toff(&self, toff: &crate::gpsd::Toff) {
        Self::set_if_some(&self.toff_real_sec, toff.real_sec);
        Self::set_if_some(&self.toff_real_nsec, toff.real_nsec);
        Self::set_if_some(&self.toff_clock_sec, toff.clock_sec);
        Self::set_if_some(&self.toff_clock_nsec, toff.clock_nsec);
    }

    fn process_osc(&self, osc: &crate::gpsd::Osc) {
        if let Some(v) = osc.running {
            self.osc_running.set(if v { 1.0 } else { 0.0 });
        }
        if let Some(v) = osc.reference {
            self.osc_reference.set(if v { 1.0 } else { 0.0 });
        }
        if let Some(v) = osc.disciplined {
            self.osc_disciplined.set(if v { 1.0 } else { 0.0 });
        }
        Self::set_if_some(&self.osc_delta, osc.delta);
    }

    fn process_version(&self, v: &crate::gpsd::Version) {
        let release = v.release.as_deref().unwrap_or("unknown");
        let rev = v.rev.as_deref().unwrap_or("unknown");
        let proto_major = v.proto_major.map_or_else(|| "0".to_string(), |n| n.to_string());
        let proto_minor = v.proto_minor.map_or_else(|| "0".to_string(), |n| n.to_string());
        self.version_info
            .with_label_values(&[release, rev, &proto_major, &proto_minor])
            .set(1.0);
    }

    fn process_devices(&self, d: &crate::gpsd::Devices) {
        for dev in &d.devices {
            let path = dev.path.as_deref().unwrap_or("unknown");
            let driver = dev.driver.as_deref().unwrap_or("Unknown");
            let subtype = dev.subtype.as_deref().unwrap_or("Unknown");
            let subtype1 = dev.subtype1.as_deref().unwrap_or("Unknown");
            let activated = dev.activated.as_deref().unwrap_or("Unknown");
            let flags = dev.flags.as_ref().map_or_else(|| "Unknown".to_string(), |v| v.to_string());
            let native = dev.native.as_ref().map_or_else(|| "Unknown".to_string(), |v| v.to_string());
            let bps = dev.bps.as_ref().map_or_else(|| "Unknown".to_string(), |v| v.to_string());
            let parity = dev.parity.as_deref().unwrap_or("Unknown");
            let stopbits = dev.stopbits.as_ref().map_or_else(|| "Unknown".to_string(), |v| v.to_string());
            let cycle = dev.cycle.as_ref().map_or_else(|| "Unknown".to_string(), |v| v.to_string());
            let mincycle = dev.mincycle.as_ref().map_or_else(|| "Unknown".to_string(), |v| v.to_string());
            self.devices_info
                .with_label_values(&[path, driver, subtype, subtype1, activated, &flags, &native, &bps, parity, &stopbits, &cycle, &mincycle])
                .set(1.0);
        }
    }
}

fn earth_distance_small(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let lat1r = lat1.to_radians();
    let lat2r = lat2.to_radians();
    let a = (dlat / 2.0).sin().powi(2) + lat1r.cos() * lat2r.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();
    EARTH_RADIUS_M * c
}

fn meter_offset_small(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> (f64, f64) {
    let mut dx = earth_distance_small(lat1, lon1, lat1, lon2);
    let mut dy = earth_distance_small(lat1, lon1, lat2, lon1);
    if lat1 < lat2 {
        dy = -dy;
    }
    if lon1 < lon2 {
        dx = -dx;
    }
    (dx, dy)
}
