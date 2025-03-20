#!/usr/bin/env bash

# note: vigem_target_ds4_update_ex is blocked becaues the struct gets marked as packed which is bad
# note: bindgen for i686 because "stdcall" is translated as "C" in x86_64
# also don't forget to `cargo install bindgen`
bindgen "wrapper.h" \
  --allowlist-function '^vigem_.+' \
  --allowlist-type '^(VIGEM_|EVT_|XUSB).+' \
  --blocklist-function 'vigem_target_ds4_update_ex' \
  --blocklist-type '.?DS4_REPORT_EX.*' \
  --output "src/bindings.rs" \
  --rustfmt-configuration-file "$(pwd)/../.rustfmt.toml" \
  -- -IVigEmClient/include -target i686-pc-windows-msvc