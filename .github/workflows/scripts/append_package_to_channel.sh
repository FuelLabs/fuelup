#!/bin/bash
set -e

add_url_and_hash() {
    _url="https://github.com/FuelLabs/$1/releases/download/$2/$3"
    _err=$(curl -sSf "${_url}" -L -o "${3}" 2>&1)
    if echo "$_err" | grep -q 404; then
        printf "Could not download from %s - the release binary might not be ready yet. You can check if a binary is available here: https://github.com/FuelLabs/%s/releases/%s\n" "${_url}" "${1}" "${2}"
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
    CHANNEL_TOML_NAME=$3
    version=$2
    date="$(date +'%Y-%m-%d')"
    tag="v${2}"
    case "${1}" in
        "forc")
            _targets=("darwin_amd64" "darwin_arm64" "linux_amd64" "linux_arm64")
            _repo="sway"
            _tarball_prefix="forc-binaries"
            ;;
        "fuel-core")
            _targets=("aarch64-apple-darwin" "aarch64-unknown-linux-gnu" "x86_64-apple-darwin" "x86_64-unknown-linux-gnu")
            _repo="fuel-core"
            _tarball_prefix="fuel-core"

            if [ "${2}" != "nightly" ]; then
                _tarball_prefix+="-${version}"
            fi

            ;;
    esac

    if [ "${2}" = "nightly" ]; then
        _repo="sway-nightly-binaries"
        version="$(curl -s https://api.github.com/repos/FuelLabs/${_repo}/releases | grep "tag_name" | grep "nightly.${date}" | grep "${_tarball_prefix}" | head -n 1 | cut -d "-" -f3- | cut -d "\"" -f1)"
        _tarball_prefix+="-${version}"
        # Replace '+' within string with '%2B' to be URL friendly
        tag=$(echo "${_tarball_prefix}" | sed -r "s/\+/\%2B/g")
    fi

    # We need to recreate channel-fuel-latest.toml, generating new URLs and sha256 hashes for the download links.
    printf "%s: Generating new package\n" "${1}"
    create_new_pkg "$1" "${version}"
    _header="$RETVAL"
    _content=""
    for target in "${_targets[@]}"; do
        _content+="[pkg.${1}.target.${target}]\n"
        # TAG is either: v0.22.1 or forc-binaries-nightly-2022-08-25
        add_url_and_hash $_repo "${tag}" "${_tarball_prefix}-${target}.tar.gz"
        _content+="$RETVAL"
    done

    # Only write to file if there's no problem downloading and hashing all the above releases.
    _package=$(printf "%s\n%s" "${_header}" "${_content}")
    echo -ne "$_package" >>"$CHANNEL_TOML_NAME"
}

main() {
    COMPONENT=$1
    VERSION=$2
    CHANNEL_TOML_NAME=$3
    trap 'rm *.tar.gz' ERR EXIT

    create_pkg_in_channel "${COMPONENT}" "${VERSION}" "${CHANNEL_TOML_NAME}"

    exit 0
}

main "$@" || exit 1
