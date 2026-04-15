#!/usr/bin/env bash
set -euo pipefail

# Ensure system binaries are available (PATH may be stripped during HM activation)
export PATH="/usr/bin:/usr/local/bin:$PATH"

OMP_VERSION="${OMP_VERSION:-v13.12.8}"
OMP_BUN_VERSION="${OMP_BUN_VERSION:-bun-v1.3.10}"
BIN_DIR="${HOME}/.local/bin"
RUNTIME_DIR="${HOME}/.local/share/omp-runtime"
FALLBACK_DIR="${RUNTIME_DIR}/${OMP_VERSION}"
FALLBACK_BIN="${FALLBACK_DIR}/omp-darwin-arm64"
WRAPPER_PATH="${BIN_DIR}/omp"
NATIVE_PATH="${BIN_DIR}/pi_natives.darwin-arm64.node"
BUN_DIR="${RUNTIME_DIR}/${OMP_BUN_VERSION}"
BUN_BIN="${BUN_DIR}/bun"
EXT_DIR="${HOME}/.omp/agent/extensions"
SOURCE_CLI="${EXT_DIR}/node_modules/@oh-my-pi/pi-coding-agent/src/cli.ts"
OMP_BASE_URL="https://github.com/can1357/oh-my-pi/releases/download/${OMP_VERSION}"
BUN_ZIP_URL="https://github.com/oven-sh/bun/releases/download/${OMP_BUN_VERSION}/bun-darwin-aarch64.zip"

mkdir -p "${BIN_DIR}" "${FALLBACK_DIR}" "${BUN_DIR}"

if [ ! -x "${BUN_BIN}" ]; then
  tmpdir="$(mktemp -d)"
  trap 'rm -rf "${tmpdir}"' EXIT
  curl -fsSL "${BUN_ZIP_URL}" -o "${tmpdir}/bun.zip"
  unzip -q "${tmpdir}/bun.zip" -d "${tmpdir}"
  install -m 755 "${tmpdir}/bun-darwin-aarch64/bun" "${BUN_BIN}"
fi

if [ ! -x "${FALLBACK_BIN}" ]; then
  curl -fsSL "${OMP_BASE_URL}/omp-darwin-arm64" -o "${FALLBACK_BIN}"
  chmod +x "${FALLBACK_BIN}"
fi

curl -fsSL "${OMP_BASE_URL}/pi_natives.darwin-arm64.node" -o "${NATIVE_PATH}"

cat > "${WRAPPER_PATH}" <<EOF
#!/usr/bin/env bash
set -euo pipefail
BUN_BIN="${BUN_BIN}"
SOURCE_CLI="${SOURCE_CLI}"
FALLBACK_BIN="${FALLBACK_BIN}"
if [ -x "\${BUN_BIN}" ] && [ -f "\${SOURCE_CLI}" ]; then
  exec "\${BUN_BIN}" "\${SOURCE_CLI}" "\$@"
fi
exec "\${FALLBACK_BIN}" "\$@"
EOF
chmod +x "${WRAPPER_PATH}"

if [ -x "${BUN_BIN}" ] && [ -f "${EXT_DIR}/package.json" ]; then
  (
    cd "${EXT_DIR}"
    "${BUN_BIN}" install
  )
  if [ -f "${EXT_DIR}/apply-omp-dependency-patches.mjs" ]; then
    node "${EXT_DIR}/apply-omp-dependency-patches.mjs"
  fi
fi

echo "Installed omp wrapper to ${WRAPPER_PATH}"
echo "Installed fallback omp binary to ${FALLBACK_BIN}"
echo "Installed dedicated Bun runtime to ${BUN_BIN}"
echo "Installed native addon to ${NATIVE_PATH}"