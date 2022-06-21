#!/bin/bash
set -e

latest_version() {
    url=""
    _latest_version="$(curl -s https://api.github.com/repos/FuelLabs/$1/releases/latest | grep "tag_name" | cut -d "\"" -f4 | cut -c 2-)"
    RETVAL="$_latest_version"
}

add_url_and_hash() {
    URL="https://github.com/FuelLabs/$1/releases/download/v$2/$3.tar.gz"
    # For some reason shasum generates extra output so we take the first 64 bytes.
    HASH=$(curl -s ${URL} | shasum -a 256 | head -c 64)
    printf "url = \"${URL}\"\n" >>channel-fuel-latest.toml
    printf "hash = \"${HASH}\"\n\n" >>channel-fuel-latest.toml
}

copy_same_pkg() {
    # We simply copy the section of the old channel-fuel-latest.toml if no changes are needed.
    printf "$1: same latest version ($2); no changes needed\n"
    # find all headers along with the next 2 lines (url, hash) below lines matching [pkg.forc...], replace -- separator and pipe into new toml
    grep -A2 "^\[pkg.$1" channel-fuel-latest.tmp.toml | sed 's/--//g' >> channel-fuel-latest.toml
    printf '\n' >> channel-fuel-latest.toml
}

create_new_pkg() {
    printf "$1: Generating new packages\n"
    printf "[pkg.$1]\n" >>channel-fuel-latest.toml
    printf "version = $2\n" >>channel-fuel-latest.toml
}

main() {
    latest_version sway
    FORC_LATEST_VERSION="$RETVAL"
    latest_version fuel-core
    FUEL_CORE_LATEST_VERSION="$RETVAL"

    FORC_CURRENT_VERSION="$(grep -A1 -E "^\[pkg.forc\]$" channel-fuel-latest.toml | grep "version" | cut -d "=" -f2 | cut -c 2-)"
    FUEL_CORE_CURRENT_VERSION="$(grep -A1 -E "^\[pkg.fuel-core\]$" channel-fuel-latest.toml | grep "version" | cut -d "=" -f2 | cut -c 2-)"

    printf "${FORC_LATEST_VERSION} ${FORC_CURRENT_VERSION}\n"
    printf "${FUEL_CORE_LATEST_VERSION} ${FUEL_CORE_CURRENT_VERSION}\n"

    if [ "$FORC_LATEST_VERSION" = "$FORC_CURRENT_VERSION" ] && [ "$FUEL_CORE_LATEST_VERSION" = "$FUEL_CORE_CURRENT_VERSION" ]; then
	    printf "No new forc and fuel-core versions; exiting"
	    exit 0 
    fi

    mv channel-fuel-latest.toml channel-fuel-latest.tmp.toml
    trap 'rm channel-fuel-latest.tmp.toml' EXIT

    if [ "$FORC_LATEST_VERSION" = "$FORC_CURRENT_VERSION" ]; then
        # We simply copy the section of the old channel-fuel-latest.toml if no changes are needed.
        copy_same_pkg forc ${FORC_LATEST_VERSION}
    else
        # We need to recreate channel-fuel-latest.toml, generating new URLs and sha256 hashes for the download links.
        FORC_TARGETS=("darwin_amd64" "darwin_arm64" "linux_amd64" "linux_arm64")
        create_new_pkg forc ${FORC_LATEST_VERSION}
        for target in ${FORC_TARGETS[@]}; do
            printf "[pkg.forc.target.${target}]\n" >>channel-fuel-latest.toml
            add_url_and_hash sway ${FORC_LATEST_VERSION} forc-binaries-${target}
        done
    fi

    if [ "$FUEL_CORE_LATEST_VERSION" = "$FUEL_CORE_CURRENT_VERSION" ]; then
        copy_same_pkg fuel-core ${FUEL_CORE_LATEST_VERSION}
    else
        # We need to recreate channel-fuel-latest.toml, generating new URLs and sha256 hashes for the download links.
        FUEL_CORE_TARGETS=("aarch64-apple-darwin" "aarch64-unknown-linux-gnu" "x86_64-apple-darwin" "x86_64-unknown-linux-gnu")
        create_new_pkg fuel-core ${FUEL_CORE_LATEST_VERSION}
        for target in ${FUEL_CORE_TARGETS[@]}; do
            printf "[pkg.fuel-core.target.${target}]\n" >>channel-fuel-latest.toml
            add_url_and_hash fuel-core ${FUEL_CORE_LATEST_VERSION} fuel-core-${FUEL_CORE_LATEST_VERSION}-${target}
        done
    fi

}

main || exit 1
