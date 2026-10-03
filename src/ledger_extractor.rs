use crate::models::MoneyDateLedger;
use regex::Regex;

/// Extracts structured financial terms, durations, and deadlines from contract text
#[derive(Debug, Clone, Default)]
pub struct LedgerExtractor;

impl LedgerExtractor {
    pub fn new() -> Self {
        Self
    }

    /// Extract canonical money and dates ledger with cross-check warnings
    pub fn extract(&self, text: &str) -> MoneyDateLedger {
        let lower = text.to_lowercase();
        let mut warnings = Vec::new();

        let contract_value = self.extract_contract_value(&lower, text);
        let liability_cap = self.extract_liability_cap(&lower, text);
        let late_fee_rate = self.extract_late_fee(&lower, text);
        let payment_terms_days = self.extract_payment_days(&lower);
        let termination_notice_days = self.extract_notice_days(&lower);
        let cure_period_days = self.extract_cure_days(&lower);

        let (auto_renewal, auto_renewal_opt_out_days) = self.extract_renewal_terms(&lower);

        // Cross-check: Liability cap relative to exposure
        if let Some(ref cap) = liability_cap {
            if cap.contains("unlimited") || cap.contains("no limit") || cap.contains("uncapped") {
                warnings.push("Financial Alert: Liability is explicitly uncapped, exposing signing party to boundless financial claims.".into());
            }
        } else if contract_value.is_some() {
            warnings.push("Financial Alert: Total contract value is specified, but no overall limitation of liability cap was detected.".into());
        }

        // Cross-check: Exorbitant late fee (> 18% annual or > 1.5% monthly)
        if let Some(ref fee) = late_fee_rate {
            if fee.contains("2%") || fee.contains("3%") || fee.contains("24%") || fee.contains("18%") {
                warnings.push(format!("Penalty Alert: Late payment fee rate of {} may exceed statutory interest rate standards under Indian law (Section 74, ICA).", fee));
            }
        }

        // Cross-check: Asymmetric or lengthy notice periods (> 60 days)
        if let Some(notice_days) = termination_notice_days {
            if notice_days >= 90 {
                warnings.push(format!("Operational Alert: Exceptionally long termination notice period of {} days creates vendor lock-in.", notice_days));
            }
        }

        // Cross-check: Auto-renewal trap
        if auto_renewal {
            if let Some(opt_out) = auto_renewal_opt_out_days {
                warnings.push(format!("Contract Trap: Agreement auto-renews automatically unless cancelled {} days prior to term end.", opt_out));
            } else {
                warnings.push("Contract Trap: Agreement contains automatic renewal mechanism without clear opt-out timeline.".into());
            }
        }

        MoneyDateLedger {
            contract_value,
            liability_cap,
            late_fee_rate,
            payment_terms_days,
            termination_notice_days,
            cure_period_days,
            auto_renewal,
            auto_renewal_opt_out_days,
            cross_check_warnings: warnings,
        }
    }

    fn extract_contract_value(&self, _lower: &str, text: &str) -> Option<String> {
        // Regex for currency amounts: INR / Rs / ₹ / USD / $ followed by digits
        let re = Regex::new(r"(?i)(?:total (?:fee|fees|value|amount|contract price|consideration)|fee of|amount of)\s*(?:is|shall be|:)?\s*(?:(?:inr|rs\.?|₹|usd|\$|eur|€)\s*[\d,]+(?:\.\d{2})?|\d[\d,]*\s*(?:lakhs?|crores?|inr|rs))").ok()?;
        if let Some(m) = re.find(text) {
            return Some(m.as_str().trim().to_string());
        }

        // Generic currency finder
        let curr_re = Regex::new(r"(?:₹|INR|Rs\.?|\$|USD)\s*[\d,]+(?:\.\d{2})?").ok()?;
        if let Some(m) = curr_re.find(text) {
            return Some(m.as_str().trim().to_string());
        }

        None
    }

