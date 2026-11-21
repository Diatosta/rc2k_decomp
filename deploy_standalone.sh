#!/usr/bin/env bash
# deploy_standalone.sh: the older name for `./build.sh <target> --deploy`, kept so that the
# commands and the deploy.local lines written for it go on working. New use: build.sh.
#
#     ./deploy_standalone.sh                  = ./build.sh windows --deploy
#     ./deploy_standalone.sh x64              = ./build.sh windows-x64 --deploy
#     ./deploy_standalone.sh linux            = ./build.sh linux --deploy
#     ./deploy_standalone.sh mac [x64|a64]    = ./build.sh mac --deploy [--arch x64|a64]
#     ./deploy_standalone.sh anbernic [pgo]   = ./build.sh handheld --deploy [--pgo]
#
# Where deploy.local has no DEPLOY_ line for the target, the older lines place the Windows
# and Linux programs as before: STANDALONE_DIR (or the folder above GAME_DIR), 64 bits in its
# x64 folder, Linux in its linux folder.
set -euo pipefail

cd "$(dirname "$0")"
# shellcheck disable=SC1091
[ -f deploy.local ] && source deploy.local

old_folder() {
    : "${GAME_DIR:?Set DEPLOY_WINDOWS / DEPLOY_WINDOWS_X64 / DEPLOY_LINUX (or the older GAME_DIR) in deploy.local}"
    printf '%s\n' "${STANDALONE_DIR:-$GAME_DIR/..}"
}

case "${1:-}" in
    "")
        if [ -z "${DEPLOY_WINDOWS:-}" ]; then
            OLD="$(old_folder)"
            export DEPLOY_WINDOWS="$OLD"
        fi
        exec ./build.sh windows --deploy ;;
    x64)
        if [ -z "${DEPLOY_WINDOWS_X64:-}" ]; then
            OLD="$(old_folder)"
            export DEPLOY_WINDOWS_X64="$OLD/x64"
        fi
        exec ./build.sh windows-x64 --deploy ;;
    linux)
        if [ -z "${DEPLOY_LINUX:-}" ]; then
            OLD="$(old_folder)"
            export DEPLOY_LINUX="$OLD/linux"
        fi
        exec ./build.sh linux --deploy ;;
    mac)
        case "${2:-}" in
            "") exec ./build.sh mac --deploy ;;
            x64|a64) exec ./build.sh mac --deploy --arch "$2" ;;
            *) echo "usage: deploy_standalone.sh mac [x64|a64]" >&2; exit 1 ;;
        esac ;;
    anbernic)
        if [ "${2:-}" = pgo ]; then exec ./build.sh handheld --deploy --pgo; fi
        exec ./build.sh handheld --deploy ;;
    *)
        echo "usage: deploy_standalone.sh [x64|linux|mac [x64|a64]|anbernic [pgo]]  (see ./build.sh)" >&2
        exit 1 ;;
esac
