#!/usr/bin/env bash
# Format (if needed) and start a local single-replica TigerBeetle cluster on 127.0.0.1:3000.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TB_VERSION="0.17.9"
DATA_DIR="${REPO_ROOT}/tmp/tigerbeetle"
DATA_FILE="${DATA_DIR}/0_0.tigerbeetle"
BIN="${DATA_DIR}/tigerbeetle"
CLUSTER_ID="${TB_CLUSTER_ID:-1}"
ADDRESS="${TB_ADDRESS:-127.0.0.1:3000}"

mkdir -p "${DATA_DIR}"

if [ ! -x "${BIN}" ]; then
    echo "Downloading TigerBeetle ${TB_VERSION}..."
    ARCHIVE="${DATA_DIR}/tigerbeetle.zip"
    ARCH="$(uname -m)"
    case "${ARCH}" in
        x86_64 | amd64) ASSET="tigerbeetle-x86_64-linux.zip" ;;
        aarch64 | arm64) ASSET="tigerbeetle-aarch64-linux.zip" ;;
        *)
            echo "Unsupported architecture: ${ARCH}" >&2
            exit 1
            ;;
    esac
    curl -fsSL -o "${ARCHIVE}" \
        "https://github.com/tigerbeetle/tigerbeetle/releases/download/${TB_VERSION}/${ASSET}"
    unzip -o "${ARCHIVE}" -d "${DATA_DIR}"
    chmod +x "${BIN}"
    rm -f "${ARCHIVE}"
fi

if [ ! -f "${DATA_FILE}" ]; then
    echo "Formatting local replica cluster_id=${CLUSTER_ID}"
    "${BIN}" format --cluster="${CLUSTER_ID}" --replica=0 --replica-count=1 "${DATA_FILE}"
fi

echo "Starting TigerBeetle at ${ADDRESS} (cluster ${CLUSTER_ID})"
echo "Export TB_CLUSTER_ID=${CLUSTER_ID} and TB_ADDRESS=${ADDRESS} before cargo run."
exec "${BIN}" start --addresses="${ADDRESS}" --development "${DATA_FILE}"
