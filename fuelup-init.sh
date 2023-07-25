#!/bin/sh
set -e

FUELUP_DIR=${FUELUP_DIR-"$HOME/.fuelup"}

main() {
    need_cmd git
    need_cmd curl
    need_cmd chmod
    need_cmd mkdir
    need_cmd rm
    need_cmd rmdir

    check_cargo_bin forc
    check_cargo_bin forc-fmt
    check_cargo_bin forc-explore
    check_cargo_bin forc-lsp
    check_cargo_bin fuel-core

    if check_cmd nix; then
        # check if conf.nix/config.nix/configuration.nix exists, if not ask user permission to create one
        # if it exists or user grants permission to create one, check if nix-command and flakes features enabled
        # and fuel.nix cachix is linked, otherwise write to file
        echo "found nix"
        true
    else
        run_fuel_nix_install_script
    fi

    get_architecture || return 1
    local _arch="$RETVAL"
    assert_nz "$_arch" "arch"

    mkdir -p "$FUELUP_DIR/bin"

    local _fuelup_version
    _published_fuelup_version_url="https://raw.githubusercontent.com/FuelLabs/fuelup/gh-pages/fuelup-version"
    _fuelup_version="$(curl -s $_published_fuelup_version_url)"
    if echo "$_fuelup_version" | grep -q -E '404|400'; then
        warn "fuelup-version was not found on fuelup gh-pages (https://github.com/FuelLabs/fuelup/tree/gh-pages); falling back to using GitHub API."
        _fuelup_version="$(curl -s https://api.github.com/repos/FuelLabs/fuelup/releases/latest | grep "tag_name" | cut -d "\"" -f4 | cut -c 2-)"
    fi

    local _fuelup_url="https://github.com/FuelLabs/fuelup/releases/download/v${_fuelup_version}/fuelup-${_fuelup_version}-${_arch}.tar.gz"

    local _dir
    _dir="$(ensure mktemp -d)"
    local _file="${_dir}/fuelup.tar.gz"

    local _ansi_escapes_are_valid=false
    if [ -t 2 ]; then
        if [ "${TERM+set}" = 'set' ]; then
            case "$TERM" in
                xterm* | rxvt* | urxvt* | linux* | vt*)
                    _ansi_escapes_are_valid=true
                    ;;
            esac
        fi
    fi

    # always prompt PATH modification, unless --no-modify-path provided
    local prompt_modify=yes
    # always install latest toolchain (for convenience), unless --skip-toolchain-installation provided
    local skip_toolchain_installation=no

    for arg in "$@"; do
        case "$arg" in
            --no-modify-path)
                prompt_modify=no
                ;;
            --skip-toolchain-installation)
                skip_toolchain_installation=yes
                ;;
            *)
                OPTIND=1
                if [ "${arg%%--*}" = "" ]; then
                    # Long option (other than --help);
                    # don't attempt to interpret it.
                    continue
                fi
                ;;
        esac
    done

    if [ "$prompt_modify" = "yes" ]; then
        case $SHELL in
            */bash)
                SHELL_PROFILE=$HOME/.bashrc
                ;;
            */zsh)
                SHELL_PROFILE=$HOME/.zshrc
                ;;
            */fish)
                SHELL_PROFILE=$HOME/.config/fish/config.fish
                ;;
            *)
                warn "Failed to detect shell; please add ${FUELUP_DIR}/bin to your PATH manually."
                ;;
        esac

        if [ -n "$SHELL_PROFILE" ]; then
            preinstall_confirmation
            read -r answer </dev/tty
            allow_modify=$(echo "$answer" | cut -c1-1)
            case $allow_modify in
                "y" | "Y")
                    allow_modify=yes
                    printf "\nfuelup will modify your PATH variable for you.\n\n"
                    ;;
                *)
                    allow_modify=no
                    printf "\nfuelup will not modify your PATH variable for you.\n\n"
                    ;;
            esac
        else
            allow_modify=no
        fi
    fi

    if $_ansi_escapes_are_valid; then
        printf "\33[1minfo:\33[0m downloading fuelup %s\n" "$_fuelup_version" 1>&2
    else
        printf 'info: downloading fuelup %s\n' "$_fuelup_version" 1>&2
    fi

    ensure downloader "$_fuelup_url" "$_file" "$_arch"

    ignore tar -xf "$_file" -C "$_dir"

    ensure mv "$_dir/fuelup-${_fuelup_version}-${_arch}/fuelup" "$FUELUP_DIR/bin/fuelup"
    ensure chmod u+x "$FUELUP_DIR/bin/fuelup"

    if [ ! -x "$FUELUP_DIR/bin/fuelup" ]; then
        printf '%s\n' "Cannot execute $FUELUP_DIR/bin/fuelup." 1>&2
        printf '%s\n' "Please copy the file to a location where you can execute binaries and run ./fuelup." 1>&2
        exit 1
    fi

    if [ "$skip_toolchain_installation" = "no" ]; then
        ignore "$FUELUP_DIR/bin/fuelup" "toolchain" "install" "latest"
    fi

    local _retval=$?

    ignore rm "$_file"
    ignore rmdir "$_dir/fuelup-${_fuelup_version}-${_arch}"
    ignore rmdir "$_dir"

    printf '\n'
    printf '%s\n' "fuelup ${_fuelup_version} has been installed in $FUELUP_DIR/bin. To fetch the latest toolchain containing the forc and fuel-core binaries, run 'fuelup toolchain install latest'. To generate completions for your shell, run 'fuelup completions --shell=SHELL'." 1>&2

    if [ "$allow_modify" = "yes" ]; then
        if echo "$PATH" | grep -q "$FUELUP_DIR/bin"; then
            printf "\n%s/bin already exists in your PATH.\n" "$FUELUP_DIR"
        else
            echo "export PATH=\"\$HOME/.fuelup/bin:\$PATH"\" >>"$SHELL_PROFILE"
            printf "\n%s added to PATH. Run 'source %s' or start a new terminal session to use fuelup.\n" "$FUELUP_DIR" "$SHELL_PROFILE"
        fi
    else
        add_path_message
    fi

    return "$_retval"
}

