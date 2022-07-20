#![no_std]
#![no_main]

use bsp::hal;
use bsp::hal::pwm::Channel;
use bsp::hal::trng;
use bsp::pac;
use wio_terminal as bsp;

use bsp::entry;
use bsp::prelude::*;
use hal::clock::{GenericClockController};
use hal::delay::Delay;
use hal::pwm::{Tcc3Pwm, TCC3Pinout};
use hal::trng::Trng;
use pac::{CorePeripherals, Peripherals};

extern crate panic_halt;

fn random_f32(trng: &Trng) -> f32 {
    trng.random_u32() as f32 / 4294967296f32
}

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let core = CorePeripherals::take().unwrap();

    let mut clocks = GenericClockController::with_external_32kosc(
        peripherals.GCLK,
        &mut peripherals.MCLK,
        &mut peripherals.OSC32KCTRL,
        &mut peripherals.OSCCTRL,
        &mut peripherals.NVMCTRL,
    );
    let mut delay = Delay::new(core.SYST, &mut clocks);

    let pins = bsp::Pins::new(peripherals.PORT);
    

    let gclk0 = clocks.gclk0();
    let pwm_clock = clocks.tcc2_tcc3(&gclk0).unwrap();
    
    let mut pwm = Tcc3Pwm::new(&pwm_clock, 5.khz(), peripherals.TCC3, TCC3Pinout::Pb12(pins.gpclk1), &mut peripherals.MCLK);
    pwm.set_period(50.hz());    // 20[ms]
    let servo_duty_range = pwm.get_max_duty()/10;   // 0.5[ms] ~ 2.5[ms]
    let servo_duty_offset = pwm.get_max_duty()/40;  // 0.5[ms]
    
    let tilt_lower = servo_duty_range / 2 - servo_duty_range / 20 + servo_duty_offset;
    let tilt_upper = servo_duty_range / 5 + servo_duty_offset;
    let pan_left = servo_duty_range / 5 + servo_duty_offset;
    let pan_right = servo_duty_range * 4 / 5 + servo_duty_offset;
    let pan_center = servo_duty_range / 2 + servo_duty_offset;

    pwm.set_duty(Channel::_0, tilt_lower);
    pwm.set_duty(Channel::_1, pan_center);
    pwm.enable(Channel::_0);
    pwm.enable(Channel::_1);
    TCC3Pinout::Pb13(pins.gpclk2);


    let trng = Trng::new(&mut peripherals.MCLK, peripherals.TRNG);

    let mut user_led = pins.user_led.into_push_pull_output();
    user_led.set_low().unwrap();

    loop {
        let tilt_duty = (((tilt_lower - tilt_upper) as f32 * random_f32(&trng)) as u32) + tilt_upper;
        let pan_duty = (((pan_right - pan_left) as f32 * random_f32(&trng)) as u32) + pan_left;
        pwm.set_duty(Channel::_0, tilt_duty);
        pwm.set_duty(Channel::_1, pan_duty);
        user_led.toggle().ok();
        delay.delay_ms(2000u16);
    }
}
