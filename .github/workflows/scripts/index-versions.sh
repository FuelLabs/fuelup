#!/bin/bash
set -e

latest_version() {
    _latest_version="$(curl -s https://api.github.com/repos/FuelLabs/"${1}"/releases/latest | grep "tag_name" | cut -d "\"" -f4 | cut -c 2-)"
    RETVAL="$_latest_version"
}

add_url_and_hash() {
    _url="https://github.com/FuelLabs/$1/releases/download/v$2/$3"
    _err=$(curl -sSf "${_url}s" -L -o "${3}" 2>&1)
    if echo "$_err" | grep -q 404; then
        printf "Could not download from %s - the release binary might not be ready yet. You can check if a binary is available here: https://github.com/FuelLabs/%s/releases/v%s\n" "${_url}" "${1}" "${2}"
        exit 1
    fi
    # shasum generates extra output so we take the first 64 bytes.
    _hash=$(shasum -a 256 "$3" | head -c 64)
    RETVAL="url = \"${_url}\"\nhash = \"${_hash}\"\n\n"
}

copy_same_pkg() {
    printf "%s: same latest version (%s); no changes needed\n" "${1}" "${2}"
    # find all headers along with the next 2 lines (url, hash) below lines matching [pkg.forc...], replace -- separator and pipe into new toml
    grep -A2 "^\[pkg.$1" channel-fuel-latest.tmp.toml | sed 's/--//g' >>channel-fuel-latest.toml
    printf '\n' >>channel-fuel-latest.toml
}

create_new_pkg() {
    _header=$(printf "[pkg.%s]\nversion = \"%s\"\n" "${1}" "${2}")
    RETVAL="$_header"
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
        printf "%s: Generating new package\n" "${1}"
        create_new_pkg "$1" "$2"
        _header="$RETVAL"
        _content=""
        for target in "${_targets[@]}"; do
            _content+="[pkg.${1}.target.${target}]\n"
            add_url_and_hash $_repo "$2" "$_tarball_prefix-${target}.tar.gz"
            _content+="$RETVAL"
        done

        # Only write to file if there's no problem downloading and hashing all the above releases.
        _package=$(printf "%s\n%s" "${_header}" "${_content}")
        echo -ne "$_package" >>channel-fuel-latest.toml
    fi
}

main() {
    latest_version sway
    FORC_LATEST_VERSION="$RETVAL"
    latest_version fuel-core
    FUEL_CORE_LATEST_VERSION="$RETVAL"

    FORC_CURRENT_VERSION="$(grep -s -A1 "\[pkg.forc\]" channel-fuel-latest.toml | grep "version" | cut -d "\"" -f 2- | rev | cut -c 2- | rev)"
    FUEL_CORE_CURRENT_VERSION="$(grep -s -A1 "\[pkg.fuel-core\]" channel-fuel-latest.toml | grep "version" | cut -d "\"" -f 2- | rev | cut -c 2- | rev)"

    if [ "${FORC_LATEST_VERSION}" = "${FORC_CURRENT_VERSION}" ] && [ "${FUEL_CORE_LATEST_VERSION}" = "${FUEL_CORE_CURRENT_VERSION}" ]; then
        printf "No new forc and fuel-core versions; exiting\n"
        exit 0
    fi

    mv channel-fuel-latest.toml channel-fuel-latest.tmp.toml
    # Cleanup tmp and downloaded tars/bin folders
    trap 'rm channel-fuel-latest.tmp.toml *.tar.gz' ERR EXIT

    create_pkg_in_channel forc "${FORC_LATEST_VERSION}" "${FORC_CURRENT_VERSION}"
    create_pkg_in_channel fuel-core "${FUEL_CORE_LATEST_VERSION}" "${FUEL_CORE_CURRENT_VERSION}"

    # remove newline at the end
    truncate -s -1 channel-fuel-latest.toml
    printf "Done.\n"
    exit 0
}

main || exit 1
