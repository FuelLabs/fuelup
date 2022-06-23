#!/bin/bash
set -e

latest_version() {
    _latest_version="$(curl -s https://api.github.com/repos/FuelLabs/"${1}"/releases/latest | grep "tag_name" | cut -d "\"" -f4 | cut -c 2-)"
    RETVAL="$_latest_version"
}

add_url_and_hash() {
    _url="https://github.com/FuelLabs/$1/releases/download/v$2/$3"
    # shasum generates extra output so we take the first 64 bytes.
    curl -sSf "${_url}" -L -o "${3}"
    _hash=$(shasum -a 256 "$3" | head -c 64)
    printf "url = \"%s\"\n" "${_url}" >>channel-fuel-latest.toml
    printf "hash = \"%s\"\n\n" "${_hash}" >>channel-fuel-latest.toml
}

copy_same_pkg() {
    printf "%s: same latest version (%s); no changes needed\n" "${1}" "${2}"
    # find all headers along with the next 2 lines (url, hash) below lines matching [pkg.forc...], replace -- separator and pipe into new toml
    grep -A2 "^\[pkg.$1" channel-fuel-latest.tmp.toml | sed 's/--//g' >>channel-fuel-latest.toml
    printf '\n' >>channel-fuel-latest.toml
}

create_new_pkg() {
    printf "%s: Generating new packages\n" "${1}"
    printf "[pkg.%s]\n" "${1}" >>channel-fuel-latest.toml
    printf "version = \"%s\"\n" "${2}" >>channel-fuel-latest.toml
}

create_pkg_in_channel() {
    case "${1}" in
        "forc")
            _targets=("darwin_amd64" "darwin_arm64" "linux_amd64" "linux_arm64")
            _repo="sway"
            _tarball_prefix="forc-binaries"
            ;;
        "fuel-core")
            _targets=("aarch64-apple-darwin" "aarch64-unknown-linux-gnu" "x86_64-apple-darwin" "x86_64-unknown-linux-gnu")
            _repo="fuel-core"
            _tarball_prefix="fuel-core-${2}"
            ;;
    esac

    if [ "$2" = "$3" ]; then
        # We simply copy the section of the old channel-fuel-latest.toml if no changes are needed.
        copy_same_pkg "$1" "$2"
    else
        # We need to recreate channel-fuel-latest.toml, generating new URLs and sha256 hashes for the download links.
        printf "%s: new version available: %s -> %s\n" "${1}" "${3}" "${2}"
        create_new_pkg "$1" "$2"
        for target in "${_targets[@]}"; do
            printf "[pkg.%s.target.%s]\n" "$1" "${target}" >>channel-fuel-latest.toml
            add_url_and_hash $_repo "$2" "$_tarball_prefix-${target}.tar.gz"
        done
    fi
}

main() {
    latest_version sway
    FORC_LATEST_VERSION="$RETVAL"
    latest_version fuel-core
    FUEL_CORE_LATEST_VERSION="$RETVAL"

    FORC_CURRENT_VERSION="$(grep -A1 "\[pkg.forc\]" channel-fuel-latest.toml | grep "version" | cut -d "\"" -f 2- | rev | cut -c 2- | rev)"
    FUEL_CORE_CURRENT_VERSION="$(grep -A1 "\[pkg.fuel-core\]" channel-fuel-latest.toml | grep "version" | cut -d "\"" -f 2- | rev | cut -c 2- | rev)"

    if [ "${FORC_LATEST_VERSION}" = "${FORC_CURRENT_VERSION}" ] && [ "${FUEL_CORE_LATEST_VERSION}" = "${FUEL_CORE_CURRENT_VERSION}" ]; then
        printf "No new forc and fuel-core versions; exiting\n"
        exit 0
    fi

    mv channel-fuel-latest.toml channel-fuel-latest.tmp.toml
    # Cleanup tmp and downloaded tars/bin folders
    trap 'rm channel-fuel-latest.tmp.toml *.tar.gz' ERR EXIT

    create_pkg_in_channel forc "${FORC_LATEST_VERSION}" "${FORC_CURRENT_VERSION}"
    create_pkg_in_channel fuel-core "${FUEL_CORE_LATEST_VERSION}" "${FUEL_CORE_CURRENT_VERSION}"

    printf "Done.\n"
    exit 0
}

main || exit 1
