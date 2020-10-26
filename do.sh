#!/usr/bin/env bash

CALLER_PWD=$PWD
cd "$(dirname "$0")"

usage() {
    cat <<EOF
Usage: do.sh <action> <project> <action specific arguments>
Supported actions:
    build
    build-lib
    clean
    clippy
    doc
    dump
    fmt
    test
    update
Supported projects:
    all
    any directory containing a Cargo.toml file
EOF
}

sdkDir=bin/bpf-sdk
profile=bpfel-unknown-unknown/release

readCargoVariable() {
  declare variable="$1"
  declare Cargo_toml="$2"

  while read -r name equals value _; do
    if [[ $name = "$variable" && $equals = = ]]; then
        echo "${value//\"/}"
        return
    fi
  done < <(cat "$Cargo_toml")
  echo "Unable to locate $variable in $Cargo_toml" 1>&2
}

perform_action() {
    set -e
    # Use relative path if arg starts with "."
    if [[ $2 == .* ]]; then
        projectDir="$CALLER_PWD"/$2
    else
        projectDir="$PWD"/$2
    fi
    targetDir="$PWD"/target
    features=

    crateName="$(readCargoVariable name "$projectDir/Cargo.toml")"
    so_path="$targetDir/$profile"
    so_name="${crateName//\-/_}"
    so_name_unstripped="${so_name}_unstripped"

    if [[ -f "$projectDir"/Xargo.toml ]]; then
        features="--features=program"
    fi
    case "$1" in
    build)
        if [[ -f "$projectDir"/Xargo.toml ]]; then
            echo "build $crateName ($projectDir)"
            "$sdkDir"/rust/build.sh "$projectDir"
            cp "$so_path/${so_name}.so" "$so_path/${so_name_unstripped}.so"
            "$sdkDir"/dependencies/llvm-native/bin/llvm-objcopy --strip-all "$so_path/${so_name}.so" "$so_path/${so_name}.so"
        else
            echo "$projectDir does not contain a program, skipping"
        fi
        ;;
    build-lib)
        (
            cd "$projectDir"
            echo "build-lib $crateName ($projectDir)"
            export RUSTFLAGS="${@:3}"
            cargo build
        )
        ;;
    clean)
        "$sdkDir"/rust/clean.sh "$projectDir"
        ;;
    clippy)
        (
            cd "$projectDir"
            echo "clippy $crateName ($projectDir)"
            cargo +nightly clippy $features ${@:3}
        )
        ;;
    doc)
        (
            cd "$projectDir"
            echo "generating docs $crateName ($projectDir)"
            cargo doc ${@:3}
        )
        ;;
    dump)
        # Dump depends on tools that are not installed by default and must be installed manually
        # - readelf
        # - rustfilt
        if [[ -f "$projectDir"/Xargo.toml ]]; then
            if ! which rustfilt > /dev/null; then
                echo "Error: rustfilt not found.  It can be installed by running: cargo install rustfilt"
                exit 1
            fi
            if ! which readelf > /dev/null; then
                if [[ $(uname) = Darwin ]]; then
                    echo "Error: readelf not found.  It can be installed by running: brew install binutils"
                else
                    echo "Error: readelf not found."
                fi
                exit 1
            fi

            (
              cd "$CALLER_PWD"
              "$0" build "$2"
            )

            echo "dump $crateName ($projectDir)"

            so="$so_path/${so_name}.so"

            if [[ ! -r "$so" ]]; then
                echo "Error: No dump created, cannot read $so"
                exit 1
            fi
            dump="$so_path/${so_name}_dump"
            (
                set -x
                ls -la "$so" > "${dump}_mangled.txt"
                readelf -aW "$so" >>"${dump}_mangled.txt"
                "$sdkDir/dependencies/llvm-native/bin/llvm-objdump" \
                    -print-imm-hex \
                    --source \
                    --disassemble \
                    "$so" \
                    >> "${dump}_mangled.txt"
                sed s/://g <"${dump}_mangled.txt" | rustfilt >"${dump}.txt"
            )
            if [[ -f "$dump.txt" ]]; then
                echo "Created $dump.txt"
            else
                echo "Error: No dump created"
                exit 1
            fi
        else
            echo "$projectDir does not contain a program, skipping"
        fi
        ;;
    fmt)
        (
            cd "$projectDir"
            echo "formatting $projectDir"
            cargo fmt ${@:3}
        )
        ;;
    help)
        usage
        exit
        ;;
    test)
        (
            cd "$projectDir"
            echo "test $projectDir"
            cargo test $features ${@:3}
        )
        ;;
    update)
        ./bpf-sdk-install.sh
        ./do.sh clean all
        ;;
    *)
        echo "Error: Unknown command"
        usage
        exit
        ;;
    esac
}

set -e
if [[ $1 == "update" ]]; then
    perform_action "$1"
    exit
else
    if [[ "$#" -lt 2 ]]; then
        usage
        exit
    fi
    if [[ ! -d "$sdkDir" ]]; then
        ./do.sh update
    fi
fi

if [[ $2 == "all" ]]; then
    # Perform operation on all projects
    for project in */program*; do
        if [[ -f "$project"/Cargo.toml ]]; then
            perform_action "$1" "${project%/}" ${@:3}
        else
            continue
        fi
    done
else
    # Perform operation on requested project
    if [[ -d $2/program ]]; then
      perform_action "$1" "$2/program" "${@:3}"
    else
      perform_action "$1" "$2" "${@:3}"
    fi
fi

exit 0
