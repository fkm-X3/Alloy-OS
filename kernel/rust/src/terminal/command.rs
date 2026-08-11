//! Command system for terminal
//! 
//! Defines the Command trait and provides a registry for command lookup

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use crate::terminal::colors;

const OS_NAME: &str = "Alloy Operating System";
const OS_VERSION: &str = "0.7.0-dev (Phase 7)";
const OS_ARCH: &str = "x86 (32-bit)";
const OS_LANGUAGE: &str = "C++ + Rust";
const OS_UNAME: &str = "AlloyOS";

fn print_u32_decimal_line(label: &str, value: u32) {
    use crate::utils::format;

    let value_buf = format::u32_to_decimal(value);
    let value_start = format::trim_leading_spaces(&value_buf);
    crate::VgaText::print(label);
    unsafe {
        crate::terminal::println_cstr(&value_buf[value_start..]);
    }
}

fn print_u64_decimal_line(label: &str, value: u64) {
    use crate::utils::format;

    let value_buf = format::u64_to_decimal(value);
    let value_start = format::trim_leading_spaces(&value_buf);
    crate::VgaText::print(label);
    unsafe {
        crate::terminal::println_cstr(&value_buf[value_start..]);
    }
}

fn print_size_line(label: &str, bytes: u64) {
    use crate::utils::format;

    let (value_buf, unit_buf) = format::format_bytes(bytes);
    let value_start = format::trim_leading_spaces(&value_buf);
    crate::VgaText::print(label);
    unsafe {
        crate::terminal::print_cstr(&value_buf[value_start..]);
        crate::VgaText::print_bytes(b" ");
        crate::terminal::println_cstr(&unit_buf[0..]);
    }
}

fn print_uptime_value(uptime_ms: u64) {
    use crate::utils::format;

    let total_seconds = uptime_ms / 1000;
    let seconds = total_seconds % 60;
    let total_minutes = total_seconds / 60;
    let minutes = total_minutes % 60;
    let total_hours = total_minutes / 60;
    let hours = total_hours % 24;
    let days = total_hours / 24;

    if days > 0 {
        let days_str = format::u32_to_decimal(days as u32);
        let days_start = format::trim_leading_spaces(&days_str);
        unsafe {
            crate::terminal::print_cstr(&days_str[days_start..]);
            if days == 1 {
                crate::VgaText::print_bytes(b" day, ");
            } else {
                crate::VgaText::print_bytes(b" days, ");
            }
        }
    }

    let hours_str = format::u32_to_decimal(hours as u32);
    let minutes_str = format::u32_to_decimal(minutes as u32);
    let seconds_str = format::u32_to_decimal(seconds as u32);

    unsafe {
        crate::terminal::print_cstr(
            &hours_str[format::trim_leading_spaces(&hours_str)..]
        );
        crate::VgaText::print_bytes(b":");

        if minutes < 10 {
            crate::VgaText::print_bytes(b"0");
        }
        crate::terminal::print_cstr(
            &minutes_str[format::trim_leading_spaces(&minutes_str)..]
        );
        crate::VgaText::print_bytes(b":");

        if seconds < 10 {
            crate::VgaText::print_bytes(b"0");
        }
        crate::terminal::println_cstr(
            &seconds_str[format::trim_leading_spaces(&seconds_str)..]
        );
    }
}

/// Command trait for terminal commands
pub trait Command {
    /// Get command name
    fn name(&self) -> &str;
    
    /// Get command help text
    fn help(&self) -> &str;
    
    /// Execute the command with given arguments and registry context
    fn execute(&self, args: &[&str], registry: &CommandRegistry) -> Result<(), &str>;
}

/// Command registry
pub struct CommandRegistry {
    commands: BTreeMap<String, Box<dyn Command>>,
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandRegistry {
    pub fn new() -> Self {
        CommandRegistry {
            commands: BTreeMap::new(),
        }
    }
    
    /// Register a command
    pub fn register(&mut self, cmd: Box<dyn Command>) {
        let name = String::from(cmd.name());
        self.commands.insert(name, cmd);
    }
    
    /// Execute a command by name
    pub fn execute(&self, name: &str, args: &[&str]) {
        if let Some(cmd) = self.commands.get(name) {
            match cmd.execute(args, self) {
                Ok(_) => {},
                Err(err) => {
                    colors::print_error(err);
                }
            }
        } else {
            colors::print_error(&alloc::format!("Unknown command: {}", name));
        }
    }
    
    /// Get all registered command names
    pub fn get_commands(&self) -> Vec<&str> {
        self.commands.keys().map(|s| s.as_str()).collect()
    }
    
    /// Get a specific command
    pub fn get(&self, name: &str) -> Option<&dyn Command> {
        self.commands.get(name).map(|cmd| cmd.as_ref())
    }
}

// Built-in commands

/// Help command
pub struct HelpCommand;

impl Command for HelpCommand {
    fn name(&self) -> &str {
        "help"
    }
    
