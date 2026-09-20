#!/usr/bin/env bash
# Builds the neb-fs-jvm native library, compiles the Java bindings, and packs
# both into build/neb-fs-jvm-<version>.jar. Publish it with:
#   mvn install:install-file -Dfile=build/neb-fs-jvm-<version>.jar \
#     -DgroupId=io.github.daylightnebula -DartifactId=neb-fs-jvm \
#     -Dversion=<version> -Dpackaging=jar
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"

VERSION=$(grep '^version' ../Cargo.toml | head -1 | cut -d'"' -f2)

cargo build --release --manifest-path ../Cargo.toml

rm -rf build
mkdir -p build/classes build/resources/native
cp ../target/release/libneb_fs_jvm.so build/resources/native/

javac --release 22 -d build/classes $(find src -name '*.java')
jar --create --file "build/neb-fs-jvm-${VERSION}.jar" -C build/classes . -C build/resources .

echo "Built build/neb-fs-jvm-${VERSION}.jar"
