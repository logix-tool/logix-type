#!/bin/bash -e

export CARGO_TARGET_DIR=target/coverage
export CARGO_INCREMENTAL=0
export RUSTFLAGS='-Cinstrument-coverage'
export LLVM_PROFILE_FILE="$CARGO_TARGET_DIR/cargo-test-%p-%m.profraw"

if ! which grcov 2> /dev/null
then
  echo "*** ERROR: Failed to locate grcov perhaps you need to run:"
  echo "      cargo install grcov"
  exit 1
fi

rm -f $CARGO_TARGET_DIR/cargo-test-*.profraw
rm -rf $CARGO_TARGET_DIR/html

find . -name "*.rs" -not -wholename "./target/*" -exec touch {} \;

cargo test

grcov . \
  --binary-path $CARGO_TARGET_DIR/debug/deps/ \
  -s . \
  -t html \
  --branch \
  --ignore-not-existing \
  -o $CARGO_TARGET_DIR/html
  #--ignore '../*' \
  #--ignore "/*" \

echo "Now open file://$(pwd)/$CARGO_TARGET_DIR/html/index.html"