    fn help(&self) -> &str {
        "Display available commands or help for a specific command"
    }
    
    fn execute(&self, args: &[&str], registry: &CommandRegistry) -> Result<(), &str> {
        if args.len() > 1 {
            return Err("Usage: help [command]");
        }

        if let Some(command_name) = args.first() {
            if let Some(command) = registry.get(command_name) {
                colors::print_info(&alloc::format!(
                    "{:<8} - {}",
                    command_name,
                    command.help()
                ));
                return Ok(());
            }
            return Err("Command not found");
        }

        colors::print_info("Available commands:");
        for command_name in registry.get_commands() {
            if let Some(command) = registry.get(command_name) {
                crate::VgaText::println(&alloc::format!(
                    "  {:<8} - {}",
                    command_name,
                    command.help()
                ));
            }
        }
        Ok(())
    }
}

/// Clear command
pub struct ClearCommand;

impl Command for ClearCommand {
    fn name(&self) -> &str {
        "clear"
    }
    
    fn help(&self) -> &str {
        "Clear the screen"
    }
    
    fn execute(&self, _args: &[&str], _registry: &CommandRegistry) -> Result<(), &str> {
        unsafe {
            // Clear screen by printing 25 empty lines
            crate::VgaText::set_color(0, 0);
            for _ in 0..25 {
                crate::VgaText::println_bytes(b"");
            }
            crate::VgaText::set_color(7, 0);
        }
        Ok(())
    }
}

/// Echo command
pub struct EchoCommand;

impl Command for EchoCommand {
    fn name(&self) -> &str {
        "echo"
    }
    
    fn help(&self) -> &str {
        "Print arguments to the screen"
    }
    
    fn execute(&self, args: &[&str], _registry: &CommandRegistry) -> Result<(), &str> {
        if args.is_empty() {
            crate::VgaText::println("");
        } else {
            let text = args.join(" ");
            crate::VgaText::println(&text);
        }
        Ok(())
    }
}

/// Version command
pub struct VersionCommand;

impl Command for VersionCommand {
    fn name(&self) -> &str {
        "version"
    }
    
    fn help(&self) -> &str {
        "Display OS version information"
    }
    
    fn execute(&self, _args: &[&str], _registry: &CommandRegistry) -> Result<(), &str> {
        colors::print_info(OS_NAME);
        crate::VgaText::println(&alloc::format!("Version: {}", OS_VERSION));
        crate::VgaText::println(&alloc::format!("Architecture: {}", OS_ARCH));
        crate::VgaText::println(&alloc::format!("Language: {}", OS_LANGUAGE));
        crate::VgaText::println("");
        crate::VgaText::println("Features:");
        crate::VgaText::println("  [x] Multiboot2 boot");
        crate::VgaText::println("  [x] VGA text mode");
        crate::VgaText::println("  [x] PS/2 keyboard");
        crate::VgaText::println("  [x] Memory management");
        crate::VgaText::println("  [x] Rust integration");
        crate::VgaText::println("  [x] Terminal interface");
        crate::VgaText::println("  [x] Diagnostic commands");
        Ok(())
    }
}

/// System summary command
pub struct SysinfoCommand;

impl Command for SysinfoCommand {
    fn name(&self) -> &str {
        "sysinfo"
    }

    fn help(&self) -> &str {
        "Display compact system summary"
    }

