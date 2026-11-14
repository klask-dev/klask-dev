#!/bin/sh
# Container entrypoint for the Klask backend.
#
# The image runs as the unprivileged user klask (UID/GID 1000) by default. In
# that case this script only prepares the directories and execs the command.
#
# When the container is started as root (for example `docker run --user root`
# on a host directory owned by another user), the script fixes the ownership of
# the data directories and then drops privileges to UID/GID 1000 with gosu.
set -eu

KLASK_UID=1000
KLASK_GID=1000

log() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') [entrypoint] $1"
}

ensure_dir() {
    dir="$1"
    if [ ! -d "$dir" ]; then
        log "Creating directory: $dir"
        mkdir -p "$dir"
    fi
}

if [ "$(id -u)" = "0" ]; then
    log "Running as root: fixing ownership of data directories"

    for dir in /app/index /app/repositories ${SEARCH_INDEX_DIR:+"$SEARCH_INDEX_DIR"}; do
        ensure_dir "$dir"
        chown -R "$KLASK_UID:$KLASK_GID" "$dir"
        chmod -R 755 "$dir"
    done

    log "Dropping privileges to $KLASK_UID:$KLASK_GID"
    exec gosu "$KLASK_UID:$KLASK_GID" "$@"
fi

# Non-root path: no ownership changes are possible, only create what is missing.
if [ -n "${SEARCH_INDEX_DIR:-}" ]; then
    ensure_dir "$SEARCH_INDEX_DIR"
fi

exec "$@"
