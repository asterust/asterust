# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
