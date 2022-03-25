rm *.profraw

cargo test -- --test-threads=1

rm -rf coverage/

grcov . --binary-path ./target/debug/ -s . -t html \
--branch --ignore-not-existing --keep-only 'src/*' -o coverage/