    fn execute(&self, args: &[&str], _registry: &CommandRegistry) -> Result<(), &str> {
        if !args.is_empty() {
            return Err("Usage: sysinfo");
        }

        colors::print_info("System Summary");
        crate::VgaText::println("");
        crate::VgaText::println(OS_NAME);
        crate::VgaText::println(&alloc::format!("Version: {}", OS_VERSION));
        crate::VgaText::println(&alloc::format!("Architecture: {}", OS_ARCH));

        let mut vendor = [0u8; 13];
        unsafe {
            crate::ffi::cpu_get_vendor_ffi(vendor.as_mut_ptr());
            crate::VgaText::print_bytes(b"CPU Vendor: ");
            crate::terminal::println_cstr(&vendor[..]);
        }

        let total_memory = alloy_kernel_hal::mem::total_memory();
        let available_memory = alloy_kernel_hal::mem::available_memory();
        let used_memory = total_memory.saturating_sub(available_memory);
        print_size_line("Memory Total: ", total_memory);
        print_size_line("Memory Used:  ", used_memory);
        print_size_line("Memory Free:  ", available_memory);

        let uptime_ms = unsafe { crate::SystemTimer::uptime_ms() };
        crate::VgaText::print("Uptime: ");
        print_uptime_value(uptime_ms);
        Ok(())
    }
}

/// Uname command
pub struct UnameCommand;

impl Command for UnameCommand {
    fn name(&self) -> &str {
        "uname"
    }

    fn help(&self) -> &str {
        "Print system name (use -a for extended output)"
    }

    fn execute(&self, args: &[&str], _registry: &CommandRegistry) -> Result<(), &str> {
        if args.len() > 1 {
            return Err("Usage: uname [-a]");
        }

        match args.first().copied() {
            None => crate::VgaText::println(OS_UNAME),
            Some("-a") => crate::VgaText::println(&alloc::format!(
                "{} {} {} {}",
                OS_UNAME,
                OS_VERSION,
                OS_ARCH,
                OS_LANGUAGE
            )),
            _ => return Err("Usage: uname [-a]"),
        }

        Ok(())
    }
}

/// Free command
pub struct FreeCommand;

impl Command for FreeCommand {
    fn name(&self) -> &str {
        "free"
    }

    fn help(&self) -> &str {
        "Display physical and virtual memory usage"
    }

    fn execute(&self, args: &[&str], _registry: &CommandRegistry) -> Result<(), &str> {
        if !args.is_empty() {
            return Err("Usage: free");
        }

        colors::print_info("Memory Usage");
        crate::VgaText::println("");

        let total_memory = alloy_kernel_hal::mem::total_memory();
        let available_memory = alloy_kernel_hal::mem::available_memory();
        let used_memory = total_memory.saturating_sub(available_memory);
        let heap_size = unsafe { crate::ffi::vmm_get_heap_size() };
        let allocated_pages = unsafe { crate::ffi::vmm_get_allocated_pages() };

        crate::VgaText::println("Physical:");
        print_size_line("  Total: ", total_memory);
        print_size_line("  Used:  ", used_memory);
        print_size_line("  Free:  ", available_memory);

        crate::VgaText::println("");
        crate::VgaText::println("Virtual Heap:");
        print_size_line("  Mapped bytes: ", heap_size as u64);
        print_u32_decimal_line("  Alloc pages:  ", allocated_pages);

        Ok(())
    }
}

/// Ticks command
pub struct TicksCommand;

impl Command for TicksCommand {
    fn name(&self) -> &str {
        "ticks"
    }

    fn help(&self) -> &str {
        "Display PIT tick count and timer configuration"
    }

    fn execute(&self, args: &[&str], _registry: &CommandRegistry) -> Result<(), &str> {
        if !args.is_empty() {
            return Err("Usage: ticks");
        }

        colors::print_info("Timer Statistics");
        crate::VgaText::println("");

        let tick_count = unsafe { crate::SystemTimer::ticks() };
        let uptime_ms = unsafe { crate::SystemTimer::uptime_ms() };
        let frequency_hz = unsafe { crate::SystemTimer::frequency() };

        print_u64_decimal_line("Tick count:     ", tick_count);
        print_u64_decimal_line("Uptime (ms):    ", uptime_ms);
        print_u32_decimal_line("Frequency (Hz): ", frequency_hz);

        if let Some(avg) = uptime_ms.checked_div(tick_count) {
            print_u64_decimal_line("Avg ms/tick:    ", avg);
        }

        Ok(())
    }
}

/// Memory info command
pub struct MeminfoCommand;

impl Command for MeminfoCommand {
    fn name(&self) -> &str {
        "meminfo"
    }
    
    fn help(&self) -> &str {
        "Display memory allocation statistics"
    }
    
