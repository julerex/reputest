#!/bin/sh
set -eu

DATA_FILE="/data/0_0.tigerbeetle"
PORT="${PORT:-3000}"

if [ -z "${TB_CLUSTER_ID:-}" ]; then
    echo "TB_CLUSTER_ID is required" >&2
    exit 1
fi

ADDR=""
if [ -f /etc/hosts ]; then
    ADDR=$(awk '/[[:space:]]fly-local-6pn([[:space:]]|$)/ { print $1; exit }' /etc/hosts)
fi
if [ -z "${ADDR}" ]; then
    echo "fly-local-6pn was not found in /etc/hosts" >&2
    exit 1
fi

ADDRESSES="[${ADDR}]:${PORT}"

if [ ! -f "${DATA_FILE}" ]; then
    echo "Formatting replica at ${DATA_FILE} cluster=${TB_CLUSTER_ID}"
    tigerbeetle format --cluster="${TB_CLUSTER_ID}" --replica=0 --replica-count=1 "${DATA_FILE}"
fi

echo "Starting TigerBeetle at ${ADDRESSES} cache-grid=4GiB"
exec tigerbeetle start --addresses="${ADDRESSES}" --cache-grid=4GiB "${DATA_FILE}"