preinstall_confirmation() {
    cat 1>&2 <<EOF

fuelup uses "$FUELUP_DIR" as its home directory to manage the Fuel toolchain, however as of *tbd release* binaries will be managed at /nix/store.

To use the toolchain, you will have to configure your PATH, which tells your machine where to locate fuelup and the Fuel toolchain.

If permitted, fuelup-init will configure your PATH for you by running the following:

    echo "export PATH="\$HOME/.fuelup/bin:\$PATH"" >> $SHELL_PROFILE

Would you like fuelup-init to modify your PATH variable for you? (N/y)
EOF
}

add_path_message() {
    cat 1>&2 <<EOF

You might have to add $FUELUP_DIR/bin to path:

bash/zsh:

export PATH="\${HOME}/.fuelup/bin:\${PATH}"

fish:

fish_add_path ~/.fuelup/bin
EOF
}

get_architecture() {
    local _ostype _cputype
    _ostype="$(uname -s)"
    _cputype="$(uname -m)"

    case "$_ostype" in
        Linux)
            _ostype="unknown-linux-gnu"
            ;;
        Darwin)
            _ostype="apple-darwin"
            ;;
        *)
            err "unsupported os type: $_ostype"
            ;;
    esac

    case "$_cputype" in
        x86_64 | x86-64 | x64 | amd64)
            _cputype="x86_64"
            ;;
        aarch64 | arm64)
            _cputype="aarch64"
            ;;
        *)
            err "unsupported cpu type: $_cputype"
            ;;
    esac

    _arch="${_cputype}-${_ostype}"

    RETVAL="$_arch"
}

check_cargo_bin() {
    if which "${1}" 2>/dev/null | grep -q "[.cargo]"; then
        warn "$1 is already installed via cargo and is in use by your system. You should update your PATH, or execute 'cargo uninstall $1'"
    fi
}

assert_nz() {
    if [ -z "$1" ]; then err "assert_nz $2"; fi
}

say() {
    printf 'fuelup: %s\n' "$1"
}

need_cmd() {
    if ! check_cmd "$1"; then
        err "need '$1' (command not found)"
    fi
}

check_cmd() {
    command -v "$1" >/dev/null 2>&1
}

# Run a command that should never fail. If the command fails execution
# will immediately terminate with an error showing the failing
# command.
ensure() {
    if ! "$@"; then err "command failed: $*"; fi
}

