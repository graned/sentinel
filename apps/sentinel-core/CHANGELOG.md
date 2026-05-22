# Sentinel Core Changelog

## v1.3.2 (in progress)

### Fixed
- **Token Exchange**: Federated user creation now properly splits `display_name` into `first_name`/`last_name` (#17)
  - Previously: `first_name: None, last_name: None` caused DB constraint violation
  - Now: `display_name` is split on first space, populating both fields
  - Fixes 401 AUTH_ERROR when creating new federated users via `/v1/api/auth/token/exchange`

## v1.3.1
- Initial federated user support (broken - see v1.3.2)

## v1.3.0
- Added token federation provider
