
// rust demo of dcc train protocol

#![no_std]
#![no_main]

mod dcc;

use core::convert::TryInto;

use cortex_m_rt::entry;
use embedded_time::rate::*;
use panic_halt as _;
use rp_pico::hal::prelude::*;
use rp_pico::hal::pac;
use rp_pico::hal;

// found on stack overflow
fn as_u32_be_lower(array: &[u8; 8]) -> u32 {
    ((array[0] as u32) << 24) +
    ((array[1] as u32) << 16) +
    ((array[2] as u32) <<  8) +
    ((array[3] as u32) <<  0)
}
fn as_u32_be_upper(array: &[u8; 8]) -> u32 {
    ((array[4] as u32) << 24) +
    ((array[5] as u32) << 16) +
    ((array[6] as u32) <<  8) +
    ((array[7] as u32) <<  0)
}

fn assemble_packet(address: u8, data: u8) -> (u32, u32){
    let checksum = address ^ data;
    let mut packet:[u8; 8] = [0xff,0xfe,address,0x00,0x00,0x00,0x00,0x00];
    packet[3] |= (data >> 1) << 0;
    packet[4] |= ((data << 6) << 6) | (0b0 << 5) | ((checksum >> 2) << 0);
    packet[5] |= (checksum << 6) << 0;
    packet[5] |= 0x20; // end of packet bit
    let w1 = as_u32_be_lower(&packet);
    let w2 = as_u32_be_upper(&packet); 
    (w1, w2)
}

#[entry]
fn main() -> ! {
    // hardware setup
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);
    let clocks = hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,).ok().unwrap();
    let sio = hal::Sio::new(pac.SIO);
    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,);
    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().integer());
    let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);    
    let mut tx = dcc::init(
        pins.gpio15.into_mode(), 
        &mut pio, 
        sm0,
        clocks.peripheral_clock.freq(),);
    
    let address = 50u8;
    let forward = 0x76u8;
    let reverse = 0x56u8;

    loop {
        
       // forward packet
       let (w1, w2) = assemble_packet(address, forward);
       tx.write(w1);
       tx.write(w2);
       tx.write(w1);
       tx.write(w2);     
       delay.delay_ms(3000);
       // delay should be idle packet every 5 millisecond

       // reverse packet
       let (w1, w2) = assemble_packet(address, reverse);
       tx.write(w1);
       tx.write(w2);
       tx.write(w1);
       tx.write(w2);        
       delay.delay_ms(3000);
    }}