downloader() {
    local _dld
    local _ciphersuites
    local _err
    local _status
    local _retry

    if check_cmd curl; then
        _dld=curl
    elif check_cmd wget; then
        _dld=wget
    else
        _dld='curl or wget' # to be used in error message of need_cmd
    fi

    if [ "$1" = --check ]; then
        need_cmd "$_dld"
    elif [ "$_dld" = curl ]; then
        check_curl_for_retry_support
        _retry="$RETVAL"
        get_ciphersuites_for_curl
        _ciphersuites="$RETVAL"
        if [ -n "$_ciphersuites" ]; then
            # shellcheck disable=SC2086
            _err=$(curl $_retry --proto '=https' --tlsv1.2 --ciphers "$_ciphersuites" --silent --show-error --fail --location "$1" --output "$2" 2>&1)
            _status=$?
        else
            echo "Warning: Not enforcing strong cipher suites for TLS, this is potentially less secure"
            if ! check_help_for "$3" curl --proto --tlsv1.2; then
                echo "Warning: Not enforcing TLS v1.2, this is potentially less secure"
                # shellcheck disable=SC2086
                _err=$(curl $_retry --silent --show-error --fail --location "$1" --output "$2" 2>&1)
                _status=$?
            else
                # shellcheck disable=SC2086
                _err=$(curl $_retry --proto '=https' --tlsv1.2 --silent --show-error --fail --location "$1" --output "$2" 2>&1)
                _status=$?
            fi
        fi
        if [ -n "$_err" ]; then
            echo "$_err" >&2
            if echo "$_err" | grep -q 404; then
                err "fuelup ${_fuelup_version} was not found - either the release is not ready yet or the tag is invalid. You can check if the release is available here: https://github.com/FuelLabs/fuelup/releases/v${_fuelup_version}"
            fi
        fi

        return $_status
    elif [ "$_dld" = wget ]; then
        if [ "$(wget -V 2>&1 | head -2 | tail -1 | cut -f1 -d" ")" = "BusyBox" ]; then
            echo "Warning: using the BusyBox version of wget.  Not enforcing strong cipher suites for TLS or TLS v1.2, this is potentially less secure"
            _err=$(wget "$1" -O "$2" 2>&1)
            _status=$?
        else
            get_ciphersuites_for_wget
            _ciphersuites="$RETVAL"
            if [ -n "$_ciphersuites" ]; then
                _err=$(wget --https-only --secure-protocol=TLSv1_2 --ciphers "$_ciphersuites" "$1" -O "$2" 2>&1)
                _status=$?
            else
                echo "Warning: Not enforcing strong cipher suites for TLS, this is potentially less secure"
                if ! check_help_for "$3" wget --https-only --secure-protocol; then
                    echo "Warning: Not enforcing TLS v1.2, this is potentially less secure"
                    _err=$(wget "$1" -O "$2" 2>&1)
                    _status=$?
                else
                    _err=$(wget --https-only --secure-protocol=TLSv1_2 "$1" -O "$2" 2>&1)
                    _status=$?
                fi
            fi
        fi
        if [ -n "$_err" ]; then
            echo "$_err" >&2
            if echo "$_err" | grep -q ' 404 Not Found$'; then
                err "installer for platform '$3' not found, this may be unsupported"
            fi
        fi
        return $_status
    else
        err "Unknown downloader" # should not reach here
    fi
}

run_fuel_nix_install_script() {
    echo "running fuel.nix install script..."
    curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix/tag/v0.9.0 | sh -s -- install --no-confirm --extra-conf "extra-substituters = https://fuellabs.cachix.org" --extra-conf "extra-trusted-public-keys = fuellabs.cachix.org-1:3gOmll82VDbT7EggylzOVJ6dr0jgPVU/KMN6+Kf8qx8="
}

# Check if curl supports the --retry flag, then pass it to the curl invocation.
check_curl_for_retry_support() {
    local _retry_supported=""
    # "unspecified" is for arch, allows for possibility old OS using macports, homebrew, etc.
    if check_help_for "notspecified" "curl" "--retry"; then
        _retry_supported="--retry 3"
    fi

    RETVAL="$_retry_supported"

}

