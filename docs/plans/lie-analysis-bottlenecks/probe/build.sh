#!/usr/bin/env bash
set -euo pipefail

if [[ $# != 1 ]]; then
  printf 'Usage: bash %s <scratch-directory>\n' "$0" >&2
  exit 2
fi

probe=$(cd "$(dirname "$0")" && pwd -P)
repo=$(git -C "$probe" rev-parse --show-toplevel)
work=$(cd "$1" && pwd -P)
mkdir "$work/bin"

build() {
  local snapshot=$1 name=$2 output=$3
  cargo build --locked --release --manifest-path "$snapshot/Cargo.toml" \
    --target-dir "$snapshot/target" -p examples --no-default-features \
    --features lie --bin "$name"
  cp "$snapshot/target/release/$name" "$work/bin/$output"
}

for revision in pre post; do
  case "$revision" in
    pre) commit=3cadca39d6e79ae3209c305487d6b277f2bc472b ;;
    post) commit=c26ea49c80ec72fb9c6f166c03bff3fba56e39a7 ;;
  esac

  snapshot="$work/$revision"
  mkdir "$snapshot"
  git -C "$repo" archive "$commit" | tar -x -C "$snapshot"
  cp "$probe/dependencies.lock" "$snapshot/Cargo.lock"
  driver="$snapshot/examples/src/bin/lie_analysis_bench.rs"
  cp "$probe/benchmark.rs" "$driver"
  build "$snapshot" lie_analysis_bench "$revision"

  git -C "$snapshot" apply "$probe/closure-diagnostics.patch"
  build "$snapshot" lie_analysis_bench "$revision-diagnostics"
  if [[ $revision == post ]]; then
    git -C "$snapshot" apply "$probe/direct-hs.patch"
    build "$snapshot" lie_analysis_bench direct-diagnostics
    git -C "$snapshot" apply --reverse "$probe/direct-hs.patch"

    cp "$probe/stability_probe.rs" "$snapshot/examples/src/bin/lie_stability_probe.rs"
    build "$snapshot" lie_stability_probe lie-stability
  fi

  cp "$probe/benchmark.rs" "$driver"
  git -C "$snapshot" apply "$probe/hotspots-$revision.patch"
  build "$snapshot" lie_analysis_bench "$revision-profile"
done
