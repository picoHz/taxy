# Changelog

## v0.3.36 - 2025-03-06

<!-- Release notes generated using configuration in .github/release.yml at v0.3.36 -->
### What's Changed

#### Bug Fixes

* Fix outdated axum router syntax by @picoHz in https://github.com/picoHz/taxy/pull/111

**Full Changelog**: https://github.com/picoHz/taxy/compare/v0.3.35...v0.3.36

## v0.3.35 - 2025-01-15

<!-- Release notes generated using configuration in .github/release.yml at v0.3.35 -->
### What's Changed

#### New Features

* Rewrite Alt-Svc response header according to port configuration by @picoHz in https://github.com/picoHz/taxy/pull/106

#### Other Changes

* Update gloo dependencies to latest versions by @picoHz in https://github.com/picoHz/taxy/pull/104
* Update Yew to version 0.21 and related dependencies by @picoHz in https://github.com/picoHz/taxy/pull/105
* Update dependencies by @picoHz in https://github.com/picoHz/taxy/pull/108

**Full Changelog**: https://github.com/picoHz/taxy/compare/v0.3.34...v0.3.35

## v0.3.34 - 2024-11-16

<!-- Release notes generated using configuration in .github/release.yml at v0.3.34 -->
### What's Changed

#### New Features

* Support regex patterns in virtual hosts by @picoHz in https://github.com/picoHz/taxy/pull/102
* Include target server URL path in proxied requests by @picoHz in https://github.com/picoHz/taxy/pull/103

#### Other Changes

* Migrate from warp to axum for HTTP handling by @picoHz in https://github.com/picoHz/taxy/pull/100
* Migrate to hyper v1 by @picoHz in https://github.com/picoHz/taxy/pull/101

**Full Changelog**: https://github.com/picoHz/taxy/compare/v0.3.33...v0.3.34

## v0.3.33 - 2024-11-11

<!-- Release notes generated using configuration in .github/release.yml at v0.3.33 -->
### What's Changed

#### Bug Fixes

* Fix style errors in webui by @picoHz in https://github.com/picoHz/taxy/pull/97

#### New Features

* Add FreeBSD pre-built binary release workflow by @picoHz in https://github.com/picoHz/taxy/pull/98
* Redirect HTTP requests to available HTTPS ports by default by @picoHz in https://github.com/picoHz/taxy/pull/99

**Full Changelog**: https://github.com/picoHz/taxy/compare/v0.3.32...v0.3.33

## v0.3.32 - 2024-11-10

<!-- Release notes generated using configuration in .github/release.yml at v0.3.32 -->
### What's Changed

#### Bug Fixes

* Always overwrite the HOST header value by @picoHz in https://github.com/picoHz/taxy/pull/95

#### New Features

* Use hickory-resolver for DNS lookup by @picoHz in https://github.com/picoHz/taxy/pull/94
* Add UDP proxy support by @picoHz in https://github.com/picoHz/taxy/pull/96

**Full Changelog**: https://github.com/picoHz/taxy/compare/v0.3.31...v0.3.32

## v0.3.31 - 2024-11-08

<!-- Release notes generated using configuration in .github/release.yml at v0.3.31 -->
### What's Changed

#### New Features

* Support proto and host directive in forwarded header by @picoHz in https://github.com/picoHz/taxy/pull/91
* Send both forwarded and x-forwarded-for headers by default by @picoHz in https://github.com/picoHz/taxy/pull/92
* Support for x-forwarded-host header by @picoHz in https://github.com/picoHz/taxy/pull/93

#### Other Changes

* Update outdated dependencies by @picoHz in https://github.com/picoHz/taxy/pull/88
* build(deps): bump openssl from 0.10.64 to 0.10.66 by @dependabot in https://github.com/picoHz/taxy/pull/89
* Update outdated dependencies by @picoHz in https://github.com/picoHz/taxy/pull/90

**Full Changelog**: https://github.com/picoHz/taxy/compare/v0.3.30...v0.3.31

## v0.3.30 - 2024-08-14

<!-- Release notes generated using configuration in .github/release.yml at v0.3.30 -->
### What's Changed

#### Bug Fixes

* Fix server URL validation and ignore invalid routes by @picoHz in https://github.com/picoHz/taxy/pull/81

#### Other Changes

* Update dependencies by @picoHz in https://github.com/picoHz/taxy/pull/80
* Fix clippy warnings by @picoHz in https://github.com/picoHz/taxy/pull/82
* Update wasm-bindgen crate to v0.2.93 by @picoHz in https://github.com/picoHz/taxy/pull/84

**Full Changelog**: https://github.com/picoHz/taxy/compare/v0.3.29...v0.3.30

