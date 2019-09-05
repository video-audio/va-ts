# video-audio mpeg-ts muxer/demuxer

[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![Crate](http://meritbadge.herokuapp.com/va-ts)](https://crates.io/crates/va-ts)

MPEG-TS implementation for Rust.

## Overview

sub-table-id:

  - **PAT** - (table-id, transport-stream-id(ext) [, version-number])
  - **PMT** - (table-id, program-number(ext) [, version-number])
  - **SDT** - (table-id, transport-stream-id(ext), original-network-id, version-number)
  - **EIT** - (table-id, service-id(ext), transport-stream-id, original-network-id, version-number)

table-id-extension:

  - **PAT** - transport-stream-id
  - **PMT** - program-number
  - **SDT** - transport-stream-id
  - **EIT** - service-id

## License

va-ts is provided under the MIT license. See [LICENSE](LICENSE).