    fn execute(&self, _args: &[&str], _registry: &CommandRegistry) -> Result<(), &str> {
        use crate::utils::format;
        
        colors::print_info("Memory Statistics");
        crate::VgaText::println("");
        
        // Get PMM statistics
        {
            let total_frames = alloy_kernel_hal::mem::total_frames();
            let used_frames = alloy_kernel_hal::mem::used_frames();
            let free_frames = total_frames - used_frames;
            let total_memory = alloy_kernel_hal::mem::total_memory();
            let available_memory = alloy_kernel_hal::mem::available_memory();
            
            crate::VgaText::println("Physical Memory Manager:");
            
            // Total memory
            let (val_buf, unit_buf) = format::format_bytes(total_memory);
            let val_start = format::trim_leading_spaces(&val_buf);
            crate::VgaText::print_bytes(b"  Total memory:     ");
            crate::terminal::print_cstr(&val_buf[val_start..]);
            crate::VgaText::print_bytes(b" ");
            crate::terminal::println_cstr(&unit_buf[0..]);
            
            // Available memory
            let (val_buf, unit_buf) = format::format_bytes(available_memory);
            let val_start = format::trim_leading_spaces(&val_buf);
            crate::VgaText::print_bytes(b"  Available memory: ");
            crate::terminal::print_cstr(&val_buf[val_start..]);
            crate::VgaText::print_bytes(b" ");
            crate::terminal::println_cstr(&unit_buf[0..]);
            
            // Frame statistics
            let total_frames_str = format::u32_to_decimal(total_frames);
            let used_frames_str = format::u32_to_decimal(used_frames);
            let free_frames_str = format::u32_to_decimal(free_frames);
            
            crate::VgaText::print_bytes(b"  Total frames:     ");
            crate::terminal::println_cstr(&total_frames_str[format::trim_leading_spaces(&total_frames_str)..]);
            
            crate::VgaText::print_bytes(b"  Used frames:      ");
            crate::terminal::println_cstr(&used_frames_str[format::trim_leading_spaces(&used_frames_str)..]);
            
            crate::VgaText::print_bytes(b"  Free frames:      ");
            crate::terminal::println_cstr(&free_frames_str[format::trim_leading_spaces(&free_frames_str)..]);
        }
        
        crate::VgaText::println("");
        
        // Get VMM statistics
        unsafe {
            let heap_start = crate::ffi::vmm_get_heap_start();
            let heap_size = crate::ffi::vmm_get_heap_size();
            let allocated_pages = crate::ffi::vmm_get_allocated_pages();
            
            crate::VgaText::println("Virtual Memory Manager:");
            
            // Heap start address
            let heap_start_hex = format::u32_to_hex(heap_start as u32);
            crate::VgaText::print_bytes(b"  Heap start:       ");
            crate::terminal::println_cstr(&heap_start_hex[0..]);
            
            // Heap size
            let (val_buf, unit_buf) = format::format_bytes(heap_size as u64);
            let val_start = format::trim_leading_spaces(&val_buf);
            crate::VgaText::print_bytes(b"  Heap size:        ");
            crate::terminal::print_cstr(&val_buf[val_start..]);
            crate::VgaText::print_bytes(b" ");
            crate::terminal::println_cstr(&unit_buf[0..]);
            
            // Allocated pages
            let allocated_pages_str = format::u32_to_decimal(allocated_pages);
            crate::VgaText::print_bytes(b"  Allocated pages:  ");
            crate::terminal::println_cstr(&allocated_pages_str[format::trim_leading_spaces(&allocated_pages_str)..]);
        }
        
        crate::VgaText::println("");
        
        // Get allocator statistics
        let (slab_stats, heap_stats) = crate::allocator::get_stats();
        
        crate::VgaText::println("Rust Allocators:");
        
        // Slab allocator
        let slab_alloc_str = format::u32_to_decimal(slab_stats.0 as u32);
        let slab_freed_str = format::u32_to_decimal(slab_stats.1 as u32);
        
        crate::VgaText::print("  Slab allocated:   ");
        unsafe {
            crate::terminal::println_cstr(&slab_alloc_str[format::trim_leading_spaces(&slab_alloc_str)..]);
        }
        
        crate::VgaText::print("  Slab freed:       ");
        unsafe {
            crate::terminal::println_cstr(&slab_freed_str[format::trim_leading_spaces(&slab_freed_str)..]);
        }
        
        // Heap allocator
        let heap_alloc_str = format::u32_to_decimal(heap_stats.0 as u32);
        let heap_freed_str = format::u32_to_decimal(heap_stats.1 as u32);
        
        crate::VgaText::print("  Heap allocated:   ");
        unsafe {
            crate::terminal::println_cstr(&heap_alloc_str[format::trim_leading_spaces(&heap_alloc_str)..]);
        }
        
        crate::VgaText::print("  Heap freed:       ");
        unsafe {
            crate::terminal::println_cstr(&heap_freed_str[format::trim_leading_spaces(&heap_freed_str)..]);
        }
        
        Ok(())
    }
}

/// CPU info command
pub struct CpuInfoCommand;

// CPU feature flag constants (matching cpu.h)
const CPU_FEATURE_FPU: u32     = 1 << 0;
const CPU_FEATURE_MMX: u32     = 1 << 23;
const CPU_FEATURE_SSE: u32     = 1 << 25;
const CPU_FEATURE_SSE2: u32    = 1 << 26;
const CPU_FEATURE_APIC: u32    = 1 << 9;
const CPU_FEATURE_TSC: u32     = 1 << 4;
const CPU_FEATURE_PAE: u32     = 1 << 6;

impl Command for CpuInfoCommand {
    fn name(&self) -> &str {
        "cpuinfo"
    }
    