## v0.3.29 - 2024-05-07

<!-- Release notes generated using configuration in .github/release.yml at v0.3.29 -->
### What's Changed

#### Bug Fixes

* Support multiple private key formats by @picoHz in https://github.com/picoHz/taxy/pull/74
* Support certificates without Subject Alternate Name by @picoHz in https://github.com/picoHz/taxy/pull/73

#### Other Changes

* Update sqlx version to 0.7.4 by @picoHz in https://github.com/picoHz/taxy/pull/67
* Add Docker installation instructions to docs by @picoHz in https://github.com/picoHz/taxy/pull/68
* Update network-interface version to 2.0.0 by @picoHz in https://github.com/picoHz/taxy/pull/69
* Update brotli version to 6.0.0 by @picoHz in https://github.com/picoHz/taxy/pull/71
* Update utoipa-swagger-ui version to 7.0.0 by @picoHz in https://github.com/picoHz/taxy/pull/72

**Full Changelog**: https://github.com/picoHz/taxy/compare/v0.3.28...v0.3.29

## v0.3.28 - 2024-04-20

<!-- Release notes generated using configuration in .github/release.yml at v0.3.28 -->
### What's Changed

#### Bug Fixes

* Upgrade websocket protocol using existing connection by @picoHz in https://github.com/picoHz/taxy/pull/66

#### New Features

* Always set HOST header in HTTP requests by @picoHz in https://github.com/picoHz/taxy/pull/65

**Full Changelog**: https://github.com/picoHz/taxy/compare/v0.3.27...v0.3.28

## v0.3.27 - 2024-04-18

<!-- Release notes generated using configuration in .github/release.yml at v0.3.27 -->
### What's Changed

#### Bug Fixes

* Preserve original HOST header value in HTTP requests by @picoHz in https://github.com/picoHz/taxy/pull/61

#### New Features

* Add x-forwarded-proto header by @picoHz in https://github.com/picoHz/taxy/pull/60

#### Other Changes

* Update serde_qs to version 0.13.0 and brotli to version 5.0.0 by @picoHz in https://github.com/picoHz/taxy/pull/62
* build(deps): bump whoami from 1.4.1 to 1.5.1 by @dependabot in https://github.com/picoHz/taxy/pull/63
* build(deps): bump h2 from 0.3.24 to 0.3.26 by @dependabot in https://github.com/picoHz/taxy/pull/64

**Full Changelog**: https://github.com/picoHz/taxy/compare/v0.3.26...v0.3.27

## v0.3.26 - 2024-03-29

<!-- Release notes generated using configuration in .github/release.yml at v0.3.26 -->
### What's Changed

#### Bug Fixes

* acme: fix requests not triggered without old certs by @picoHz in https://github.com/picoHz/taxy/pull/56

#### WebUI Updates

* Fix status indicator shrinking issue by @picoHz in https://github.com/picoHz/taxy/pull/58

#### Other Changes

* update rcgen to v0.13.0 by @picoHz in https://github.com/picoHz/taxy/pull/57

**Full Changelog**: https://github.com/picoHz/taxy/compare/v0.3.25...v0.3.26

## v0.3.25 - 2024-03-27

<!-- Release notes generated using configuration in .github/release.yml at v0.3.25 -->
### What's Changed

#### Bug Fixes

* Fix path matching in RequestFilter by @picoHz in https://github.com/picoHz/taxy/pull/53

#### WebUI Updates

* Fix human-friendly duration formatting by @picoHz in https://github.com/picoHz/taxy/pull/51
* Add confirmation dialog for logout by @picoHz in https://github.com/picoHz/taxy/pull/52
* Use vertical tabs on larger screens by @picoHz in https://github.com/picoHz/taxy/pull/54

#### Other Changes

* Remove --insecure-webui option by @picoHz in https://github.com/picoHz/taxy/pull/55

**Full Changelog**: https://github.com/picoHz/taxy/compare/v0.3.24...v0.3.25

## v0.3.24 - 2024-03-24

<!-- Release notes generated using configuration in .github/release.yml at v0.3.24 -->
### What's Changed

#### Bug Fixes

* Fix text style in empty list message by @picoHz in https://github.com/picoHz/taxy/pull/49

#### WebUI Updates

* Use insecure cookie automatically on HTTP by @picoHz in https://github.com/picoHz/taxy/pull/50

#### Other Changes

* Update dependencies by @picoHz in https://github.com/picoHz/taxy/pull/47
* build(deps): bump mio from 0.8.8 to 0.8.11 by @dependabot in https://github.com/picoHz/taxy/pull/48

**Full Changelog**: https://github.com/picoHz/taxy/compare/v0.3.23...v0.3.24

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
