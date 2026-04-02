use prometheus::{GaugeVec, Opts, Registry};

use crate::gpsd::{GpsdMessage, Satellite};

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

pub struct Metrics {
    pub registry: Registry,

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

    // Per-satellite
    pub sat_ss: GaugeVec,
    pub sat_az: GaugeVec,
    pub sat_el: GaugeVec,
    pub sat_is_used: GaugeVec,
    pub sat_health: GaugeVec,
    pub sat_sigid: GaugeVec,
    pub sat_freqid: GaugeVec,

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

    // PPS
    pub pps_real_sec: prometheus::Gauge,
    pub pps_real_nsec: prometheus::Gauge,
    pub pps_clock_sec: prometheus::Gauge,
    pub pps_clock_nsec: prometheus::Gauge,
    pub pps_precision: prometheus::Gauge,
    pub pps_q_err: prometheus::Gauge,

    // OSC
    pub osc_running: prometheus::Gauge,
    pub osc_reference: prometheus::Gauge,
    pub osc_disciplined: prometheus::Gauge,
    pub osc_delta: prometheus::Gauge,
}

impl Metrics {
    #[allow(clippy::too_many_lines)]
    pub fn new() -> Self {
        let registry = Registry::new();

        Self {
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

            // Per-satellite
            sat_ss: gauge_vec!(registry, "gpsd_sat_ss", "Signal to noise ratio in dBHz", &["PRN", "svid", "gnssid", "used"]),
            sat_az: gauge_vec!(registry, "gpsd_sat_az", "Azimuth, degrees from true north", &["PRN", "svid", "gnssid", "used"]),
            sat_el: gauge_vec!(registry, "gpsd_sat_el", "Elevation in degrees", &["PRN", "svid", "gnssid", "used"]),
            sat_is_used: gauge_vec!(registry, "gpsd_used", "Used satellite", &["PRN", "svid", "gnssid", "used"]),
            sat_health: gauge_vec!(registry, "gpsd_health", "Satellite health: 0=unknown, 1=OK, 2=unhealthy", &["PRN", "svid", "gnssid", "used"]),
            sat_sigid: gauge_vec!(registry, "gpsd_sat_sigid", "Signal ID", &["PRN", "svid", "gnssid", "used"]),
            sat_freqid: gauge_vec!(registry, "gpsd_sat_freqid", "GLONASS frequency ID", &["PRN", "svid", "gnssid", "used"]),

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

            // PPS
            pps_real_sec: gauge!(registry, "gpsd_pps_real_sec", "Seconds from the PPS source"),
            pps_real_nsec: gauge!(registry, "gpsd_pps_real_nsec", "Nanoseconds from the PPS source"),
            pps_clock_sec: gauge!(registry, "gpsd_pps_clock_sec", "Seconds from the system clock"),
            pps_clock_nsec: gauge!(registry, "gpsd_pps_clock_nsec", "Nanoseconds from the system clock"),
            pps_precision: gauge!(registry, "gpsd_pps_precision", "NTP style estimate of PPS precision"),
            pps_q_err: gauge!(registry, "gpsd_pps_qErr", "Quantization error of PPS in picoseconds"),

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

            self.reset_satellite_vecs();
            for sat in sats {
                self.process_satellite(sat);
            }
        }
    }

    fn reset_satellite_vecs(&self) {
        self.sat_ss.reset();
        self.sat_az.reset();
        self.sat_el.reset();
        self.sat_is_used.reset();
        self.sat_health.reset();
        self.sat_sigid.reset();
        self.sat_freqid.reset();
    }

    fn process_satellite(&self, sat: &Satellite) {
        let prn = sat.PRN.map_or_else(|| "?".to_string(), |v| format!("{}", v as i64));
        let svid = sat.svid.unwrap_or(sat.PRN.unwrap_or(0.0));
        let svid_s = format!("{}", svid as i64);
        let gnssid = sat.gnssid.map_or_else(|| "0".to_string(), |v| format!("{}", v as i64));
        let used = sat.used.map_or("False", |u| if u { "True" } else { "False" });
        let labels = [prn.as_str(), svid_s.as_str(), gnssid.as_str(), used];

        if let Some(v) = sat.ss {
            self.sat_ss.with_label_values(&labels).set(v);
        }
        if let Some(v) = sat.az {
            self.sat_az.with_label_values(&labels).set(v);
        }
        if let Some(v) = sat.el {
            self.sat_el.with_label_values(&labels).set(v);
        }
        self.sat_is_used
            .with_label_values(&labels)
            .set(if sat.used.unwrap_or(false) { 1.0 } else { 0.0 });
        if let Some(v) = sat.health {
            self.sat_health.with_label_values(&labels).set(v);
        }
        if let Some(v) = sat.sigid {
            self.sat_sigid.with_label_values(&labels).set(v);
        }
        if let Some(v) = sat.freqid {
            self.sat_freqid.with_label_values(&labels).set(v);
        }
    }

    fn process_pps(&self, pps: &crate::gpsd::Pps) {
        Self::set_if_some(&self.pps_real_sec, pps.real_sec);
        Self::set_if_some(&self.pps_real_nsec, pps.real_nsec);
        Self::set_if_some(&self.pps_clock_sec, pps.clock_sec);
        Self::set_if_some(&self.pps_clock_nsec, pps.clock_nsec);
        Self::set_if_some(&self.pps_precision, pps.precision);
        Self::set_if_some(&self.pps_q_err, pps.q_err);
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
