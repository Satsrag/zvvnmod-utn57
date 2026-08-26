#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
install_path=${ZVVNMOD_MONGOL_NORM_PATH:-"$repo_root/.mongol-norm-site"}
python_command=${ZVVNMOD_MONGOL_NORM_PYTHON:-${PYTHON:-python3}}
requirements="$repo_root/requirements-mongol-norm.txt"

if [ -L "$install_path" ]; then
    printf '%s\n' "Refusing symlink install destination: $install_path" >&2
    exit 1
fi

install_parent=$(dirname -- "$install_path")
mkdir -p "$install_parent"
stage=$(mktemp -d "$install_parent/.mongol-norm-stage.XXXXXX")
backup=
cleanup() {
    if [ -n "$stage" ]; then
        rm -rf -- "$stage"
    fi
    if [ -n "$backup" ] && [ -d "$backup" ] && [ ! -e "$install_path" ] && [ ! -L "$install_path" ]; then
        mv -- "$backup" "$install_path"
    fi
}
trap cleanup EXIT
trap 'exit 1' HUP INT TERM

"$python_command" -I -m pip install \
    --disable-pip-version-check \
    --require-hashes \
    --no-deps \
    --upgrade \
    --target "$stage" \
    -r "$requirements"

validation_output=$(printf '%s\n' '{"protocol":1,"records":[{"unit":"O","position":"init"}]}' | \
    "$python_command" -I -S "$repo_root/scripts/mongol_norm_positioned.py" "$stage")
expected_output=$(printf '\341\240\244\341\240\213\342\200\215')
if [ "$validation_output" != "$expected_output" ]; then
    printf '%s\n' "mongol-norm staged validation returned unexpected output" >&2
    exit 1
fi

if [ -L "$install_path" ]; then
    printf '%s\n' "Refusing symlink install destination: $install_path" >&2
    exit 1
fi
if [ -e "$install_path" ]; then
    if [ ! -d "$install_path" ]; then
        printf '%s\n' "Refusing non-directory install destination: $install_path" >&2
        exit 1
    fi
    backup=$(mktemp -d "$install_parent/.mongol-norm-backup.XXXXXX")
    rmdir "$backup"
    mv -- "$install_path" "$backup"
fi
mv -- "$stage" "$install_path"
stage=
if [ -n "$backup" ]; then
    rm -rf -- "$backup"
    backup=
fi
trap - EXIT HUP INT TERM

printf '%s\n' "mongol-norm 0.0.4 installed and verified."
printf '%s\n' "Install path: $install_path"
printf '%s\n' "The zvvnmod-to-unicode command discovers the default repository-local path automatically."
printf '%s\n' "For other installations, configure ZVVNMOD_MONGOL_NORM_PATH to this install path."
