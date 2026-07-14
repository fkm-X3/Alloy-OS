# Alloy OS Userland Build System
ARCH ?= x86_64

ifeq ($(ARCH),x86_64)
    CC = gcc
    CXX = g++
    LD = ld
    CFLAGS = -m64 -std=gnu11 -ffreestanding -nostdlib -fno-builtin -Wall -Wextra -O2 -Ilib
    CXXFLAGS = -m64 -std=c++17 -ffreestanding -nostdlib -fno-builtin -fno-exceptions -fno-rtti -Wall -Wextra -O2 -Ilib
    LDFLAGS = -m elf_x86_64 -T userland.ld -static -nostdlib
else ifeq ($(ARCH),aarch64)
    CC = aarch64-linux-gnu-gcc
    CXX = aarch64-linux-gnu-g++
    LD = aarch64-linux-gnu-ld
    CFLAGS = -march=armv8-a -std=gnu11 -ffreestanding -nostdlib -fno-builtin -Wall -Wextra -O2 -Ilib
    CXXFLAGS = -march=armv8-a -std=c++17 -ffreestanding -nostdlib -fno-builtin -fno-exceptions -fno-rtti -Wall -Wextra -O2 -Ilib
    LDFLAGS = -m aarch64elf -T userland.ld -static -nostdlib
else
    $(error Unsupported architecture: $(ARCH). Use x86_64 or aarch64)
endif

LIB_DIR = lib
TEST_DIR = tests
BUILD_DIR = build/$(ARCH)

# Library objects (order: crt0, crt_cpp, then the rest)
CRT0_OBJ = $(BUILD_DIR)/crt0.o
CRT_CPP_OBJ = $(BUILD_DIR)/crt_cpp.o
C_LIB_OBJS = $(BUILD_DIR)/stdlib.o $(BUILD_DIR)/stdio.o $(BUILD_DIR)/wayland_client.o
CPP_LIB_OBJS = $(BUILD_DIR)/cxxabi.o
LIB_OBJS = $(CRT0_OBJ) $(CRT_CPP_OBJ) $(C_LIB_OBJS) $(CPP_LIB_OBJS)

