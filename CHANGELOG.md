# Changelog

## v0.3.23 - 2024-02-01

<!-- Release notes generated using configuration in .github/release.yml at v0.3.23 -->
### What's Changed

#### New Features

* Show acme renewal date on WebUI by @picoHz in https://github.com/picoHz/taxy/pull/42

#### WebUI Updates

* Add dark theme support by @picoHz in https://github.com/picoHz/taxy/pull/40
* Fix loading indicator bahovior by @picoHz in https://github.com/picoHz/taxy/pull/41

#### Other Changes

* Update dependencies around tungstenite by @picoHz in https://github.com/picoHz/taxy/pull/43
* build(deps): bump h2 from 0.3.20 to 0.3.24 by @dependabot in https://github.com/picoHz/taxy/pull/44
* build(deps): bump rustix from 0.37.23 to 0.37.27 by @dependabot in https://github.com/picoHz/taxy/pull/45

**Full Changelog**: https://github.com/picoHz/taxy/compare/v0.3.22...v0.3.23

## v0.3.22 - 2024-01-25

<!-- Release notes generated using configuration in .github/release.yml at v0.3.22 -->
### What's Changed

#### Bug Fixes

* Support wss scheme in target server URL by @picoHz in https://github.com/picoHz/taxy/pull/37
* Use fmt::Display instead of fmt::Debug in logging if possible by @picoHz in https://github.com/picoHz/taxy/pull/38

#### New Features

* Add error logging to HTTP requests by @picoHz in https://github.com/picoHz/taxy/pull/39

#### Other Changes

* Improve instructions for setting up development environment by @picoHz in https://github.com/picoHz/taxy/pull/36

**Full Changelog**: https://github.com/picoHz/taxy/compare/v0.3.21...v0.3.22

## v0.3.21 - 2024-01-24

<!-- Release notes generated using configuration in .github/release.yml at v0.3.21 -->
### What's Changed

#### Bug Fixes

* Fix self-sign not working without existing CA certs by @picoHz in https://github.com/picoHz/taxy/pull/32

#### WebUI Updates

* Update port/proxy name labels by @picoHz in https://github.com/picoHz/taxy/pull/33
* Add loading indicator to list components by @picoHz in https://github.com/picoHz/taxy/pull/34
* Add certificate expiry date to certificate list by @picoHz in https://github.com/picoHz/taxy/pull/35

**Full Changelog**: https://github.com/picoHz/taxy/compare/v0.3.20...v0.3.21

## v0.3.20 - 2024-01-24

<!-- Release notes generated using configuration in .github/release.yml at v0.3.20 -->
### What's Changed

#### Bug Fixes

* Fix vhost matching in http proxy by @picoHz in https://github.com/picoHz/taxy/pull/31

#### New Features

* Support dark theme on error page by @picoHz in https://github.com/picoHz/taxy/pull/30

**Full Changelog**: https://github.com/picoHz/taxy/compare/v0.3.19...v0.3.20

## v0.3.19 - 2024-01-23

<!-- Release notes generated using configuration in .github/release.yml at v0.3.19 -->
### What's Changed

#### Bug Fixes

* Fix trailing slash preservation in URI by @picoHz in https://github.com/picoHz/taxy/pull/28
* Fix vhost resolution issue in HTTP/2 by @picoHz in https://github.com/picoHz/taxy/pull/29

**Full Changelog**: https://github.com/picoHz/taxy/compare/v0.3.18...v0.3.19

## v0.3.18 - 2024-01-22

<!-- Release notes generated using configuration in .github/release.yml at v0.3.18 -->
### What's Changed

#### Bug Fixes

* Fix secure flag in cookie attribute by @picoHz in https://github.com/picoHz/taxy/pull/26

#### New Features

* Add --insecure-webui option to taxy start command by @picoHz in https://github.com/picoHz/taxy/pull/27

#### Other Changes

* upgrade toml and toml_edit by @picoHz in https://github.com/picoHz/taxy/pull/18
* Update dependencies in Cargo.toml files by @picoHz in https://github.com/picoHz/taxy/pull/23
* Upgrade tokio-rustls to v0.25.0 by @picoHz in https://github.com/picoHz/taxy/pull/24
* Fix clippy warnings by @picoHz in https://github.com/picoHz/taxy/pull/25

**Full Changelog**: https://github.com/picoHz/taxy/compare/v0.3.17...v0.3.18

## v0.3.17 - 2023-09-14

<!-- Release notes generated using configuration in .github/release.yml at v0.3.17 -->
### What's Changed

#### Bug Fixes

- ServerState: filter out inactive proxies properly by @picoHz in https://github.com/picoHz/taxy/pull/16

#### New Features

- ACME: add feature to activate or inactivate periodic ACME requests by @picoHz in https://github.com/picoHz/taxy/pull/12
- add feature to activate or inactivate proxies by @picoHz in https://github.com/picoHz/taxy/pull/13
- add feature to activate or inactivate ports by @picoHz in https://github.com/picoHz/taxy/pull/14
- Proxy: add status notification by @picoHz in https://github.com/picoHz/taxy/pull/17

#### WebUI Updates

- webui: fix table layout by @picoHz in https://github.com/picoHz/taxy/pull/15

**Full Changelog**: https://github.com/picoHz/taxy/compare/v0.3.16...v0.3.17

## v0.3.16 - 2023-09-03

<!-- Release notes generated using configuration in .github/release.yml at v0.3.16 -->
### What's Changed

#### Bug Fixes

- webui: fix logview scroll-lock algorithm by @picoHz in https://github.com/picoHz/taxy/pull/6

#### WebUI Updates

- webui: set http as a default port protocol by @picoHz in https://github.com/picoHz/taxy/pull/5
- webui: add pre-defined acme providers by @picoHz in https://github.com/picoHz/taxy/pull/10

#### Other Changes

- build(deps): bump rustls-webpki from 0.101.1 to 0.101.4 by @dependabot in https://github.com/picoHz/taxy/pull/7
- upgrade instant-acme to v0.4.0 by @picoHz in https://github.com/picoHz/taxy/pull/8
- certs: remove is_trusted attribute by @picoHz in https://github.com/picoHz/taxy/pull/9
- upgrade webpki to v0.22.1 by @picoHz in https://github.com/picoHz/taxy/pull/11

### New Contributors

- @dependabot made their first contribution in https://github.com/picoHz/taxy/pull/7

**Full Changelog**: https://github.com/picoHz/taxy/compare/v0.3.15...v0.3.16

## v0.3.15 - 2023-08-04

<!-- Release notes generated using configuration in .github/release.yml at v0.3.15 -->
### What's Changed

#### Bug Fixes

- https: return status 421 for domain-fronting requests by @picoHz in https://github.com/picoHz/taxy/pull/3

#### Other Changes

- config: record PKG_VERSION in config files by @picoHz in https://github.com/picoHz/taxy/pull/4

### New Contributors

- @picoHz made their first contribution in https://github.com/picoHz/taxy/pull/3

**Full Changelog**: https://github.com/picoHz/taxy/compare/v0.3.14...v0.3.15