    fn help(&self) -> &str {
        "Display CPU information and features"
    }
    
    fn execute(&self, _args: &[&str], _registry: &CommandRegistry) -> Result<(), &str> {
        use crate::utils::format;
        
        colors::print_info("CPU Information");
        crate::VgaText::println("");
        
        unsafe {
            // Get CPU vendor
            let mut vendor = [0u8; 13];
            crate::ffi::cpu_get_vendor_ffi(vendor.as_mut_ptr());
            crate::VgaText::print_bytes(b"Vendor:   ");
            crate::terminal::println_cstr(&vendor[..]);
            
            // Get model info
            let mut family: u32 = 0;
            let mut model: u32 = 0;
            let mut stepping: u32 = 0;
            crate::ffi::cpu_get_model_info_ffi(&mut family, &mut model, &mut stepping);
            
            let family_str = format::u32_to_decimal(family);
            let model_str = format::u32_to_decimal(model);
            let stepping_str = format::u32_to_decimal(stepping);
            
            crate::VgaText::print_bytes(b"Family:   ");
            crate::terminal::println_cstr(&family_str[format::trim_leading_spaces(&family_str)..]);
            
            crate::VgaText::print_bytes(b"Model:    ");
            crate::terminal::println_cstr(&model_str[format::trim_leading_spaces(&model_str)..]);
            
            crate::VgaText::print_bytes(b"Stepping: ");
            crate::terminal::println_cstr(&stepping_str[format::trim_leading_spaces(&stepping_str)..]);
            
            // Get features
            let features = crate::ffi::cpu_get_features_ffi();
            
            crate::VgaText::println_bytes(b"\nFeatures:");
            
            if features & CPU_FEATURE_FPU != 0 {
                crate::VgaText::println_bytes(b"  [x] FPU   - Floating Point Unit");
            }
            if features & CPU_FEATURE_TSC != 0 {
                crate::VgaText::println_bytes(b"  [x] TSC   - Time Stamp Counter");
            }
            if features & CPU_FEATURE_PAE != 0 {
                crate::VgaText::println_bytes(b"  [x] PAE   - Physical Address Extension");
            }
            if features & CPU_FEATURE_APIC != 0 {
                crate::VgaText::println_bytes(b"  [x] APIC  - Advanced Programmable Interrupt Controller");
            }
            if features & CPU_FEATURE_MMX != 0 {
                crate::VgaText::println_bytes(b"  [x] MMX   - MMX Instructions");
            }
            if features & CPU_FEATURE_SSE != 0 {
                crate::VgaText::println_bytes(b"  [x] SSE   - Streaming SIMD Extensions");
            }
            if features & CPU_FEATURE_SSE2 != 0 {
                crate::VgaText::println_bytes(b"  [x] SSE2  - Streaming SIMD Extensions 2");
            }
        }
        
        Ok(())
    }
}

/// Uptime command
pub struct UptimeCommand;

impl Command for UptimeCommand {
    fn name(&self) -> &str {
        "uptime"
    }
    
    fn help(&self) -> &str {
        "Display system uptime"
    }
    
    fn execute(&self, _args: &[&str], _registry: &CommandRegistry) -> Result<(), &str> {
        unsafe {
            let uptime_ms = crate::SystemTimer::uptime_ms();

            colors::print_info("System Uptime");
            print_uptime_value(uptime_ms);
        }
        
        Ok(())
    }
}
