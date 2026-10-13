#!/usr/bin/env bash
# Clone TigerBeetle 0.17.9 and build the in-tree Rust client assets when missing.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TB_DIR="${REPO_ROOT}/third_party/tigerbeetle"
TB_COMMIT="cc1c06a924e49b11089c521b2209d34c92caaf18"
TB_TAG="0.17.9"
CLIENT_DIR="${TB_DIR}/src/clients/rust"
ASSET_HEADER="${CLIENT_DIR}/assets/tb_client.h"

if [ -f "${ASSET_HEADER}" ] && [ -d "${CLIENT_DIR}/assets/lib" ]; then
    echo "TigerBeetle Rust client assets already present."
    exit 0
fi

mkdir -p "${REPO_ROOT}/third_party"

if [ ! -d "${TB_DIR}/.git" ]; then
    echo "Cloning TigerBeetle ${TB_TAG} (${TB_COMMIT})..."
    rm -rf "${TB_DIR}"
    # Full history is required: the Zig build runs `git tag --merged HEAD^`.
    git clone --branch "${TB_TAG}" https://github.com/tigerbeetle/tigerbeetle.git "${TB_DIR}"
fi

cd "${TB_DIR}"
HEAD_COMMIT="$(git rev-parse HEAD)"
if [ "${HEAD_COMMIT}" != "${TB_COMMIT}" ]; then
    echo "TigerBeetle checkout ${HEAD_COMMIT} does not match pin ${TB_COMMIT}" >&2
    exit 1
fi

if [ ! -x "${TB_DIR}/zig/zig" ]; then
    echo "Downloading Zig for the TigerBeetle client build..."
    ./zig/download.sh
fi

echo "Building TigerBeetle Rust client assets for ${TB_TAG}..."
./zig/zig build clients:rust -Drelease \
    -Dconfig-release="${TB_TAG}" \
    -Dconfig-release-client-min=0.16.4

if [ ! -f "${ASSET_HEADER}" ] || [ ! -d "${CLIENT_DIR}/assets/lib" ]; then
    echo "TigerBeetle client assets were not produced at ${CLIENT_DIR}/assets" >&2
    exit 1
fi

echo "TigerBeetle Rust client assets ready."
