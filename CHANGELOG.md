# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.5](https://github.com/asterust/asterust/compare/asterust-v0.2.4...asterust-v0.2.5) - 2026-11-26

### Other

- fill in and modernize rustdoc for the core asterust crate

## [0.2.4](https://github.com/asterust/asterust/compare/asterust-v0.2.3...asterust-v0.2.4) - 2026-11-26

### Added

- add asterust-junos crate with typed JunOS views

### Other

- Merge pull request #29 from asterust/release-plz-2026-11-26T08-26-51Z
- *(junos)* ground the BGP/interface/OSPF fixtures in Juniper's BGP guide

## [0.2.3](https://github.com/asterust/asterust/compare/asterust-v0.2.2...asterust-v0.2.3) - 2026-11-26

### Other

- Merge pull request #27 from asterust/docs/readme-workspace
- update README for the workspace, platform crates and Python package

## [0.2.2](https://github.com/asterust/asterust/compare/asterust-v0.2.1...asterust-v0.2.2) - 2026-11-26

### Other

- use the default asterust-py-v tag and release names for the Python package
- name the asterust-py GitHub Release after its py-v tag

## [0.2.1](https://github.com/asterust/asterust/compare/asterust-v0.2.0...asterust-v0.2.1) - 2026-11-26

### Added

- add ConfigLine keyword helpers and device traits
- add asterust.iosxe Python module
- add asterust-iosxe crate with typed IOS XE blocks
- add asterust-py PyO3 bindings (config tree + IosXrDevice)
- add asterust-iosxr workspace crate with typed IOS XR blocks

### Other

- automate per-crate releases with release-plz
- publish wheels and sdist to PyPI on py-v* tags
- shorten IosXrDevice getters to interfaces/ospf/bgp/isis
