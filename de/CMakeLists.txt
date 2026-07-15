cmake_minimum_required(VERSION 3.16)

# ── Cross-compilation (must be before project()) ────────────────
set(CMAKE_C_COMPILER x86_64-elf-gcc-qt6)
set(CMAKE_CXX_COMPILER x86_64-elf-g++-qt6)
set(CMAKE_SYSTEM_NAME Linux)
set(CMAKE_SYSTEM_PROCESSOR x86_64)
set(CMAKE_TRY_COMPILE_TARGET_TYPE STATIC_LIBRARY)

project(AlloyDE LANGUAGES C CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# ── Paths ───────────────────────────────────────────────────────
set(QT6_INSTALL   /opt/alloy/qt6)
set(QT6_QTBUILD   ${QT6_INSTALL}/build/qtbase-hangtest)
set(QT6_QDBUILD   ${QT6_INSTALL}/src/qtdeclarative/build)
set(USERLAND      ${CMAKE_SOURCE_DIR}/../os/userland)

# ── Include paths ───────────────────────────────────────────────
# NOTE: USERLAND/lib uses -iquote (not -I) to avoid shadowing system
# headers like <stdio.h> with our minimal stubs.
include_directories(
    ${CMAKE_SOURCE_DIR}
    ${CMAKE_SOURCE_DIR}/qpa
    ${CMAKE_SOURCE_DIR}/src
    ${QT6_INSTALL}/include
    ${QT6_QTBUILD}/include
    ${QT6_QTBUILD}/include/QtGui
    ${QT6_QTBUILD}/include/QtCore
    ${QT6_QTBUILD}/include/QtGui/6.4.2
    ${QT6_QTBUILD}/include/QtCore/6.4.2
    ${QT6_QTBUILD}/include/QtCore/6.4.2/QtCore
    ${QT6_QTBUILD}/include/QtGui/6.4.2/QtGui
    ${QT6_QTBUILD}/include/QtGui/6.4.2/QtGui/private
    ${QT6_QTBUILD}/include/QtCore/6.4.2/QtCore/private
    ${QT6_INSTALL}/src/qtbase/src
    ${QT6_INSTALL}/src/qtbase/include
    ${QT6_INSTALL}/src/qtbase/mkspecs/alloyos-g++
    ${QT6_INSTALL}/src/qtbase/mkspecs/linux-g++-64
    ${QT6_QTBUILD}/src/gui
    ${QT6_QDBUILD}/include
    ${QT6_QDBUILD}/include/QtQml
    ${QT6_QDBUILD}/include/QtQuick
)

# ── Compile flags ───────────────────────────────────────────────
add_compile_options(
    -m64 -O2 -Wall -Wextra
    -std=c++17 -fno-exceptions -fno-rtti
    -ffunction-sections -fdata-sections
    -iquote ${USERLAND}/lib
    -include ${CMAKE_SOURCE_DIR}/qpa/qtgui_features_fix.h
)

# ── Linker flags (matches os/userland/Makefile QT6_LDFLAGS) ─────
set(CMAKE_EXE_LINKER_FLAGS
    "-m64 -static -nostdlib -nostartfiles \
     -Wl,--gc-sections -Wl,-e_start \
     -Wl,-Ttext-segment=0x10000")

# ── QPA plugin sources ──────────────────────────────────────────
set(QPA_SRC
    qpa/alloyplatform.cpp
    qpa/alloystcreen.cpp
    qpa/alloywindow.cpp
    qpa/alloybackingstore.cpp
    qpa/alloycursor.cpp
    qpa/alloykeyboard.cpp
    qpa/alloymouse.cpp
    qpa/plugin.cpp
    qpa/new_delete.cpp
)

# ── Application sources ─────────────────────────────────────────
set(APP_SRC
    src/TerminalProcess.cpp
    src/main.cpp
)

# ── QML resources (via rcc) ─────────────────────────────────────
set(QRC_FILE ${CMAKE_SOURCE_DIR}/resources.qrc)
set(RCC_OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/qml_resources.cpp)
add_custom_command(
    OUTPUT ${RCC_OUTPUT}
    COMMAND ${QT6_INSTALL}/libexec/rcc --name qml --output ${RCC_OUTPUT} ${QRC_FILE}
    DEPENDS ${QRC_FILE}
        qml/main.qml qml/Desktop.qml qml/Panel.qml
        qml/AppLauncher.qml qml/Terminal.qml
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
    COMMENT "Compiling QML resources with rcc"
)

# ── MOC for TerminalProcess (Q_OBJECT without #include .moc) ────
set(MOC_OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/moc_TerminalProcess.cpp)
set(MOC_INCLUDES -I${QT6_QTBUILD}/include -I${QT6_QTBUILD}/include/QtCore -I${QT6_QTBUILD}/include/QtCore/6.4.2 -I${QT6_QTBUILD}/include/QtCore/6.4.2/QtCore -I${QT6_INSTALL}/src/qtbase/src -I${CMAKE_SOURCE_DIR}/src)
add_custom_command(
    OUTPUT ${MOC_OUTPUT}
    COMMAND ${QT6_INSTALL}/libexec/moc ${MOC_INCLUDES} -o ${MOC_OUTPUT} ${CMAKE_SOURCE_DIR}/src/TerminalProcess.h
    DEPENDS ${CMAKE_SOURCE_DIR}/src/TerminalProcess.h
    COMMENT "Running MOC on TerminalProcess.h"
)

# ── Userland stubs (compiled from source) ───────────────────────
set(USERLAND_STUBS
    ${USERLAND}/lib/posix_stubs.c
    ${USERLAND}/lib/cxxabi_stubs.cpp
    ${USERLAND}/lib/qt6_syscall_stub.c
    ${USERLAND}/lib/qt6_platform_stubs.cpp
    ${USERLAND}/lib/qt6_extra_stubs.cpp
    ${USERLAND}/lib/wayland_client.c
    ${USERLAND}/lib/stdlib.c
)

# ── Qt6 static libraries ───────────────────────────────────────
set(QT6_BASE_LIBS
    ${QT6_INSTALL}/lib/libQt6Gui.a
    ${QT6_INSTALL}/lib/libQt6Core.a
    ${QT6_INSTALL}/lib/libQt6BundledPcre2.a
    ${QT6_INSTALL}/lib/libQt6BundledFreetype.a
    ${QT6_INSTALL}/lib/libQt6BundledHarfbuzz.a
    ${QT6_INSTALL}/lib/libQt6BundledLibpng.a
    ${QT6_INSTALL}/lib/libQt6BundledZLIB.a
)

set(QT6_QML_LIBS
    ${QT6_QDBUILD}/lib/libQt6Qml.a
    ${QT6_QDBUILD}/lib/libQt6QmlModels.a
    ${QT6_QDBUILD}/lib/libQt6Quick.a
    ${QT6_QDBUILD}/lib/libQt6QuickControls2.a
    ${QT6_QDBUILD}/lib/libQt6QuickLayouts.a
    ${QT6_QDBUILD}/lib/libQt6QuickTemplates2.a
)

# ── Userland CRT objects (pre-built) ────────────────────────────
set(USERLAND_CRT
    ${USERLAND}/build/x86_64/crt0.o
    ${USERLAND}/build/x86_64/crt_cpp.o
)

# ── Executable ──────────────────────────────────────────────────
add_executable(alloy_de_qml
    ${USERLAND_CRT}
    ${APP_SRC}
    ${QPA_SRC}
    ${RCC_OUTPUT}
    ${MOC_OUTPUT}
    ${USERLAND_STUBS}
)

# Link in the correct order: CRT, then objects, then Qt6 libs in --start-group
target_link_libraries(alloy_de_qml PRIVATE
    -Wl,--start-group
    ${QT6_QML_LIBS}
    ${QT6_BASE_LIBS}
    -lgcc
    -Wl,--end-group
)
