//! Serial port abstraction

#[cfg(feature = "x86_64")]
use crate::io::{IoPort, X86IoPort};

#[cfg(feature = "aarch64")]
use crate::io::{Mmio, DefaultMmio};

/// Serial port trait
pub trait SerialPort {
    /// Initialize the serial port
    fn init(&mut self, port: u16, baud: u32);

    /// Check if the serial port is ready to transmit
    fn is_transmit_ready(&self) -> bool;

    /// Check if there is data to receive
    fn has_data(&self) -> bool;

    /// Send a byte
    fn send_byte(&mut self, byte: u8);

    /// Receive a byte
    fn receive_byte(&mut self) -> u8;

    /// Send a null-terminated string
    fn print(&mut self, s: &str);
}

/// 16550 UART serial port (common on x86)
#[cfg(feature = "x86_64")]
pub struct Uart16550 {
    pub port: u16,
    pub initialized: bool,
}

#[cfg(feature = "x86_64")]
impl Uart16550 {
    pub const fn new() -> Self {
        Self {
            port: 0x3F8, // COM1
            initialized: false,
        }
    }

    fn write_register(&self, offset: u16, value: u8) {
        unsafe {
            <X86IoPort as IoPort>::outb(self.port + offset, value);
        }
    }

    fn read_register(&self, offset: u16) -> u8 {
        unsafe { <X86IoPort as IoPort>::inb(self.port + offset) }
    }
}

#[cfg(feature = "x86_64")]
impl SerialPort for Uart16550 {
    fn init(&mut self, port: u16, baud: u32) {
        self.port = port;

        // Disable interrupts
        self.write_register(1, 0x00);

        // Enable DLAB (set baud rate divisor)
        self.write_register(3, 0x80);

        // Set divisor for baud rate
        let divisor = 115200 / baud;
        self.write_register(0, (divisor & 0xFF) as u8);
        self.write_register(1, ((divisor >> 8) & 0xFF) as u8);

        // Disable DLAB, set data bits (8), no parity, 1 stop bit
        self.write_register(3, 0x03);

        // Enable FIFO, clear them, with 14-byte threshold
        self.write_register(2, 0xC7);

        // IRQs enabled, RTS/DSR set
        self.write_register(4, 0x0B);

        // Set loopback mode, test serial chip
        self.write_register(4, 0x1E);

        // Check if serial is faulty
        self.write_register(0, 0xAE);

        if self.read_register(0) != 0xAE {
            self.initialized = false;
            return;
        }

        // Set in normal operation mode
        self.write_register(4, 0x0F);

        self.initialized = true;
    }

    fn is_transmit_ready(&self) -> bool {
        self.read_register(5) & 0x20 != 0
    }

    fn has_data(&self) -> bool {
        self.read_register(5) & 0x01 != 0
    }

    fn send_byte(&mut self, byte: u8) {
        while !self.is_transmit_ready() {}
        self.write_register(0, byte);
    }

    fn receive_byte(&mut self) -> u8 {
        while !self.has_data() {}
        self.read_register(0)
    }

    fn print(&mut self, s: &str) {
        for byte in s.bytes() {
            self.send_byte(byte);
        }
    }
}

/// ARM PL011 UART (common on ARM platforms like Raspberry Pi)
#[cfg(feature = "aarch64")]
pub struct Pl011Uart {
    pub base_address: u64,
    pub initialized: bool,
}

#[cfg(feature = "aarch64")]
impl Pl011Uart {
    pub const fn new(base_address: u64) -> Self {
        Self {
            base_address,
            initialized: false,
        }
    }

    /// Default for QEMU virt machine (PL011 at 0x09000000)
    pub const fn qemu_virt() -> Self {
        Self {
            base_address: 0x0900_0000,
            initialized: false,
        }
    }

    fn write_register(&self, offset: u32, value: u32) {
        unsafe {
            let addr = self.base_address as usize + offset as usize;
            DefaultMmio::write32(addr, value);
        }
    }

    fn read_register(&self, offset: u32) -> u32 {
        unsafe {
            let addr = self.base_address as usize + offset as usize;
            DefaultMmio::read32(addr)
        }
    }
}

#[cfg(feature = "aarch64")]
impl SerialPort for Pl011Uart {
    fn init(&mut self, _port: u16, baud: u32) {
        // Disable UART
        self.write_register(0x30, 0); // UARTCR

        // Set baud rate (3 MHz clock typical for PL011)
        let uart_clk = 3000000;
        let divisor = uart_clk / (16 * baud);
        let ibrd = divisor;
        let fbrd = ((divisor % 16) * 4 + 2) / 4;

        self.write_register(0x24, ibrd as u32); // UARTIBRD
        self.write_register(0x28, fbrd as u32); // UARTFBRD

        // Set line control: 8 bits, no parity, 1 stop bit, FIFO enabled
        self.write_register(0x2C, 0x70); // UARTLCR_H

        // Enable UART, TXE, RXE
        self.write_register(0x30, 0x301); // UARTCR

        self.initialized = true;
    }

    fn is_transmit_ready(&self) -> bool {
        // Check TXFF (Transmit FIFO Full) bit
        self.read_register(0x18) & (1 << 5) == 0 // UARTFR
    }

    fn has_data(&self) -> bool {
        // Check RXFE (Receive FIFO Empty) bit
        self.read_register(0x18) & (1 << 4) == 0 // UARTFR
    }

    fn send_byte(&mut self, byte: u8) {
        while !self.is_transmit_ready() {}
        self.write_register(0x00, byte as u32); // UARTDR
    }

    fn receive_byte(&mut self) -> u8 {
        while !self.has_data() {}
        self.read_register(0x00) as u8 // UARTDR
    }

    fn print(&mut self, s: &str) {
        for byte in s.bytes() {
            self.send_byte(byte);
        }
    }
}
