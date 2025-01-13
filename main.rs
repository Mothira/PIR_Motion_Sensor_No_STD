#![no_std]
#![no_main]

use core::cell::{Cell, RefCell};
use critical_section::Mutex;
use esp_backtrace as _;
use esp_println::println;
use esp_hal::
{
    delay::Delay,
    prelude::*, 
    gpio::{Event, Input, Pull, Io, Level, Output}
};

// Create a Global Variable for a GPIO Peripheral to pass around between threads
static PIR_PIN: Mutex<RefCell<Option<Input>>> = Mutex::new(RefCell::new(None));
// Create a GLobal Variable for a FLAG to pass around between threads
// Option is not used since the variable is being directly initialized
// Using Cell instead of RefCell because it's a bool so we can take a copy instead of a reference
static PIR_FLAG: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));

#[handler]
fn motion_interrupt()
{
    critical_section::with(|cs| 
    {
        // Obtain access to Global Input Peripheral
        // Clear Interrupt Pending Flag
        PIR_PIN.borrow_ref_mut(cs).as_mut().unwrap().clear_interrupt();

        // Assert G_FLAG indicating a press button happened
        PIR_FLAG.borrow(cs).set(true);
    });
}


#[entry]
fn main() -> ! 
{
    // Take Peripherals
    let peripherals = esp_hal::init(esp_hal::Config::default());
    // Declare Delay
    let mut delay = Delay::new();

    // Create IO driver
    let mut io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    // Register interrupt handler
    io.set_interrupt_handler(motion_interrupt);

    // Configure Pin Direction
    let mut motion_pin = Input::new(io.pins.gpio0, Pull::Up);

    // Interrupt Configuration
    // Configure input to trigger an interrupt
    motion_pin.listen(Event::HighLevel);
    // Move pin to global context
    critical_section::with(|cs| { PIR_PIN.borrow_ref_mut(cs).replace(motion_pin); });

    // Configure output pin
    let mut led_pin = Output::new(io.pins.gpio9, Level::High);

    // Initialize output to start as low
    led_pin.set_low();

    loop 
    {
        // This is where the motion is repeatedly checked
        critical_section::with(|cs| 
        {
            // If flag is set (motion detected i.e. gpio0 is high)
            if PIR_FLAG.borrow(cs).get()
            {
                // Set output pin high and delay for 5s
                led_pin.set_high();
                delay.delay_millis(5000_u32);
                // Clear interrupt
                PIR_FLAG.borrow(cs).set(false);
            }
        });
        // Set output pin low and delay for 1.2s
        led_pin.set_low();
        delay.delay_millis(1200_u32);
    }
}