check_help_for() {
    local _arch
    local _cmd
    local _arg
    _arch="$1"
    shift
    _cmd="$1"
    shift

    local _category
    if "$_cmd" --help | grep -q 'For all options use the manual or "--help all".'; then
        _category="all"
    else
        _category=""
    fi

    case "$_arch" in

        *darwin*)
            if check_cmd sw_vers; then
                case $(sw_vers -productVersion) in
                    10.*)
                        # If we're running on macOS, older than 10.13, then we always
                        # fail to find these options to force fallback
                        if [ "$(sw_vers -productVersion | cut -d. -f2)" -lt 13 ]; then
                            # Older than 10.13
                            echo "Warning: Detected macOS platform older than 10.13"
                            return 1
                        fi
                        ;;
                    11.*)
                        # We assume Big Sur will be OK for now
                        ;;
                    *)
                        # Unknown product version, warn and continue
                        echo "Warning: Detected unknown macOS major version: $(sw_vers -productVersion)"
                        echo "Warning TLS capabilities detection may fail"
                        ;;
                esac
            fi
            ;;

    esac

    for _arg in "$@"; do
        if ! "$_cmd" --help "$_category" | grep -q -- "$_arg"; then
            return 1
        fi
    done

    true # not strictly needed

}

# Return cipher suite string specified by user, otherwise return strong TLS 1.2-1.3 cipher suites
# if support by local tools is detected. Detection currently supports these curl backends:
# GnuTLS and OpenSSL (possibly also LibreSSL and BoringSSL). Return value can be empty.
get_ciphersuites_for_curl() {
    if [ -n "${RUSTUP_TLS_CIPHERSUITES-}" ]; then
        # user specified custom cipher suites, assume they know what they're doing
        RETVAL="$RUSTUP_TLS_CIPHERSUITES"
        return
    fi

    local _openssl_syntax="no"
    local _gnutls_syntax="no"
    local _backend_supported="yes"
    if curl -V | grep -q ' OpenSSL/'; then
        _openssl_syntax="yes"
    elif curl -V | grep -iq ' LibreSSL/'; then
        _openssl_syntax="yes"
    elif curl -V | grep -iq ' BoringSSL/'; then
        _openssl_syntax="yes"
    elif curl -V | grep -iq ' GnuTLS/'; then
        _gnutls_syntax="yes"
    else
        _backend_supported="no"
    fi

    local _args_supported="no"
    if [ "$_backend_supported" = "yes" ]; then
        # "unspecified" is for arch, allows for possibility old OS using macports, homebrew, etc.
        if check_help_for "notspecified" "curl" "--tlsv1.2" "--ciphers" "--proto"; then
            _args_supported="yes"
        fi
    fi

    local _cs=""
    if [ "$_args_supported" = "yes" ]; then
        if [ "$_openssl_syntax" = "yes" ]; then
            _cs=$(get_strong_ciphersuites_for "openssl")
        elif [ "$_gnutls_syntax" = "yes" ]; then
            _cs=$(get_strong_ciphersuites_for "gnutls")
        fi
    fi

    RETVAL="$_cs"
}

# Return strong TLS 1.2-1.3 cipher suites in OpenSSL or GnuTLS syntax. TLS 1.2
# excludes non-ECDHE and non-AEAD cipher suites. DHE is excluded due to bad
# DH params often found on servers (see RFC 7919). Sequence matches or is
# similar to Firefox 68 ESR with weak cipher suites disabled via about:config.
# $1 must be openssl or gnutls.
get_strong_ciphersuites_for() {
    if [ "$1" = "openssl" ]; then
        # OpenSSL is forgiving of unknown values, no problems with TLS 1.3 values on versions that don't support it yet.
        echo "TLS_AES_128_GCM_SHA256:TLS_CHACHA20_POLY1305_SHA256:TLS_AES_256_GCM_SHA384:ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-CHACHA20-POLY1305:ECDHE-RSA-CHACHA20-POLY1305:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384"
    elif [ "$1" = "gnutls" ]; then
        # GnuTLS isn't forgiving of unknown values, so this may require a GnuTLS version that supports TLS 1.3 even if wget doesn't.
        # Begin with SECURE128 (and higher) then remove/add to build cipher suites. Produces same 9 cipher suites as OpenSSL but in slightly different order.
        echo "SECURE128:-VERS-SSL3.0:-VERS-TLS1.0:-VERS-TLS1.1:-VERS-DTLS-ALL:-CIPHER-ALL:-MAC-ALL:-KX-ALL:+AEAD:+ECDHE-ECDSA:+ECDHE-RSA:+AES-128-GCM:+CHACHA20-POLY1305:+AES-256-GCM"
    fi
}

err() {
    say "$1" >&2
    exit 1
}

warn() {
    say "warning: ${1}" >&2
}

# This is just for indicating that commands' results are being
# intentionally ignored. Usually, because it's being executed
# as part of error handling.
ignore() {
    "$@"
}
main "$@" || exit 1