# Test binaries
TEST_C_SRCS = $(wildcard $(TEST_DIR)/*.c)
TEST_CPP_SRCS = $(wildcard $(TEST_DIR)/*.cpp)
TEST_C_BINS = $(patsubst $(TEST_DIR)/%.c,$(BUILD_DIR)/%,$(TEST_C_SRCS))
TEST_CPP_BINS = $(patsubst $(TEST_DIR)/%.cpp,$(BUILD_DIR)/%,$(TEST_CPP_SRCS))
TEST_BINS = $(TEST_C_BINS) $(TEST_CPP_BINS)

ALL_OBJS = $(LIB_OBJS) $(patsubst $(BUILD_DIR)/%,$(BUILD_DIR)/%.o,$(TEST_BINS))

.PHONY: all clean

all: $(BUILD_DIR) $(LIB_OBJS) $(TEST_BINS)

$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

# Assembly entry point
$(CRT0_OBJ): $(LIB_DIR)/crt0_$(ARCH).S | $(BUILD_DIR)
	$(CC) $(CFLAGS) -c $< -o $@

# CRT C++ (compiled as C)
$(CRT_CPP_OBJ): $(LIB_DIR)/crt_cpp.c | $(BUILD_DIR)
	$(CC) $(CFLAGS) -c $< -o $@

# C library objects
$(BUILD_DIR)/%.o: $(LIB_DIR)/%.c | $(BUILD_DIR)
	$(CC) $(CFLAGS) -c $< -o $@

# C test binaries
$(BUILD_DIR)/%: $(TEST_DIR)/%.c $(LIB_OBJS)
	$(CC) $(CFLAGS) -c $< -o $(BUILD_DIR)/$*.o
	$(LD) $(LDFLAGS) -o $@ $(CRT0_OBJ) $(CRT_CPP_OBJ) $(BUILD_DIR)/$*.o $(C_LIB_OBJS) $(CPP_LIB_OBJS)

# C++ test binaries
$(BUILD_DIR)/%: $(TEST_DIR)/%.cpp $(LIB_OBJS)
	$(CXX) $(CXXFLAGS) -c $< -o $(BUILD_DIR)/$*.o
	$(LD) $(LDFLAGS) -o $@ $(CRT0_OBJ) $(CRT_CPP_OBJ) $(BUILD_DIR)/$*.o $(C_LIB_OBJS) $(CPP_LIB_OBJS)

# ── Qt6 Test App (x86_64 only) ─────────────────────────────────────────────────
# Compiles the minimal_window test + Fusion QPA plugin against static Qt6 libs.
# Uses host g++ (not freestanding) for compilation, then links with userland.ld.

QT6_CXX = x86_64-elf-g++-qt6
QT6_LD = ld
QT6_BASE = /opt/alloy/qt6
QT6_BUILD = $(QT6_BASE)/build/qtbase-hangtest
QT6_INCLUDES = -I$(QT6_BASE)/include \
               -I$(QT6_BUILD)/include \
               -I$(QT6_BUILD)/include/QtGui \
               -I$(QT6_BUILD)/include/QtCore \
               -I$(QT6_BUILD)/include/QtGui/6.4.2 \
               -I$(QT6_BUILD)/include/QtCore/6.4.2 \
               -I$(QT6_BUILD)/include/QtCore/6.4.2/QtCore \
               -I$(QT6_BUILD)/include/QtGui/6.4.2/QtGui \
               -I$(QT6_BUILD)/include/QtGui/6.4.2/QtGui/private \
               -I$(QT6_BUILD)/include/QtCore/6.4.2/QtCore/private \
               -I$(QT6_BASE)/src/qtbase/src \
               -I$(QT6_BASE)/src/qtbase/include \
               -I$(QT6_BASE)/src/qtbase/mkspecs/alloyos-g++ \
               -I$(QT6_BASE)/src/qtbase/mkspecs/linux-g++-64 \
               -I$(QT6_BUILD)/src/gui \
               -iquote $(LIB_DIR) \
               -I../../de/qpa
QT6_CXXFLAGS = -m64 -std=c++17 -fno-exceptions -Wall -O2 -ffunction-sections -fdata-sections $(QT6_INCLUDES) -include qtgui_features_fix.h
QT6_LD = gcc
QT6_LDFLAGS = -m64 -static -nostdlib -nostartfiles -Wl,--gc-sections -Wl,-e_start -Wl,-Ttext-segment=0x10000
QT6_LIBS = /opt/alloy/qt6/lib/libQt6Gui.a \
           /opt/alloy/qt6/lib/libQt6Core.a \
           /opt/alloy/qt6/lib/libQt6BundledPcre2.a \
           /opt/alloy/qt6/lib/libQt6BundledFreetype.a \
           /opt/alloy/qt6/lib/libQt6BundledHarfbuzz.a \
           /opt/alloy/qt6/lib/libQt6BundledLibpng.a \
           /opt/alloy/qt6/lib/libQt6BundledZLIB.a
QD_BUILD = /opt/alloy/qt6/src/qtdeclarative/build
QT6_QML_LIBS = $(QD_BUILD)/lib/libQt6Qml.a \
               $(QD_BUILD)/lib/libQt6Quick.a \
               $(QD_BUILD)/lib/libQt6QuickControls2.a \
               $(QD_BUILD)/lib/libQt6QuickLayouts.a \
               $(QD_BUILD)/lib/libQt6QuickTemplates2.a
QT6_HOSTLIBS = -lgcc

# Qt6 test binary (only for x86_64 with Qt6 libraries available)
QT6_AVAILABLE := $(wildcard /opt/alloy/qt6/lib/libQt6Core.a)
ifeq ($(ARCH),x86_64)
ifneq ($(QT6_AVAILABLE),)
QT6_BUILD_DIR = $(BUILD_DIR)/qt6_test
QT6_TEST_BIN = $(BUILD_DIR)/test_window

# Source files that need Qt6-aware compilation (paths relative to os/userland/)
QPA_SRCS = ../../de/qpa/alloyplatform.cpp ../../de/qpa/alloystcreen.cpp \
           ../../de/qpa/alloywindow.cpp ../../de/qpa/alloybackingstore.cpp \
           ../../de/qpa/alloycursor.cpp ../../de/qpa/alloykeyboard.cpp \
           ../../de/qpa/alloymouse.cpp ../../de/qpa/plugin.cpp
QT6_SRCS = $(QPA_SRCS) ../../de/tests/minimal_window.cpp

# Objects (basename-based to keep object names short)
QT6_OBJS = $(QT6_BUILD_DIR)/alloyplatform.o \
           $(QT6_BUILD_DIR)/alloystcreen.o \
           $(QT6_BUILD_DIR)/alloywindow.o \
           $(QT6_BUILD_DIR)/alloybackingstore.o \
           $(QT6_BUILD_DIR)/alloycursor.o \
           $(QT6_BUILD_DIR)/alloykeyboard.o \
           $(QT6_BUILD_DIR)/alloymouse.o \
           $(QT6_BUILD_DIR)/plugin.o \
           $(QT6_BUILD_DIR)/minimal_window.o \
           $(QT6_BUILD_DIR)/posix_stubs.o \
           $(QT6_BUILD_DIR)/cxxabi_stubs.o \
           $(QT6_BUILD_DIR)/qt6_syscall_stub.o \
           $(QT6_BUILD_DIR)/wayland_client.o \
           $(QT6_BUILD_DIR)/new_delete.o \
           $(QT6_BUILD_DIR)/qt6_platform_stubs.o \
           $(QT6_BUILD_DIR)/qt6_extra_stubs.o \
           $(BUILD_DIR)/stdlib.o

# Qt6 test binary depends on all these objects and Qt6 libs
$(QT6_TEST_BIN): $(QT6_OBJS) $(CRT0_OBJ) $(CRT_CPP_OBJ)
	@mkdir -p $(dir $@)
	$(QT6_LD) $(QT6_LDFLAGS) -o $@ $(CRT0_OBJ) $(CRT_CPP_OBJ) \
	    $(QT6_OBJS) -Wl,--start-group $(QT6_LIBS) $(QT6_HOSTLIBS) -Wl,--end-group

# Rule to compile Qt6 sources (with host g++, not freestanding)
$(QT6_BUILD_DIR)/minimal_window.o: ../../de/tests/minimal_window.cpp | $(QT6_BUILD_DIR)
	$(QT6_CXX) $(QT6_CXXFLAGS) -c $< -o $@

$(QT6_BUILD_DIR)/alloyplatform.o: ../../de/qpa/alloyplatform.cpp | $(QT6_BUILD_DIR)
	$(QT6_CXX) $(QT6_CXXFLAGS) -c $< -o $@

$(QT6_BUILD_DIR)/alloystcreen.o: ../../de/qpa/alloystcreen.cpp | $(QT6_BUILD_DIR)
	$(QT6_CXX) $(QT6_CXXFLAGS) -c $< -o $@

$(QT6_BUILD_DIR)/alloywindow.o: ../../de/qpa/alloywindow.cpp | $(QT6_BUILD_DIR)
	$(QT6_CXX) $(QT6_CXXFLAGS) -c $< -o $@

$(QT6_BUILD_DIR)/alloybackingstore.o: ../../de/qpa/alloybackingstore.cpp | $(QT6_BUILD_DIR)
	$(QT6_CXX) $(QT6_CXXFLAGS) -c $< -o $@

$(QT6_BUILD_DIR)/alloycursor.o: ../../de/qpa/alloycursor.cpp | $(QT6_BUILD_DIR)
	$(QT6_CXX) $(QT6_CXXFLAGS) -c $< -o $@

$(QT6_BUILD_DIR)/alloykeyboard.o: ../../de/qpa/alloykeyboard.cpp | $(QT6_BUILD_DIR)
	$(QT6_CXX) $(QT6_CXXFLAGS) -c $< -o $@

$(QT6_BUILD_DIR)/alloymouse.o: ../../de/qpa/alloymouse.cpp | $(QT6_BUILD_DIR)
	$(QT6_CXX) $(QT6_CXXFLAGS) -c $< -o $@

$(QT6_BUILD_DIR)/plugin.o: ../../de/qpa/plugin.cpp | $(QT6_BUILD_DIR)
	$(QT6_CXX) $(QT6_CXXFLAGS) -c $< -o $@

# Posix stubs and wayland client compiled with Qt6-aware host toolchain
$(QT6_BUILD_DIR)/posix_stubs.o: $(LIB_DIR)/posix_stubs.c | $(QT6_BUILD_DIR)
	$(QT6_CXX) $(QT6_CXXFLAGS) -x c -c $< -o $@

$(QT6_BUILD_DIR)/wayland_client.o: $(LIB_DIR)/wayland_client.c | $(QT6_BUILD_DIR)
	$(QT6_CXX) $(QT6_CXXFLAGS) -x c -c $< -o $@

# C++ operator new/delete stubs
$(QT6_BUILD_DIR)/new_delete.o: ../../de/qpa/new_delete.cpp | $(QT6_BUILD_DIR)
	$(QT6_CXX) $(QT6_CXXFLAGS) -c $< -o $@

$(QT6_BUILD_DIR)/cxxabi_stubs.o: $(LIB_DIR)/cxxabi_stubs.cpp | $(QT6_BUILD_DIR)
	$(QT6_CXX) $(QT6_CXXFLAGS) -c $< -o $@

$(QT6_BUILD_DIR)/qt6_syscall_stub.o: $(LIB_DIR)/qt6_syscall_stub.c | $(QT6_BUILD_DIR)
	$(QT6_CXX) $(QT6_CXXFLAGS) -x c -c $< -o $@

$(QT6_BUILD_DIR)/qt6_platform_stubs.o: $(LIB_DIR)/qt6_platform_stubs.cpp | $(QT6_BUILD_DIR)
	$(QT6_CXX) $(QT6_CXXFLAGS) -c $< -o $@

$(QT6_BUILD_DIR)/qt6_extra_stubs.o: $(LIB_DIR)/qt6_extra_stubs.cpp | $(QT6_BUILD_DIR)
	$(QT6_CXX) $(QT6_CXXFLAGS) -c $< -o $@

$(QT6_BUILD_DIR):
	mkdir -p $(QT6_BUILD_DIR)

# ── Qt6 QML test binary ───────────────────────────────────────────────────
QT6_QML_TEST_BIN = $(BUILD_DIR)/test_qml
QT6_QML_OBJS = $(QT6_BUILD_DIR)/alloyplatform.o \
               $(QT6_BUILD_DIR)/alloystcreen.o \
               $(QT6_BUILD_DIR)/alloywindow.o \
               $(QT6_BUILD_DIR)/alloybackingstore.o \
               $(QT6_BUILD_DIR)/alloycursor.o \
               $(QT6_BUILD_DIR)/alloykeyboard.o \
               $(QT6_BUILD_DIR)/alloymouse.o \
               $(QT6_BUILD_DIR)/plugin.o \
               $(QT6_BUILD_DIR)/minimal_qml.o \
               $(QT6_BUILD_DIR)/TerminalProcess.o \
               $(QT6_BUILD_DIR)/posix_stubs.o \
               $(QT6_BUILD_DIR)/cxxabi_stubs.o \
               $(QT6_BUILD_DIR)/qt6_syscall_stub.o \
               $(QT6_BUILD_DIR)/wayland_client.o \
               $(QT6_BUILD_DIR)/new_delete.o \
               $(QT6_BUILD_DIR)/qt6_platform_stubs.o \
               $(QT6_BUILD_DIR)/qt6_extra_stubs.o \
               $(BUILD_DIR)/stdlib.o

$(QT6_QML_TEST_BIN): $(QT6_QML_OBJS) $(CRT0_OBJ) $(CRT_CPP_OBJ)
	@mkdir -p $(dir $@)
	$(QT6_LD) $(QT6_LDFLAGS) -o $@ $(CRT0_OBJ) $(CRT_CPP_OBJ) \
	    $(QT6_QML_OBJS) -Wl,--start-group $(QT6_LIBS) $(QT6_QML_LIBS) $(QT6_HOSTLIBS) -Wl,--end-group

$(QT6_BUILD_DIR)/minimal_qml.o: ../../de/tests/minimal_qml.cpp | $(QT6_BUILD_DIR)
	$(QT6_CXX) $(QT6_CXXFLAGS) -I$(QD_BUILD)/include -I$(QD_BUILD)/include/QtQml -I$(QD_BUILD)/include/QtQuick -c $< -o $@

$(QT6_BUILD_DIR)/TerminalProcess.o: ../../de/src/TerminalProcess.cpp ../../de/src/TerminalProcess.h ../../de/src/alloy_syscalls.h | $(QT6_BUILD_DIR)
	$(QT6_CXX) $(QT6_CXXFLAGS) -I$(QD_BUILD)/include -I$(QD_BUILD)/include/QtQml -I$(QD_BUILD)/include/QtQuick -c $< -o $@

# Build target for Qt6 test
.PHONY: qt6-test qt6-qml-test
qt6-test: $(QT6_TEST_BIN)
qt6-qml-test: $(QT6_QML_TEST_BIN)

all: $(QT6_TEST_BIN) $(QT6_QML_TEST_BIN)
endif # QT6_AVAILABLE
endif # ARCH == x86_64

clean:
	rm -rf $(BUILD_DIR)
