use std::fmt;

use chrono::{DateTime, TimeZone, Utc};
use num_format::{Locale, ToFormattedString};
use serde::{Deserialize, Serialize};

/// Check result
#[derive(Debug, Default)]
pub struct CheckResult {
    /// True when SSL certificate is valid in grace period of days
    pub ok: bool,
    /// When is domain name got checked in seconds since Unix epoch
    pub checked_at: i64,
    /// Remaining days to the expiration date
    pub days: i64,
    /// Domain name that got checked
    pub domain_name: String,
    /// Already expired?
    pub expired: bool,
    /// Exact expiration time in seconds since Unix epoch
    pub not_after: i64,
}

impl CheckResult {
    /// Create a result from expired domain name and when the check occurred
    ///
    /// ```
    /// # use hcc::CheckResult;
    /// use chrono::Utc;
    /// CheckResult::expired("expired.badssl.com", &Utc::now());
    /// ```
    pub fn expired(domain_name: &str, checked_at: &DateTime<Utc>) -> Self {
        CheckResult {
            checked_at: checked_at.timestamp(),
            domain_name: domain_name.to_string(),
            expired: true,
            ..Default::default()
        }
    }
}

impl fmt::Display for CheckResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // [v] certificate of sha512.badssl.com expires in 512 days (timestamp in RFC3339)
        // [-] certificate of sha512.badssl.com expires in 512 days (timestamp in RFC3339)
        // [x] certificate of expired.badssl.com has expired (timestamp in RFC3339)
        let mut s = String::with_capacity(100);

        if self.expired {
            s.push_str("[x]");
        } else if self.ok {
            s.push_str("[v]");
        } else {
            s.push_str("[-]");
        }

        s.push(' ');

        s.push_str(&format!("certificate of {0}", self.domain_name));

        s.push(' ');

        if self.expired {
            s.push_str("has expired");
        } else {
            let days = self.days.to_formatted_string(&Locale::en);
            let ts = Utc.timestamp(self.not_after, 0).to_rfc3339();
            let s2 = &format!("expires in {0} days ({1})", days, ts);
            s.push_str(s2);
        }

        write!(f, "{}", s)
    }
}

/// Check result in JSON format
#[derive(Serialize, Deserialize)]
pub struct CheckResultJSON {
    /// True when SSL certificate is valid in grace in period
    pub ok: bool,
    /// When is the domain name got checked
    pub checked_at: String,
    /// Remaining days to the expiration date
    pub days: i64,
    /// Domain name that got checked
    pub domain_name: String,
    /// Already expired?
    pub expired: bool,
    /// Expiration time in RFC3389 format
    pub expired_at: String,
}

impl CheckResultJSON {
    /// Convert result to JSON
    ///
    /// ```
    /// # use hcc::{CheckResult, CheckResultJSON};
    /// use chrono::Utc;
    /// let result = CheckResult {
    ///     domain_name: "sha512.badssl.com".into(),
    ///     checked_at: Utc::now().timestamp(),
    ///     ..Default::default()
    /// };
    /// CheckResultJSON::new(&result);
    /// ```
    pub fn new(result: &CheckResult) -> CheckResultJSON {
        CheckResultJSON {
            ok: result.ok,
            days: result.days,
            domain_name: result.domain_name.clone(),
            checked_at: Utc.timestamp(result.checked_at, 0).to_rfc3339(),
            expired: result.expired,
            expired_at: Utc.timestamp(result.not_after, 0).to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod test {
    use chrono::{Duration, SubsecRound, TimeZone, Utc};

    use crate::CheckResult;

    fn build_result() -> CheckResult {
        let days = 512;
        let now = Utc::now().round_subsecs(0);
        let expired_at = now + Duration::days(days);
        CheckResult {
            checked_at: now.timestamp(),
            days,
            domain_name: "example.com".to_string(),
            not_after: expired_at.timestamp(),
            ..Default::default()
        }
    }

    #[test]
    fn test_display() {
        let mut result = build_result();
        result.ok = true;

        let left = format!("{0}", result);
        let right = format!(
            "[v] certificate of example.com expires in 512 days ({0})",
            Utc.timestamp(result.not_after, 0).to_rfc3339()
        );
        assert_eq!(left, right);
    }

    #[test]
    fn test_display_not_ok() {
        let result = build_result();
        let left = format!("{0}", result);
        let right = format!(
            "[-] certificate of example.com expires in 512 days ({0})",
            Utc.timestamp(result.not_after, 0).to_rfc3339()
        );
        assert_eq!(left, right);
    }

    #[test]
    fn test_display_expired() {
        let mut result = build_result();
        result.expired = true;
        let left = format!("{0}", result);
        let right = "[x] certificate of example.com has expired";
        assert_eq!(left, right);
    }
}
