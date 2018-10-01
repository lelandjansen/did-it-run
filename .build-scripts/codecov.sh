#!/usr/bin/env bash
# source: https://github.com/codecov/example-rust
set -xe
wget https://github.com/SimonKagstrom/kcov/archive/master.tar.gz
tar -xzf master.tar.gz
rm master.tar.gz
cd kcov-master
mkdir build
cd build
cmake ..
make
make install DESTDIR=../../kcov-build
cd ../..
rm -rf kcov-master
files=$(find target/debug \
  -maxdepth 1 \
  -type f \
  -regextype grep \
  -regex ".*-[0-9a-f]\{16\}" \
  -executable);
for file in $files; do
  mkdir -p "target/cov/$(basename $file)"
  ./kcov-build/usr/local/bin/kcov \
    --exclude-pattern=/.cargo,/usr/lib \
    --verify "target/cov/$(basename $file)" "$file"
done
bash <(curl -s https://codecov.io/bash)
echo "Uploaded code coverage"
