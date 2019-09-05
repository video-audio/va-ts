//! golang style duration format wrapper
use std::cmp;
use std::fmt;
use std::time::Duration;

pub struct DurationFmt(pub Duration);

impl DurationFmt {
    pub fn from_nanos(nanos: u64) -> DurationFmt {
        DurationFmt(Duration::from_nanos(nanos))
    }

    #[inline(always)]
    fn duration(&self) -> Duration {
        self.0
    }

    #[inline(always)]
    fn pure_nanos(&self) -> u128 {
        self.0.as_nanos() % Duration::from_micros(1).as_nanos()
    }

    #[inline(always)]
    fn pure_micros(&self) -> u128 {
        (self.0.as_nanos() % Duration::from_millis(1).as_nanos())
            / Duration::from_micros(1).as_nanos()
    }

    #[inline(always)]
    fn pure_millis(&self) -> u128 {
        (self.0.as_nanos() % Duration::from_secs(1).as_nanos())
            / Duration::from_millis(1).as_nanos()
    }

    #[inline(always)]
    fn pure_secs_as_f64(&self) -> f64 {
        ((self.0.as_nanos() % Duration::from_secs(60).as_nanos()) as f64)
            / (Duration::from_secs(1).as_nanos() as f64)
    }

    #[inline(always)]
    fn pure_mins(&self) -> u128 {
        (self.0.as_nanos() % Duration::from_secs(60 * 60).as_nanos())
            / Duration::from_secs(60).as_nanos()
    }

    #[inline(always)]
    fn pure_hours(&self) -> u128 {
        self.0.as_nanos() / Duration::from_secs(60 * 60).as_nanos()
    }
}

impl cmp::PartialEq for DurationFmt {
    fn eq(&self, other: &Self) -> bool {
        self.duration() == other.duration()
    }
}

impl From<Duration> for DurationFmt {
    #[inline(always)]
    fn from(d: Duration) -> Self {
        DurationFmt(d)
    }
}

impl fmt::Display for DurationFmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.duration() {
            d if d <= Duration::from_micros(1) => write!(f, "{}ns", self.pure_nanos()),
            d if d <= Duration::from_millis(1) => {
                let ns = self.pure_nanos();
                let mcs = self.pure_micros();

                match ns {
                    0 => write!(f, "{}us", mcs),
                    _ => write!(f, "{}us{}ns", mcs, ns),
                }
            }
            d if d <= (Duration::from_secs(1) / 10) => {
                let mcs = self.pure_micros();
                let ms = self.pure_millis();

                match mcs {
                    0 => write!(f, "{}ms", ms),
                    _ => write!(f, "{}ms{}us", ms, mcs),
                }
            }
            _ => {
                let h = self.pure_hours();
                let m = self.pure_mins();
                let s = self.pure_secs_as_f64();

                if h != 0 {
                    write!(f, "{}h", h)?;
                }

                if m != 0 {
                    write!(f, "{}m", m)?;
                }

                if s != 0.0 {
                    write!(f, "{:.2}s", s)
                } else {
                    Ok(())
                }
            }
        }
    }
}

impl fmt::Debug for DurationFmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::DurationFmt;

    use std::time::Duration;

    #[test]
    fn fmt_ns() {
        assert_eq!(format!("{}", DurationFmt::from_nanos(1)), "1ns");
    }

    #[test]
    fn fmt_h_m_s() {
        assert_eq!(
            format!(
                "{}",
                DurationFmt::from(
                    Duration::from_secs(10*3600) + // 10h
                    Duration::from_secs(30*60) + // 30m
                    Duration::from_secs(15) + // 15s
                    Duration::from_millis(100) // 0.1s
                )
            ),
            "10h30m15.10s"
        );
    }

    #[test]
    fn fmt_ms_us() {
        assert_eq!(
            format!(
                "{}",
                DurationFmt::from(
                    Duration::from_millis(23) + // 23ms
                    Duration::from_micros(17) + // 17us
                    Duration::from_nanos(40) // 40ns
                )
            ),
            "23ms17us"
        );
    }
}
