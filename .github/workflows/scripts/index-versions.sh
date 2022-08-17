#!/bin/bash
set -e

add_url_and_hash() {
    _url="https://github.com/FuelLabs/$1/releases/download/v$2/$3"
    _err=$(curl -sSf "${_url}" -L -o "${3}" 2>&1)
    if echo "$_err" | grep -q 404; then
        printf "Could not download from %s - the release binary might not be ready yet. You can check if a binary is available here: https://github.com/FuelLabs/%s/releases/v%s\n" "${_url}" "${1}" "${2}"
        exit 1
    fi
    # shasum generates extra output so we take the first 64 bytes.
    _hash=$(shasum -a 256 "$3" | head -c 64)
    RETVAL="url = \"${_url}\"\nhash = \"${_hash}\"\n\n"
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

    # We need to recreate channel-fuel-latest.toml, generating new URLs and sha256 hashes for the download links.
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
}

main() {
    FORC_LATEST_VERSION=$1
    FUEL_CORE_LATEST_VERSION=$2
    GITHUB_RUN_ID=$3
    mv channel-fuel-latest.toml channel-fuel-latest.tmp.toml
    # Cleanup tmp and downloaded tars/bin folders
    trap 'rm channel-fuel-latest.tmp.toml *.tar.gz' ERR EXIT

    echo -e "run_url = \"https://github.com/FuelLabs/fuelup/actions/runs/${GITHUB_RUN_ID}\"\n" >>channel-fuel-latest.toml

    create_pkg_in_channel forc "${FORC_LATEST_VERSION}"
    create_pkg_in_channel fuel-core "${FUEL_CORE_LATEST_VERSION}"

    # remove newline at the end
    truncate -s -1 channel-fuel-latest.toml
    printf "Done.\n"
    exit 0
}

main "$@" || exit 1
