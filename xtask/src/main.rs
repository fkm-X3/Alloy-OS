use std::env;
use std::path::{Path, PathBuf};
use xshell::{cmd, Shell};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Arch {
    X86_64,
    Aarch64,
}

impl Arch {
    fn parse(s: &str) -> Self {
        match s {
            "x86_64" => Arch::X86_64,
            "aarch64" => Arch::Aarch64,
            other => panic!("Unsupported architecture: {other}. Use x86_64 or aarch64"),
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Arch::X86_64 => "x86_64",
            Arch::Aarch64 => "aarch64",
        }
    }
}

struct Config {
    arch: Arch,
    target: &'static str,
    cc: &'static str,
    ld: &'static str,
    as_bin: &'static str,
    as_flags: &'static [&'static str],
    c_flags_arch: &'static [&'static str],
    ld_flags_arch: &'static [&'static str],
    qemu: &'static str,
    qemu_flags: &'static [&'static str],
    rust_target: &'static str,
    rust_features: &'static [&'static str],
    linker: &'static str,
    boot_asm: &'static [&'static str],
    arch_asm: &'static [&'static str],
}

impl Config {
    fn new(arch: Arch) -> Self {
        match arch {
            Arch::X86_64 => Config {
                arch,
                target: "x86_64-alloy",
                cc: "gcc",
                ld: "ld",
                as_bin: "nasm",
                as_flags: &["-f", "elf64", "-dARCH_X86_64"],
                c_flags_arch: &[
                    "-m64",
                    "-DARCH_X86_64",
                    "-mno-sse",
                    "-mno-sse2",
                    "-mno-mmx",
                    "-mno-avx",
                    "-mno-80387",
                    "-mno-fp-ret-in-387",
                ],
                ld_flags_arch: &["-m", "elf_x86_64"],
                qemu: "qemu-system-x86_64",
                qemu_flags: &["-serial", "stdio"],
                rust_target: "x86_64-alloy.json",
                rust_features: &["--no-default-features", "--features", "x86_64,ported"],
                linker: "kernel/linker_x86_64.ld",
                boot_asm: &["boot/multiboot2.asm", "boot/boot_x86_64.asm"],
                arch_asm: &[
                    "kernel/c/arch/x86_64/gdt_flush.asm",
                    "kernel/c/arch/x86_64/idt_stubs.asm",
                    "kernel/c/arch/x86_64/context_switch.asm",
                    "kernel/c/arch/x86_64/syscall_entry.asm",
                ],
            },
            Arch::Aarch64 => Config {
                arch,
                target: "aarch64-alloy",
                cc: "aarch64-linux-gnu-gcc",
                ld: "aarch64-linux-gnu-ld",
                as_bin: "aarch64-linux-gnu-gcc",
                as_flags: &["-c", "-march=armv8-a"],
                c_flags_arch: &["-march=armv8-a", "-DARCH_AARCH64"],
                ld_flags_arch: &["-m", "aarch64elf"],
                qemu: "qemu-system-aarch64",
                qemu_flags: &[
                    "-machine",
                    "virt",
                    "-cpu",
                    "cortex-a53",
                    "-serial",
                    "stdio",
                ],
                rust_target: "aarch64-alloy.json",
                rust_features: &["--no-default-features", "--features", "aarch64,ported"],
                linker: "kernel/linker_aarch64.ld",
                boot_asm: &["boot/boot_aarch64.S"],
                arch_asm: &[
                    "kernel/c/arch/aarch64/context_switch.S",
                    "kernel/c/arch/aarch64/exception_vectors.S",
                ],
            },
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sh = Shell::new()?;
    let arch_str = env::var("ARCH").unwrap_or_else(|_| "x86_64".to_string());
    let config = Config::new(Arch::parse(&arch_str));

    let args: Vec<String> = env::args().skip(1).collect();
    let command = args.first().map(|s| s.as_str()).unwrap_or("all");

    match command {
        "all" => {
            build_userland(&sh, &config)?;
            build_de(&sh, &config)?;
            build_kernel_elf(&sh, &config)?;
        }
        "iso" => {
            build_iso(&sh, &config)?;
        }
        "userland" => build_userland(&sh, &config)?,
        "de-build" => {
            build_userland(&sh, &config)?;
            build_de(&sh, &config)?;
        }
        "run-de" => run_de(&sh)?,
        "run" => run_qemu(&sh, &config, false, false)?,
        "run-ahci" => run_qemu(&sh, &config, false, true)?,
        "run-elf" => run_qemu_elf(&sh, &config, false)?,
        "output" => run_qemu(&sh, &config, true, false)?,
        "output-ahci" => run_qemu(&sh, &config, true, true)?,
        "output-elf" => run_qemu_elf(&sh, &config, true)?,
        "screenshot" => run_screenshot(&sh, &config, false)?,
        "screenshot-elf" => run_screenshot(&sh, &config, true)?,
        "mouse-smoke" => run_mouse_smoke(&sh, &config, false)?,
        "mouse-smoke-elf" => run_mouse_smoke(&sh, &config, true)?,
        "mouse-screenshot" => run_mouse_screenshot(&sh, &config)?,
        "debug" => run_debug(&sh, &config, false)?,
        "debug-elf" => run_debug(&sh, &config, true)?,
        "clean" => clean(&sh)?,
        "fat32-img" => {
            create_disk_img(&sh, "build/fat32.img", 64)?;
            let _ = cmd!(sh, "mkfs.fat -F 32 build/fat32.img").run();
        }
        "lazy" => {
            clean(&sh)?;
            println!("Cleaned your shitass code, compiling the iso.");
            build_iso(&sh, &config)?;
            println!("Run 'cargo xtask run' to test.");
        }
        "print-arch" => print_arch(&config),
        "docker-build" => {
            let tag = format!("alloy-os-dev-{}:latest", config.arch.as_str());
            let dockerfile = format!("Dockerfile.{}", config.arch.as_str());
            cmd!(sh, "docker build -t {tag} -f {dockerfile} .").run()?;
        }
        "docker-run" => {
            let arch = config.arch.as_str();
            cmd!(sh, "docker compose run -e ALLOY_ARCH={arch} --rm -it alloy").run()?;
        }
        other => eprintln!("Unknown command: {other}"),
    }

    Ok(())
}

fn build_userland(sh: &Shell, config: &Config) -> Result<(), xshell::Error> {
    println!("--- Building Userland Binaries ---");
    let arch = config.arch.as_str();
    cmd!(sh, "make -C os/userland ARCH={arch}").run()?;

    let src_dir = format!("os/userland/build/{arch}");
    let copies = [
        ("hello", "hello"),
        ("compositor", "compositor"),
        ("test_wl_client", "test_wl_client"),
        ("forktest", "forktest"),
    ];

    for (src, dst) in copies {
        let from = format!("{src_dir}/{src}");
        let _ = sh.copy_file(&from, dst);
    }

    if config.arch != Arch::Aarch64 {
        let _ = sh.copy_file(format!("{src_dir}/hello_cpp"), "hello_cpp");
        let _ = sh.copy_file(format!("{src_dir}/test_qml"), "test_window");
    } else {
        sh.write_file("hello_cpp", "")?;
        sh.write_file("test_window", "")?;
    }
    Ok(())
}

fn build_de(sh: &Shell, config: &Config) -> Result<(), xshell::Error> {
    if config.arch == Arch::Aarch64 {
        println!("Skipping DE build (x86_64-only, ARCH=aarch64)");
        sh.write_file("alloy_de_qml", "")?;
        return Ok(());
    }

    println!("Building DE (cross-compile for Alloy OS x86_64)...");
    sh.create_dir("de/build")?;
    
    let qt_tools = env::var("QT_HOST_TOOLS").ok();
    let qt_path = env::var("QT_HOST_PATH").ok();

    let mut cmake_args = vec!["-B", "build", "-DCMAKE_BUILD_TYPE=Release"];
    let flag_tools;
    let flag_path;
    if let Some(ref t) = qt_tools {
        flag_tools = format!("-DQT_HOST_TOOLS_DIR={t}");
        cmake_args.push(&flag_tools);
    }
    if let Some(ref p) = qt_path {
        flag_path = format!("-DQT_HOST_PATH={p}");
        cmake_args.push(&flag_path);
    }

    let _ = sh.change_dir("de");
    let _ = cmd!(sh, "cmake {cmake_args...}").run();
    let _ = cmd!(sh, "cmake --build build --target alloy_de_qml").run();
    let _ = sh.change_dir("..");

    if sh.copy_file("de/build/alloy_de_qml", "alloy_de_qml").is_err() {
        sh.write_file("alloy_de_qml", "")?;
    }
    Ok(())
}

fn run_de(sh: &Shell) -> Result<(), xshell::Error> {
    println!("Running DE (host)...");
    let _p = sh.push_dir("de");
    cmd!(sh, "cmake -B build-host -DCMAKE_BUILD_TYPE=Release").run()?;
    cmd!(sh, "cmake --build build-host").run()?;
    let _p2 = sh.push_dir("build-host");
    cmd!(sh, "./AlloyDE").run()?;
    Ok(())
}

fn build_rust_lib(sh: &Shell, config: &Config) -> Result<(), xshell::Error> {
    println!("Building Rust kernel library ({})", config.arch.as_str());
    sh.create_dir("build/kernel/rust")?;

    let cargo = env::var("CARGO").unwrap_or_else(|_| {
        let home = env::var("HOME").unwrap_or_default();
        format!("{home}/.cargo/bin/cargo")
    });

    let _p = sh.push_dir("kernel/rust");
    let target = config.rust_target;
    let features = config.rust_features;

    cmd!(
        sh,
        "{cargo} +nightly build --release --target {target} {features...} -Zbuild-std=core,alloc -Zbuild-std-features=compiler-builtins-mem -Zjson-target-spec"
    )
    .run()?;

    let src_a = format!("target/{}/release/liballoy_kernel_rust.a", config.target);
    sh.copy_file(src_a, "../../build/kernel/rust/liballoy_kernel_rust.a")?;
    Ok(())
}

fn build_kernel_elf(sh: &Shell, config: &Config) -> Result<PathBuf, xshell::Error> {
    build_userland(sh, config)?;

    let mut objects = Vec::new();
    sh.create_dir("build")?;

    // Assemble ASM files
    let asm_files = config.boot_asm.iter().chain(config.arch_asm.iter());
    for asm in asm_files {
        let path = Path::new(asm);
        let obj = Path::new("build").join(path.with_extension("o"));
        if let Some(parent) = obj.parent() {
            sh.create_dir(parent)?;
        }

        let as_bin = config.as_bin;
        let as_flags = config.as_flags;
        cmd!(sh, "{as_bin} {as_flags...} {asm} -o {obj}").run()?;
        objects.push(obj);
    }

    // Build Rust Lib
    build_rust_lib(sh, config)?;
    let rust_lib = PathBuf::from("build/kernel/rust/liballoy_kernel_rust.a");

    // Link
    let kernel_elf = PathBuf::from("build/alloy.elf");
    println!("Linking kernel ({})...", config.arch.as_str());

    let ld = config.ld;
    let ld_flags = config.ld_flags_arch;
    let linker = config.linker;

    cmd!(
        sh,
        "{ld} {ld_flags...} -T {linker} -o {kernel_elf} {objects...} {rust_lib}"
    )
    .run()?;

    println!("Kernel built successfully: {}", kernel_elf.display());
    Ok(kernel_elf)
}

fn build_iso(sh: &Shell, config: &Config) -> Result<PathBuf, xshell::Error> {
    let kernel_elf = build_kernel_elf(sh, config)?;
    let iso = PathBuf::from("build/alloy.iso");

    println!("Creating ISO image...");
    sh.create_dir("build/isodir/boot/grub")?;
    sh.copy_file(kernel_elf, "build/isodir/boot/alloy.elf")?;
    sh.copy_file("boot/grub.cfg", "build/isodir/boot/grub/")?;
    cmd!(sh, "grub-mkrescue -o {iso} build/isodir").run()?;
    println!("ISO created: {}", iso.display());
    Ok(iso)
}

fn create_disk_img(sh: &Shell, path: &str, size_mb: u32) -> Result<(), xshell::Error> {
    if !sh.path_exists(path) {
        println!("Creating {size_mb}MB disk image...");
        sh.create_dir("build")?;
        if cmd!(sh, "qemu-img create -f raw {path} {size_mb}M").run().is_err() {
            cmd!(sh, "dd if=/dev/zero of={path} bs=1M count={size_mb}").run()?;
        }
    }
    Ok(())
}

fn get_qemu_fw() -> String {
    let candidates = [
        "/usr/share/qemu-efi-aarch64/QEMU_EFI.fd",
        "/usr/share/AAVMF/AAVMF_CODE.fd",
        "/usr/share/edk2/aarch64/QEMU_EFI.fd",
        "/opt/homebrew/share/qemu-efi-aarch64/QEMU_EFI.fd",
        "C:/msys64/usr/share/qemu-efi-aarch64/QEMU_EFI.fd",
    ];
    for c in candidates {
        if Path::new(c).exists() {
            return format!("-bios {c}");
        }
    }
    String::new()
}

fn run_qemu(
    sh: &Shell,
    config: &Config,
    display_none: bool,
    ahci: bool,
) -> Result<(), xshell::Error> {
    let qemu = config.qemu;
    let qemu_flags = config.qemu_flags;
    let display = if display_none { vec!["-display", "none"] } else { vec![] };

    if config.arch == Arch::Aarch64 {
        let elf = build_kernel_elf(sh, config)?;
        cmd!(
            sh,
            "{qemu} {qemu_flags...} {display...} -kernel {elf} -no-reboot -no-shutdown -D qemu.log"
        )
        .run()?;
    } else {
        let iso = build_iso(sh, config)?;
        create_disk_img(sh, "build/disk.img", 64)?;

        if ahci {
            cmd!(
                sh,
                "{qemu} -cdrom {iso} {qemu_flags...} {display...} -no-reboot -no-shutdown -D qemu.log -machine q35 -drive file=build/disk.img,format=raw,if=none,id=disk -device ahci,id=ahci -device ide-hd,drive=disk,bus=ahci.0"
            )
            .run()?;
        } else {
            cmd!(
                sh,
                "{qemu} -cdrom {iso} {qemu_flags...} {display...} -no-reboot -no-shutdown -D qemu.log -drive file=build/disk.img,format=raw,if=ide"
            )
            .run()?;
        }
    }
    Ok(())
}

fn run_qemu_elf(sh: &Shell, config: &Config, display_none: bool) -> Result<(), xshell::Error> {
    let elf = build_kernel_elf(sh, config)?;
    let qemu = config.qemu;
    let qemu_flags = config.qemu_flags;
    let display = if display_none { vec!["-display", "none"] } else { vec![] };

    if config.arch == Arch::Aarch64 {
        cmd!(
            sh,
            "{qemu} {qemu_flags...} {display...} -kernel {elf} -no-reboot -no-shutdown -D qemu.log"
        )
        .run()?;
    } else {
        create_disk_img(sh, "build/disk.img", 64)?;
        cmd!(
            sh,
            "{qemu} {qemu_flags...} {display...} -kernel {elf} -no-reboot -no-shutdown -D qemu.log -drive file=build/disk.img,format=raw,if=ide"
        )
        .run()?;
    }
    Ok(())
}

fn run_screenshot(sh: &Shell, config: &Config, use_elf: bool) -> Result<(), xshell::Error> {
    let qemu_fw = get_qemu_fw();
    let script = "os/userland/tools/capture_desktop_screenshot.py";

    if use_elf || config.arch == Arch::Aarch64 {
        let elf = build_kernel_elf(sh, config)?;
        if config.arch == Arch::Aarch64 {
            cmd!(
                sh,
                "python3 {script} --kernel {elf} --bios '' --qemu-binary qemu-system-aarch64 --qemu-extra '-machine virt -cpu cortex-a53 -m 128M' --output build/desktop-shell-grid.png --serial-log build/desktop-shell-boot.log --qemu-log build/qemu-screenshot.log --settle-seconds 5"
            )
            .run()?;
        } else {
            cmd!(
                sh,
                "python3 {script} --kernel {elf} --bios {qemu_fw} --output build/desktop-shell-grid.png --serial-log build/desktop-shell-boot.log --qemu-log build/qemu-screenshot.log --settle-seconds 5"
            )
            .run()?;
        }
    } else {
        let iso = build_iso(sh, config)?;
        cmd!(
            sh,
            "python3 {script} --iso {iso} --bios {qemu_fw} --output build/desktop-shell-grid.png --serial-log build/desktop-shell-boot.log --qemu-log build/qemu-screenshot.log --settle-seconds 5"
        )
        .run()?;
    }
    Ok(())
}

fn run_mouse_smoke(sh: &Shell, config: &Config, use_elf: bool) -> Result<(), xshell::Error> {
    let qemu_fw = get_qemu_fw();
    let script = "os/userland/tools/mouse_smoke.py";

    if use_elf {
        let elf = build_kernel_elf(sh, config)?;
        cmd!(
            sh,
            "python3 {script} --kernel {elf} --bios {qemu_fw} --serial-log build/mouse-smoke-boot.log --qemu-log build/qemu-mouse-smoke.log"
        )
        .run()?;
    } else {
        let iso = build_iso(sh, config)?;
        cmd!(
            sh,
            "python3 {script} --iso {iso} --bios {qemu_fw} --serial-log build/mouse-smoke-boot.log --qemu-log build/qemu-mouse-smoke.log"
        )
        .run()?;
    }
    Ok(())
}

fn run_mouse_screenshot(sh: &Shell, config: &Config) -> Result<(), xshell::Error> {
    let qemu_fw = get_qemu_fw();
    let iso = build_iso(sh, config)?;
    cmd!(
        sh,
        "python3 os/userland/tools/mouse_smoke.py --iso {iso} --bios {qemu_fw} --serial-log build/mouse-screenshot-boot.log --qemu-log build/mouse-screenshot.log --screenshot build/mouse-smoke.png ."
    )
    .run()?;
    Ok(())
}

fn run_debug(sh: &Shell, config: &Config, use_elf: bool) -> Result<(), xshell::Error> {
    let qemu = config.qemu;
    let qemu_flags = config.qemu_flags;
    let qemu_fw = get_qemu_fw();

    if use_elf || config.arch == Arch::Aarch64 {
        let elf = build_kernel_elf(sh, config)?;
        if config.arch == Arch::Aarch64 {
            cmd!(sh, "{qemu} {qemu_flags...} -kernel {elf} -s -S").run()?;
        } else {
            cmd!(sh, "{qemu} {qemu_flags...} -kernel {elf} {qemu_fw} -s -S").run()?;
        }
    } else {
        let iso = build_iso(sh, config)?;
        cmd!(sh, "{qemu} -cdrom {iso} {qemu_flags...} -s -S").run()?;
    }
    Ok(())
}

fn clean(sh: &Shell) -> Result<(), xshell::Error> {
    let _ = sh.remove_path("build");
    let _ = sh.remove_path("de/build");
    let _ = sh.remove_path("de/build-host");
    for file in ["hello", "compositor", "test_window", "alloy_de_qml"] {
        let _ = sh.remove_path(file);
    }

    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let _p = sh.push_dir("kernel/rust");
    cmd!(sh, "{cargo} clean").run()?;
    Ok(())
}

fn print_arch(config: &Config) {
    println!("ARCH = {}", config.arch.as_str());
    println!("TARGET = {}", config.target);
    println!("CC = {}", config.cc);
    println!("LD = {}", config.ld);
    println!("AS = {}", config.as_bin);
    println!("ASFLAGS = {:?}", config.as_flags);
    println!("CFLAGS_ARCH = {:?}", config.c_flags_arch);
    println!("LDFLAGS_ARCH = {:?}", config.ld_flags_arch);
    println!("QEMU = {}", config.qemu);
    println!("RUST_TARGET = {}", config.rust_target);
    println!("RUST_FEATURES = {:?}", config.rust_features);
    println!("LINKER = {}", config.linker);
}