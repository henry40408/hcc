use std::fmt;

use chrono::{DateTime, TimeZone, Utc};
use num_format::{Locale, ToFormattedString};
use serde::{Deserialize, Serialize};

/// Check result
#[derive(Debug)]
pub struct CheckResult {
    /// True when SSL certificate is valid in grace period of days
    pub ok: bool,
    /// When is domain name got checked
    pub checked_at: DateTime<Utc>,
    /// Remaining days to the expiration date
    pub days: i64,
    /// Domain name that got checked
    pub domain_name: String,
    /// Already expired?
    pub expired: bool,
    /// Exact expiration time
    pub not_after: DateTime<Utc>,
}

impl CheckResult {
    /// Create a result from domain name and when the check occurred
    ///
    /// ```
    /// # use potential_giggle::CheckResult;
    /// use chrono::Utc;
    /// CheckResult::default("sha512.badssl.com", &Utc::now());
    /// ```
    pub fn default(domain_name: &str, checked_at: &DateTime<Utc>) -> Self {
        CheckResult {
            ok: false,
            checked_at: checked_at.clone(),
            domain_name: domain_name.to_string(),
            days: 0,
            expired: false,
            not_after: Utc.timestamp(0, 0),
        }
    }

    /// Create a result from expired domain name and when the check occurred
    ///
    /// ```
    /// # use potential_giggle::CheckResult;
    /// use chrono::Utc;
    /// CheckResult::new_expired("expired.badssl.com", &Utc::now());
    /// ```
    pub fn new_expired(domain_name: &str, checked_at: &DateTime<Utc>) -> Self {
        CheckResult {
            ok: false,
            checked_at: checked_at.clone(),
            domain_name: domain_name.to_string(),
            days: 0,
            expired: true,
            not_after: Utc.timestamp(0, 0),
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
            let ts = self.not_after.to_rfc3339();
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
    /// # use potential_giggle::{CheckResult, CheckResultJSON};
    /// use chrono::Utc;
    /// let result = CheckResult::default("sha512.badssl.com", &Utc::now());
    /// CheckResultJSON::new(&result);
    /// ```
    pub fn new(result: &CheckResult) -> CheckResultJSON {
        CheckResultJSON {
            ok: result.ok,
            days: result.days,
            domain_name: result.domain_name.clone(),
            checked_at: result.checked_at.to_rfc3339(),
            expired: result.expired,
            expired_at: result.not_after.to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::CheckResult;
    use chrono::{Duration, SubsecRound, Utc};

    fn build_result() -> CheckResult {
        let days = 512;
        let now = Utc::now().round_subsecs(0);
        let expired_at = now + Duration::days(days);
        CheckResult {
            ok: false,
            checked_at: now,
            days,
            domain_name: "example.com".to_string(),
            expired: false,
            not_after: expired_at,
        }
    }

    #[test]
    fn test_display() {
        let mut result = build_result();
        result.ok = true;

        let left = format!("{0}", result);
        let right = format!(
            "[v] certificate of example.com expires in 512 days ({0})",
            result.not_after.to_rfc3339()
        );
        assert_eq!(left, right);
    }

    #[test]
    fn test_display_not_ok() {
        let result = build_result();
        let left = format!("{0}", result);
        let right = format!(
            "[-] certificate of example.com expires in 512 days ({0})",
            result.not_after.to_rfc3339()
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
