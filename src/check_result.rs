use std::fmt;

use chrono::{DateTime, TimeZone, Utc};
use num_format::{Locale, ToFormattedString};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct CheckResult {
    pub ok: bool,
    pub checked_at: DateTime<Utc>,
    pub days: i64,
    pub domain_name: String,
    pub expired_at: String,
    pub not_after: DateTime<Utc>,
}

impl CheckResult {
    pub fn new(domain_name: &str, checked_at: DateTime<Utc>) -> CheckResult {
        CheckResult {
            ok: false,
            checked_at,
            domain_name: domain_name.to_string(),
            days: 0,
            expired_at: "".to_string(),
            not_after: Utc.timestamp(0, 0),
        }
    }

    pub fn to_json(&self) -> CheckResultJSON {
        CheckResultJSON {
            ok: self.ok,
            days: self.days,
            domain_name: self.domain_name.clone(),
            checked_at: self.checked_at.to_rfc3339(),
            expired_at: self.expired_at.clone(),
        }
    }
}

impl fmt::Display for CheckResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // [v] certificate of sha512.badssl.com expires in 512 days
        // [x] certificate of expired.badssl.com is expired
        let mut s = Vec::<String>::new();

        if self.ok {
            s.push("[v]".into());
        } else {
            s.push("[x]".into());
        }

        s.push(format!("certificate of {0}", self.domain_name));

        if self.ok {
            s.push(format!(
                "expires in {0} days ({1})",
                self.days.to_formatted_string(&Locale::en),
                self.expired_at
            ));
        } else {
            s.push(format!("is expired"));
        }

        write!(f, "{}", s.join(" "))
    }
}

#[derive(Serialize, Deserialize)]
pub struct CheckResultJSON {
    pub ok: bool,
    pub checked_at: String,
    pub days: i64,
    pub domain_name: String,
    pub expired_at: String,
}

pub type CheckResultsJSON = Vec<CheckResultJSON>;
