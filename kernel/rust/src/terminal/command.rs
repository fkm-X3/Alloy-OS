/// Command system for terminal
/// 
/// Defines the Command trait and provides a registry for command lookup

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use crate::terminal::colors;

/// Command trait for terminal commands
pub trait Command {
    /// Get command name
    fn name(&self) -> &str;
    
    /// Get command help text
    fn help(&self) -> &str;
    
    /// Execute the command with given arguments
    fn execute(&self, args: &[&str]) -> Result<(), &str>;
}

/// Command registry
pub struct CommandRegistry {
    commands: BTreeMap<String, Box<dyn Command>>,
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
            match cmd.execute(args) {
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
    pub fn get(&self, name: &str) -> Option<&Box<dyn Command>> {
        self.commands.get(name)
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
    
    fn execute(&self, args: &[&str]) -> Result<(), &str> {
        // TODO: Implement help display
        colors::print_info("Available commands:");
        crate::ffi::vga_println_str("  help     - Show this help message");
        crate::ffi::vga_println_str("  clear    - Clear the screen");
        crate::ffi::vga_println_str("  echo     - Print text to the screen");
        crate::ffi::vga_println_str("  version  - Show OS version information");
        crate::ffi::vga_println_str("  meminfo  - Display memory statistics");
        crate::ffi::vga_println_str("  cpuinfo  - Display CPU information");
        crate::ffi::vga_println_str("  uptime   - Display system uptime");
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
    
    fn execute(&self, _args: &[&str]) -> Result<(), &str> {
        unsafe {
            // Clear screen by printing 25 empty lines
            crate::ffi::vga_set_color(0, 0);
            for _ in 0..25 {
                crate::ffi::vga_println(b"\0".as_ptr());
            }
            crate::ffi::vga_set_color(7, 0);
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
    
    fn execute(&self, args: &[&str]) -> Result<(), &str> {
        if args.is_empty() {
            crate::ffi::vga_println_str("");
        } else {
            let text = args.join(" ");
            crate::ffi::vga_println_str(&text);
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
    
    fn execute(&self, _args: &[&str]) -> Result<(), &str> {
        colors::print_info("Alloy Operating System");
        crate::ffi::vga_println_str("Version: 0.7.0-dev (Phase 7)");
        crate::ffi::vga_println_str("Architecture: x86 (32-bit)");
        crate::ffi::vga_println_str("Language: C++ + Rust");
        crate::ffi::vga_println_str("");
        crate::ffi::vga_println_str("Features:");
        crate::ffi::vga_println_str("  [x] Multiboot2 boot");
        crate::ffi::vga_println_str("  [x] VGA text mode");
        crate::ffi::vga_println_str("  [x] PS/2 keyboard");
        crate::ffi::vga_println_str("  [x] Memory management");
        crate::ffi::vga_println_str("  [x] Rust integration");
        crate::ffi::vga_println_str("  [x] Terminal interface");
        crate::ffi::vga_println_str("  [x] Diagnostic commands");
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
    
    fn execute(&self, _args: &[&str]) -> Result<(), &str> {
        use crate::utils::format;
        
        colors::print_info("Memory Statistics");
        crate::ffi::vga_println_str("");
        
        // Get PMM statistics
        unsafe {
            let total_frames = crate::ffi::pmm_get_total_frames();
            let used_frames = crate::ffi::pmm_get_used_frames();
            let free_frames = total_frames - used_frames;
            let total_memory = crate::ffi::pmm_get_total_memory();
            let available_memory = crate::ffi::pmm_get_available_memory();
            
            crate::ffi::vga_println_str("Physical Memory Manager:");
            
            // Total memory
            let (val_buf, unit_buf) = format::format_bytes(total_memory);
            let val_start = format::trim_leading_spaces(&val_buf);
            crate::ffi::vga_print(b"  Total memory:     \0".as_ptr());
            crate::ffi::vga_print(&val_buf[val_start] as *const u8);
            crate::ffi::vga_print(b" \0".as_ptr());
            crate::ffi::vga_println(&unit_buf[0] as *const u8);
            
            // Available memory
            let (val_buf, unit_buf) = format::format_bytes(available_memory);
            let val_start = format::trim_leading_spaces(&val_buf);
            crate::ffi::vga_print(b"  Available memory: \0".as_ptr());
            crate::ffi::vga_print(&val_buf[val_start] as *const u8);
            crate::ffi::vga_print(b" \0".as_ptr());
            crate::ffi::vga_println(&unit_buf[0] as *const u8);
            
            // Frame statistics
            let total_frames_str = format::u32_to_decimal(total_frames);
            let used_frames_str = format::u32_to_decimal(used_frames);
            let free_frames_str = format::u32_to_decimal(free_frames);
            
            crate::ffi::vga_print(b"  Total frames:     \0".as_ptr());
            crate::ffi::vga_println(&total_frames_str[format::trim_leading_spaces(&total_frames_str)] as *const u8);
            
            crate::ffi::vga_print(b"  Used frames:      \0".as_ptr());
            crate::ffi::vga_println(&used_frames_str[format::trim_leading_spaces(&used_frames_str)] as *const u8);
            
            crate::ffi::vga_print(b"  Free frames:      \0".as_ptr());
            crate::ffi::vga_println(&free_frames_str[format::trim_leading_spaces(&free_frames_str)] as *const u8);
        }
        
        crate::ffi::vga_println_str("");
        
        // Get VMM statistics
        unsafe {
            let heap_start = crate::ffi::vmm_get_heap_start();
            let heap_size = crate::ffi::vmm_get_heap_size();
            let allocated_pages = crate::ffi::vmm_get_allocated_pages();
            
            crate::ffi::vga_println_str("Virtual Memory Manager:");
            
            // Heap start address
            let heap_start_hex = format::u32_to_hex(heap_start);
            crate::ffi::vga_print(b"  Heap start:       \0".as_ptr());
            crate::ffi::vga_println(&heap_start_hex[0] as *const u8);
            
            // Heap size
            let (val_buf, unit_buf) = format::format_bytes(heap_size as u64);
            let val_start = format::trim_leading_spaces(&val_buf);
            crate::ffi::vga_print(b"  Heap size:        \0".as_ptr());
            crate::ffi::vga_print(&val_buf[val_start] as *const u8);
            crate::ffi::vga_print(b" \0".as_ptr());
            crate::ffi::vga_println(&unit_buf[0] as *const u8);
            
            // Allocated pages
            let allocated_pages_str = format::u32_to_decimal(allocated_pages);
            crate::ffi::vga_print(b"  Allocated pages:  \0".as_ptr());
            crate::ffi::vga_println(&allocated_pages_str[format::trim_leading_spaces(&allocated_pages_str)] as *const u8);
        }
        
        crate::ffi::vga_println_str("");
        
        // Get allocator statistics
        let (slab_stats, heap_stats) = crate::allocator::get_stats();
        
        crate::ffi::vga_println_str("Rust Allocators:");
        
        // Slab allocator
        let slab_alloc_str = format::u32_to_decimal(slab_stats.0 as u32);
        let slab_freed_str = format::u32_to_decimal(slab_stats.1 as u32);
        
        crate::ffi::vga_print_str("  Slab allocated:   ");
        unsafe {
            crate::ffi::vga_println(&slab_alloc_str[format::trim_leading_spaces(&slab_alloc_str)] as *const u8);
        }
        
        crate::ffi::vga_print_str("  Slab freed:       ");
        unsafe {
            crate::ffi::vga_println(&slab_freed_str[format::trim_leading_spaces(&slab_freed_str)] as *const u8);
        }
        
        // Heap allocator
        let heap_alloc_str = format::u32_to_decimal(heap_stats.0 as u32);
        let heap_freed_str = format::u32_to_decimal(heap_stats.1 as u32);
        
        crate::ffi::vga_print_str("  Heap allocated:   ");
        unsafe {
            crate::ffi::vga_println(&heap_alloc_str[format::trim_leading_spaces(&heap_alloc_str)] as *const u8);
        }
        
        crate::ffi::vga_print_str("  Heap freed:       ");
        unsafe {
            crate::ffi::vga_println(&heap_freed_str[format::trim_leading_spaces(&heap_freed_str)] as *const u8);
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
    
    fn execute(&self, _args: &[&str]) -> Result<(), &str> {
        use crate::utils::format;
        
        colors::print_info("CPU Information");
        crate::ffi::vga_println_str("");
        
        unsafe {
            // Get CPU vendor
            let mut vendor = [0u8; 13];
            crate::ffi::cpu_get_vendor_ffi(vendor.as_mut_ptr());
            crate::ffi::vga_print(b"Vendor:   \0".as_ptr());
            crate::ffi::vga_println(vendor.as_ptr());
            
            // Get model info
            let mut family: u32 = 0;
            let mut model: u32 = 0;
            let mut stepping: u32 = 0;
            crate::ffi::cpu_get_model_info_ffi(&mut family, &mut model, &mut stepping);
            
            let family_str = format::u32_to_decimal(family);
            let model_str = format::u32_to_decimal(model);
            let stepping_str = format::u32_to_decimal(stepping);
            
            crate::ffi::vga_print(b"Family:   \0".as_ptr());
            crate::ffi::vga_println(&family_str[format::trim_leading_spaces(&family_str)] as *const u8);
            
            crate::ffi::vga_print(b"Model:    \0".as_ptr());
            crate::ffi::vga_println(&model_str[format::trim_leading_spaces(&model_str)] as *const u8);
            
            crate::ffi::vga_print(b"Stepping: \0".as_ptr());
            crate::ffi::vga_println(&stepping_str[format::trim_leading_spaces(&stepping_str)] as *const u8);
            
            // Get features
            let features = crate::ffi::cpu_get_features_ffi();
            
            crate::ffi::vga_println(b"\nFeatures:\0".as_ptr());
            
            if features & CPU_FEATURE_FPU != 0 {
                crate::ffi::vga_println(b"  [x] FPU   - Floating Point Unit\0".as_ptr());
            }
            if features & CPU_FEATURE_TSC != 0 {
                crate::ffi::vga_println(b"  [x] TSC   - Time Stamp Counter\0".as_ptr());
            }
            if features & CPU_FEATURE_PAE != 0 {
                crate::ffi::vga_println(b"  [x] PAE   - Physical Address Extension\0".as_ptr());
            }
            if features & CPU_FEATURE_APIC != 0 {
                crate::ffi::vga_println(b"  [x] APIC  - Advanced Programmable Interrupt Controller\0".as_ptr());
            }
            if features & CPU_FEATURE_MMX != 0 {
                crate::ffi::vga_println(b"  [x] MMX   - MMX Instructions\0".as_ptr());
            }
            if features & CPU_FEATURE_SSE != 0 {
                crate::ffi::vga_println(b"  [x] SSE   - Streaming SIMD Extensions\0".as_ptr());
            }
            if features & CPU_FEATURE_SSE2 != 0 {
                crate::ffi::vga_println(b"  [x] SSE2  - Streaming SIMD Extensions 2\0".as_ptr());
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
    
    fn execute(&self, _args: &[&str]) -> Result<(), &str> {
        use crate::utils::format;
        
        unsafe {
            let uptime_ms = crate::ffi::get_system_uptime_ms();
            
            // Convert to seconds, minutes, hours
            let total_seconds = uptime_ms / 1000;
            let seconds = total_seconds % 60;
            let total_minutes = total_seconds / 60;
            let minutes = total_minutes % 60;
            let total_hours = total_minutes / 60;
            let hours = total_hours % 24;
            let days = total_hours / 24;
            
            colors::print_info("System Uptime");
            
            if days > 0 {
                let days_str = format::u32_to_decimal(days as u32);
                crate::ffi::vga_print(&days_str[format::trim_leading_spaces(&days_str)] as *const u8);
                if days == 1 {
                    crate::ffi::vga_print(b" day, \0".as_ptr());
                } else {
                    crate::ffi::vga_print(b" days, \0".as_ptr());
                }
            }
            
            let hours_str = format::u32_to_decimal(hours as u32);
            let minutes_str = format::u32_to_decimal(minutes as u32);
            let seconds_str = format::u32_to_decimal(seconds as u32);
            
            crate::ffi::vga_print(&hours_str[format::trim_leading_spaces(&hours_str)] as *const u8);
            crate::ffi::vga_print(b":\0".as_ptr());
            
            // Pad minutes and seconds with leading zeros if needed
            if minutes < 10 {
                crate::ffi::vga_print(b"0\0".as_ptr());
            }
            crate::ffi::vga_print(&minutes_str[format::trim_leading_spaces(&minutes_str)] as *const u8);
            crate::ffi::vga_print(b":\0".as_ptr());
            
            if seconds < 10 {
                crate::ffi::vga_print(b"0\0".as_ptr());
            }
            crate::ffi::vga_println(&seconds_str[format::trim_leading_spaces(&seconds_str)] as *const u8);
        }
        
        Ok(())
    }
}
