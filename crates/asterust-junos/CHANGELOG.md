# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.4](https://github.com/asterust/asterust/compare/asterust-junos-v0.3.3...asterust-junos-v0.3.4) - 2026-11-26

### Added

- *(junos)* add interfaces vrrp-group and aggregated-ether-options bfd-liveness-detection

### Other

- Merge pull request #76 from asterust/feat/junos-interfaces-vrrp-bfd
- Merge origin/main into feat/junos-interfaces-vrrp-bfd

## [0.3.3](https://github.com/asterust/asterust/compare/asterust-junos-v0.3.2...asterust-junos-v0.3.3) - 2026-11-26

### Added

- *(junos)* add interfaces unit family filter, rpf-check, policer, sampling and address options

### Other

- Merge pull request #74 from asterust/feat/junos-interfaces-unit-family
- Merge origin/main into feat/junos-interfaces-unit-family

## [0.3.2](https://github.com/asterust/asterust/compare/asterust-junos-v0.3.1...asterust-junos-v0.3.2) - 2026-11-26

### Added

- *(junos)* add interfaces ether-options and aggregated-ether-options

### Other

- Merge pull request #72 from asterust/feat/junos-interfaces-ethernet-lag
- Merge origin/main into feat/junos-interfaces-ethernet-lag

## [0.3.1](https://github.com/asterust/asterust/compare/asterust-junos-v0.3.0...asterust-junos-v0.3.1) - 2026-11-26

### Added

- *(junos)* add interfaces mtu, speed, encapsulation, vlan-tagging and other physical settings

### Other

- Merge pull request #70 from asterust/feat/junos-interfaces-physical-basics

## [0.3.0](https://github.com/asterust/asterust/compare/asterust-junos-v0.2.4...asterust-junos-v0.3.0) - 2026-11-26

### Other

- Merge pull request #68 from asterust/refactor/junos-mirror-config-layout
- Merge origin/main into refactor/junos-mirror-config-layout
- *(junos)* [**breaking**] mirror the config hierarchy in the module tree

## [0.2.4](https://github.com/asterust/asterust/compare/asterust-junos-v0.2.3...asterust-junos-v0.2.4) - 2026-11-26

### Added

- *(junos)* add system extensions, autoinstallation, schema and other niche settings

### Other

- Merge pull request #66 from asterust/feat/junos-system-niche-settings

## [0.2.3](https://github.com/asterust/asterust/compare/asterust-junos-v0.2.2...asterust-junos-v0.2.3) - 2026-11-26

### Added

- *(junos)* add system subscriber-management, static-subscribers and dhcp-local-server

### Other

- Merge remote-tracking branch 'origin/main' into feat/junos-system-subscriber-settings

## [0.2.2](https://github.com/asterust/asterust/compare/asterust-junos-v0.2.1...asterust-junos-v0.2.2) - 2026-11-26

### Added

- *(junos)* add system services web-management, netconf and xnm details
- *(junos)* add system commit, processes, scripts and flags

### Other

- Merge pull request #61 from asterust/feat/junos-system-services-details
- Merge pull request #60 from asterust/feat/junos-system-commit-processes-scripts

## [0.2.1](https://github.com/asterust/asterust/compare/asterust-junos-v0.2.0...asterust-junos-v0.2.1) - 2026-11-26

### Added

- *(junos)* add more system login, syslog and ssh options

### Other

- Merge pull request #58 from asterust/feat/junos-system-login-syslog-ssh-options

## [0.2.0](https://github.com/asterust/asterust/compare/asterust-junos-v0.1.5...asterust-junos-v0.2.0) - 2026-11-26

### Added

- *(junos)* add system archival and accounting

### Other

- Merge pull request #54 from asterust/refactor/junos-system-location-ports
- *(junos)* [**breaking**] model system location and ports as in the config

## [0.1.5](https://github.com/asterust/asterust/compare/asterust-junos-v0.1.4...asterust-junos-v0.1.5) - 2026-11-26

### Added

- *(junos)* add system internet-options and default-address-selection

### Other

- Merge pull request #53 from asterust/feat/junos-system-internet-options
- Merge pull request #52 from asterust/refactor/junos-system-modules
- *(junos)* split system into submodules

## [0.1.4](https://github.com/asterust/asterust/compare/asterust-junos-v0.1.3...asterust-junos-v0.1.4) - 2026-11-26

### Added

- *(junos)* add system location, ports and configuration limits
- *(junos)* add system authentication-order, radius-server and tacplus-server
- *(junos)* add system name resolution and backup-router settings

## [0.1.3](https://github.com/asterust/asterust/compare/asterust-junos-v0.1.2...asterust-junos-v0.1.3) - 2026-11-26

### Added

- *(junos)* add system services outbound-ssh
- *(junos)* add system syslog archive
- *(junos)* add system root-authentication
- *(junos)* add system time-zone
- *(junos)* add system domain-name

### Other

- Merge pull request #47 from asterust/feat/junos-system-outbound-ssh
- Merge pull request #46 from asterust/feat/junos-system-syslog-archive
- Merge pull request #44 from asterust/feat/junos-system-root-authentication
- Merge pull request #43 from asterust/feat/junos-system-time-zone
- Merge pull request #42 from asterust/feat/junos-system-domain-name

## [0.1.2](https://github.com/asterust/asterust/compare/asterust-junos-v0.1.1...asterust-junos-v0.1.2) - 2026-11-26

### Added

- *(junos)* add system services
- *(junos)* add system login

### Other

- Merge pull request #41 from asterust/feat/junos-system-services
- Merge pull request #39 from asterust/feat/junos-system-login

## [0.1.1](https://github.com/asterust/asterust/compare/asterust-junos-v0.1.0...asterust-junos-v0.1.1) - 2026-11-26

### Added

- *(junos)* add system syslog
- *(junos)* add system ntp
- *(junos)* add system name-servers
- *(junos)* add system view with host-name

## [0.1.0](https://github.com/asterust/asterust/releases/tag/asterust-junos-v0.1.0) - 2026-11-26

### Added

- add asterust-junos crate with typed JunOS views

### Other

- *(junos)* ground the BGP/interface/OSPF fixtures in Juniper's BGP guide
