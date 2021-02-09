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
    /// Exact expiration time in RFC3389 format
    pub expired_at: String,
    /// Exact expiration time
    pub not_after: DateTime<Utc>,
}

impl CheckResult {
    /// Create a result from domain name and when the check occurred
    ///
    /// ```
    /// # use potential_giggle::CheckResult;
    /// use chrono::Utc;
    /// CheckResult::default("sha512.badssl.com", Utc::now());
    /// ```
    pub fn default(domain_name: &str, checked_at: DateTime<Utc>) -> Self {
        CheckResult {
            ok: false,
            checked_at,
            domain_name: domain_name.to_string(),
            days: 0,
            expired: false,
            expired_at: "".to_string(),
            not_after: Utc.timestamp(0, 0),
        }
    }

    /// Create a result from expired domain name and when the check occurred
    ///
    /// ```
    /// # use potential_giggle::CheckResult;
    /// use chrono::Utc;
    /// CheckResult::new_expired("expired.badssl.com", Utc::now());
    /// ```
    pub fn new_expired(domain_name: &str, checked_at: DateTime<Utc>) -> Self {
        CheckResult {
            ok: false,
            checked_at,
            domain_name: domain_name.to_string(),
            days: 0,
            expired: true,
            expired_at: "".to_string(),
            not_after: Utc.timestamp(0, 0),
        }
    }

    /// Convert result to JSON
    ///
    /// ```
    /// # use potential_giggle::CheckResult;
    /// use chrono::Utc;
    /// let result = CheckResult::default("sha512.badssl.com", Utc::now());
    /// result.to_json();
    /// ```
    pub fn to_json(&self) -> CheckResultJSON {
        CheckResultJSON {
            ok: self.ok,
            days: self.days,
            domain_name: self.domain_name.clone(),
            checked_at: self.checked_at.to_rfc3339(),
            expired: self.expired,
            expired_at: self.expired_at.clone(),
        }
    }
}

impl fmt::Display for CheckResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // [v] certificate of sha512.badssl.com expires in 512 days
        // [x] certificate of expired.badssl.com is expired
        let mut s = Vec::<String>::new();

        if self.expired {
            s.push("[x]".into());
        } else if self.ok {
            s.push("[v]".into());
        } else {
            s.push("[-]".into());
        }

        s.push(format!("certificate of {0}", self.domain_name));

        if self.expired {
            s.push(format!("is expired"));
        } else {
            s.push(format!(
                "expires in {0} days ({1})",
                self.days.to_formatted_string(&Locale::en),
                self.expired_at
            ));
        }

        write!(f, "{}", s.join(" "))
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

/// List of check result in JSON format
pub type CheckResultsJSON = Vec<CheckResultJSON>;
