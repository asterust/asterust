//! Extracts the `signal-degrade` thresholds of an OTN interface.
//!
//! Configuration path: `interfaces { <name> { otn-options { signal-degrade { ... } } } }`.
//!
//! # See also
//!
//! - [signal-degrade](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/signal-degrade-edit-interfaces-otn-options-qfx-series.html)

use crate::arg_of;
use asterust::config::ConfigLine;

/// The BER thresholds of `signal-degrade`, e.g. `"7.5E-3"`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct SignalDegrade {
    pub ber_threshold_signal_degrade: Option<String>,
    pub ber_threshold_clear: Option<String>,
    /// Milliseconds.
    pub interval: Option<String>,
}

pub(super) fn parse_signal_degrade(signal_degrade: &ConfigLine) -> SignalDegrade {
    let field = |keyword: &str| arg_of(signal_degrade, keyword).map(str::to_string);
    SignalDegrade {
        ber_threshold_signal_degrade: field("ber-threshold-signal-degrade"),
        ber_threshold_clear: field("ber-threshold-clear"),
        interval: field("interval"),
    }
}