    fn extract_liability_cap(&self, lower: &str, text: &str) -> Option<String> {
        if lower.contains("unlimited liability") || lower.contains("shall not be limited") || lower.contains("without limitation") {
            return Some("UNLIMITED (Uncapped Exposure)".into());
        }

        let cap_re = Regex::new(r"(?i)(?:aggregate liability|total liability|liability cap|liability shall not exceed)\s*(?:shall not exceed|limited to|is capped at|up to)?\s*(?:(?:inr|rs\.?|₹|usd|\$)\s*[\d,]+|the total fees paid|fees paid in the (?:preceding|prior) \d+ months)").ok()?;
        if let Some(m) = cap_re.find(text) {
            return Some(m.as_str().trim().to_string());
        }

        if lower.contains("preceding 12 months") || lower.contains("last 12 months") {
            return Some("Fees paid in preceding 12 months".into());
        }

        None
    }

    fn extract_late_fee(&self, _lower: &str, text: &str) -> Option<String> {
        let fee_re = Regex::new(r"(?i)(?:late (?:fee|interest|payment|charge)|interest rate of)\s*(?:of|at)?\s*(\d+(?:\.\d+)?%\s*(?:per (?:month|annum|year)|monthly|p\.a\.|p\.m\.))").ok()?;
        if let Some(caps) = fee_re.captures(text) {
            if let Some(m) = caps.get(1) {
                return Some(m.as_str().trim().to_string());
            }
        }

        let pct_re = Regex::new(r"(\d+(?:\.\d+)?%\s*(?:per (?:month|annum)|monthly|p\.m\.|p\.a\.))").ok()?;
        if let Some(caps) = pct_re.captures(text) {
            if let Some(m) = caps.get(1) {
                return Some(m.as_str().trim().to_string());
            }
        }

        None
    }

    fn extract_payment_days(&self, lower: &str) -> Option<u32> {
        let pay_re = Regex::new(r"(?:payment|payable|paid|invoice|due)\s*(?:within|in)?\s*(\d+)\s*(?:calendar\s*)?days").ok()?;
        if let Some(caps) = pay_re.captures(lower) {
            if let Some(m) = caps.get(1) {
                return m.as_str().parse::<u32>().ok();
            }
        }
        None
    }

    fn extract_notice_days(&self, lower: &str) -> Option<u32> {
        let notice_re = Regex::new(r"(?:notice|termination|terminate)\s*(?:period of|prior notice of|upon)?\s*(\d+)\s*(?:calendar\s*)?days").ok()?;
        if let Some(caps) = notice_re.captures(lower) {
            if let Some(m) = caps.get(1) {
                return m.as_str().parse::<u32>().ok();
            }
        }
        None
    }

    fn extract_cure_days(&self, lower: &str) -> Option<u32> {
        let cure_re = Regex::new(r"(?:cure|remedy)\s*(?:period of|within)?\s*(\d+)\s*(?:calendar\s*)?days").ok()?;
        if let Some(caps) = cure_re.captures(lower) {
            if let Some(m) = caps.get(1) {
                return m.as_str().parse::<u32>().ok();
            }
        }
        None
    }

    fn extract_renewal_terms(&self, lower: &str) -> (bool, Option<u32>) {
        let auto_renewal = lower.contains("auto-renew")
            || lower.contains("automatically renew")
            || lower.contains("successive terms")
            || lower.contains("automatically extended");

        let opt_out_days = if auto_renewal {
            let opt_re = Regex::new(r"(?:at least|prior to (?:expiration|renewal)|notice of)\s*(\d+)\s*days").ok();
            opt_re.and_then(|re| {
                re.captures(lower)
                    .and_then(|caps| caps.get(1))
                    .and_then(|m| m.as_str().parse::<u32>().ok())
            })
        } else {
            None
        };

        (auto_renewal, opt_out_days)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_money_dates_ledger() {
        let extractor = LedgerExtractor::new();
        let contract = r#"
            The Total Contract Value is INR 1,50,000 payable within 30 days of invoice.
            Late payment fee is 2% per month.
            Either party may terminate upon 90 days prior written notice.
            This Agreement shall automatically renew unless terminated at least 30 days prior.
            Liability shall be limited to fees paid in preceding 12 months.
        "#;

        let ledger = extractor.extract(contract);
        assert!(ledger.contract_value.is_some());
        assert_eq!(ledger.payment_terms_days, Some(30));
        assert_eq!(ledger.termination_notice_days, Some(90));
        assert!(ledger.auto_renewal);
        assert_eq!(ledger.auto_renewal_opt_out_days, Some(30));
        assert!(ledger.cross_check_warnings.len() >= 2);
    }
}
