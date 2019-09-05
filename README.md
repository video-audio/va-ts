# video-audio mpeg-ts muxer/demuxer

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
