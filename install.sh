#!/usr/bin/env bash
set -euo pipefail

if [ "${EUID}" -eq 0 ]; then
    printf 'Ne pas lancer en root : ce script installe dans le HOME utilisateur.\n' >&2
    exit 1
fi

root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bin_dir="${HOME}/.local/bin"
unit_dir="${HOME}/.config/systemd/user"
app_dir="${HOME}/.local/share/applications"
icon_dir="${HOME}/.local/share/icons/hicolor/256x256/apps"

cargo build --release --locked --target-dir "${root}/target" --manifest-path "${root}/Cargo.toml"

install -Dm755 "${root}/target/release/noot-settings" "${bin_dir}/noot-settings"
install -Dm755 "${root}/target/release/nootd" "${bin_dir}/nootd"
mkdir -p "${unit_dir}"
sed "s|@EXEC@|${bin_dir}/nootd|" "${root}/packaging/nootd.service" > "${unit_dir}/nootd.service"
install -Dm644 "${root}/assets/noot-on.png" "${icon_dir}/noot.png"

mkdir -p "${app_dir}"
sed "s|@EXEC@|${bin_dir}/noot-settings|; s|@ICON@|noot|" "${root}/packaging/noot-settings.desktop.in" > "${app_dir}/noot-settings.desktop"

systemctl --user daemon-reload

printf 'Installé :\n  %s/noot-settings\n  %s/nootd.service\n  %s/noot-settings.desktop\n' "${bin_dir}" "${unit_dir}" "${app_dir}"
