# tools/build-qt6.sh
set -euo pipefail

QT_VERSION="6.4.2"
QT_MAJOR="6.4"
DESTDIR="${DESTDIR:-}"
INSTALL_PREFIX="${DESTDIR}/opt/alloy/qt6"
JOBS="${JOBS:-$(nproc)}"
WORK_DIR=$(pwd)/qt6-build-tmp

echo "=== Building Qt6 ${QT_VERSION} for Alloy OS ==="
echo "Install prefix: ${INSTALL_PREFIX}"
echo "Parallel jobs:  ${JOBS}"

# ── Install prefix ──────────────────────────────────────────────
mkdir -p "${INSTALL_PREFIX}"

# ── Clone sources ───────────────────────────────────────────────
mkdir -p "${WORK_DIR}/src"
cd "${WORK_DIR}/src"

for MODULE in qtbase qtdeclarative qtshadertools; do
    if [ ! -d "${MODULE}" ]; then
        echo "--- Cloning ${MODULE} v${QT_VERSION} ---"
        git clone --depth 1 --branch "v${QT_VERSION}" \
            "https://code.qt.io/qt/${MODULE}.git" "${MODULE}"
    fi
done

# ── Create alloyos-g++ mkspec ──────────────────────────────────
MKSPEC_DIR="${WORK_DIR}/src/qtbase/mkspecs/alloyos-g++"
mkdir -p "${MKSPEC_DIR}"
cat > "${MKSPEC_DIR}/qplatformdefs.h" <<'EOF'
#include "../linux-g++/qplatformdefs.h"
EOF

# ── Compiler wrappers (host g++ -m64, used by DE build) ────────
# These are only used by the DE/userland builds, not for Qt6 itself.
CROSS_DIR="${WORK_DIR}/cross-bin"
mkdir -p "${CROSS_DIR}"

cat > "${CROSS_DIR}/x86_64-elf-gcc-qt6" <<'WRAPPER'
#!/bin/bash
exec /usr/bin/gcc -m64 "$@"
WRAPPER
cat > "${CROSS_DIR}/x86_64-elf-g++-qt6" <<'WRAPPER'
#!/bin/bash
exec /usr/bin/g++ -m64 "$@"
WRAPPER
chmod +x "${CROSS_DIR}/x86_64-elf-gcc-qt6" "${CROSS_DIR}/x86_64-elf-g++-qt6"

# ── Build qtbase (native build) ─────────────────────────────────
# Build Qt6 natively on the host. Host == target (x86_64 Linux), so
# no cross-compilation flags. This avoids the QT_HOST_PATH requirement.
QTBASE_SRC="${WORK_DIR}/src/qtbase"
QTBASE_BUILD="${WORK_DIR}/build/qtbase-hangtest"
QTBASE_INSTALL="${INSTALL_PREFIX}"

# Clean cmake state to prevent stale CMAKE_CROSSCOMPILING from cache
rm -rf "${QTBASE_BUILD}"
mkdir -p "${QTBASE_BUILD}"
cd "${QTBASE_BUILD}"

echo "--- Configuring qtbase ---"
cmake "${QTBASE_SRC}" \
    -G Ninja \
    -DCMAKE_INSTALL_PREFIX="${QTBASE_INSTALL}" \
    -DCMAKE_C_FLAGS="-m64" \
    -DCMAKE_CXX_FLAGS="-m64" \
    -DBUILD_SHARED_LIBS=OFF \
    -DQT_QMAKE_TARGET_MKSPEC=alloyos-g++ \
    -DFEATURE_xkbcommon=OFF \
    -DFEATURE_evdev=OFF \
    -DFEATURE_linuxfb=OFF \
    -DFEATURE_kms=OFF \
    -DFEATURE_drm_atomic=OFF \
    -DFEATURE_opengl=OFF \
    -DFEATURE_vulkan=OFF \
    -DFEATURE_sql=OFF \
    -DFEATURE_xml=OFF \
    -DFEATURE_network=OFF \
    -DFEATURE_dbus=OFF \
    -DFEATURE_testlib=OFF \
    -DINPUT_doubleconversion=qt \
    -DFEATURE_system_pcre2=OFF \
    2>&1 | tee "${WORK_DIR}/qtbase-configure.log"

echo "--- Building qtbase (Core + Gui) ---"
cmake --build . --target Qt6Core Qt6Gui -j"${JOBS}" \
    2>&1 | tee "${WORK_DIR}/qtbase-build.log"

echo "--- Installing qtbase ---"
cmake --install . \
    2>&1 | tee "${WORK_DIR}/qtbase-install.log"

# ── Build qtdeclarative ─────────────────────────────────────────
QTDECL_SRC="${WORK_DIR}/src/qtdeclarative"
QTDECL_BUILD="${WORK_DIR}/src/qtdeclarative/build"
QTDECL_INSTALL="${INSTALL_PREFIX}"

# Clean cmake state
rm -rf "${QTDECL_BUILD}"
mkdir -p "${QTDECL_BUILD}"
cd "${QTDECL_BUILD}"

echo "--- Configuring qtdeclarative ---"
cmake "${QTDECL_SRC}" \
    -G Ninja \
    -DCMAKE_INSTALL_PREFIX="${QTDECL_INSTALL}" \
    -DCMAKE_PREFIX_PATH="${QTDECL_INSTALL}" \
    -DCMAKE_C_FLAGS="-m64" \
    -DCMAKE_CXX_FLAGS="-m64" \
    -DBUILD_SHARED_LIBS=OFF \
    -DQT_QMAKE_TARGET_MKSPEC=alloyos-g++ \
    -DFEATURE_qml_debug=OFF \
    -DFEATURE_qml_jit=OFF \
    -DFEATURE_qml_network=OFF \
    -DFEATURE_qml_profiler=OFF \
    -DFEATURE_qml_worker_script=ON \
    2>&1 | tee "${WORK_DIR}/qtdeclarative-configure.log"

echo "--- Building qtdeclarative (Qml + Quick) ---"
cmake --build . --target Qt6Qml Qt6QmlModels Qt6Quick \
    Qt6QuickControls2 Qt6QuickLayouts Qt6QuickTemplates2 \
    -j"${JOBS}" \
    2>&1 | tee "${WORK_DIR}/qtdeclarative-build.log"

echo "--- Installing qtdeclarative ---"
cmake --install . \
    2>&1 | tee "${WORK_DIR}/qtdeclarative-install.log"

# ── Package ─────────────────────────────────────────────────────
echo "=== Qt6 build complete ==="
echo "Installed to: ${INSTALL_PREFIX}"
echo ""
echo "Contents:"
ls -lh "${INSTALL_PREFIX}/lib/" 2>/dev/null | head -20
echo ""
echo "Tools:"
ls -lh "${INSTALL_PREFIX}/libexec/" 2>/dev/null
ls -lh "${INSTALL_PREFIX}/bin/" 2>/dev/null

# ── Create archive (for CI upload) ──────────────────────────────
if [ "${CREATE_ARCHIVE:-0}" = "1" ]; then
    ARCHIVE="qt6-alloy-x86_64.tar.gz"
    echo "--- Creating archive: ${ARCHIVE} ---"
    cd "${INSTALL_PREFIX}/../.."
    tar czf "${WORK_DIR}/${ARCHIVE}" opt/alloy/qt6
    echo "Archive: ${WORK_DIR}/${ARCHIVE}"
    ls -lh "${WORK_DIR}/${ARCHIVE}"
fi

# Cleanup
echo "--- Cleaning up build directory ---"
rm -rf "${WORK_DIR}"

echo "=== Done ==="
