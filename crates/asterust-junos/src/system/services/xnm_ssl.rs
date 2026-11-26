//! `xnm-ssl`
//! `system { services { xnm-ssl { ... } } }`.
//!
//! # See also
//!
//! - [xnm-ssl](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/xnm-ssl-edit-system.html)

use crate::arg_of;
use asterust::config::ConfigLine;

/// The `xnm-ssl` block.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct XnmSsl {
    pub port: Option<String>,
    pub connection_limit: Option<String>,
    pub rate_limit: Option<String>,
    pub local_certificate: Option<String>,
}

pub(super) fn parse_xnm_ssl(xnm: &ConfigLine) -> XnmSsl {
    XnmSsl {
        port: arg_of(xnm, "port").map(str::to_string),
        connection_limit: arg_of(xnm, "connection-limit").map(str::to_string),
        rate_limit: arg_of(xnm, "rate-limit").map(str::to_string),
        local_certificate: arg_of(xnm, "local-certificate").map(str::to_string),
    }
}
