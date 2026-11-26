//! Extracts the `service-deployment` settings of `system services`.
//!
//! Configuration path: `system { services { service-deployment { ... } } }`.
//!
//! # See also
//!
//! - [service-deployment](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/service-deployment-edit-system.html)

use crate::{arg_of, named_children};
use asterust::config::ConfigLine;

/// A `servers <address>` entry of `service-deployment`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ServiceDeploymentServer {
    pub address: String,
    pub port: Option<String>,
}

/// The `system services service-deployment` configuration, which connects Junos OS to SRC software.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ServiceDeployment {
    pub servers: Vec<ServiceDeploymentServer>,
    pub source_address: Option<String>,
}

pub(super) fn parse_service_deployment(deployment: &ConfigLine) -> ServiceDeployment {
    ServiceDeployment {
        servers: named_children(deployment, "servers")
            .map(|(address, server)| ServiceDeploymentServer {
                address: address.to_string(),
                port: arg_of(server, "port").map(str::to_string),
            })
            .collect(),
        source_address: arg_of(deployment, "source-address").map(str::to_string),
    }
}

#[cfg(test)]
mod tests {
    use crate::system::system;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax page. See the README "Sources".
    #[test]
    fn extracts_service_deployment() {
        let deployment = system(&juniper::parse_braces(
            "system {\n    services {\n        service-deployment {\n            servers 192.0.2.40 {\n                port 3333;\n            }\n            servers 192.0.2.41;\n            source-address 192.0.2.1;\n        }\n    }\n}\n",
        ))
        .unwrap()
        .services
        .unwrap()
        .service_deployment
        .unwrap();
        assert_eq!(deployment.servers.len(), 2);
        assert_eq!(deployment.servers[0].address, "192.0.2.40");
        assert_eq!(deployment.servers[0].port.as_deref(), Some("3333"));
        assert_eq!(deployment.servers[1].port, None);
        assert_eq!(deployment.source_address.as_deref(), Some("192.0.2.1"));
    }

    #[test]
    fn no_service_deployment_is_none() {
        let services = system(&juniper::parse_braces(
            "system {\n    services {\n        ssh;\n    }\n}\n",
        ))
        .unwrap()
        .services
        .unwrap();
        assert!(services.service_deployment.is_none());
    }
}
