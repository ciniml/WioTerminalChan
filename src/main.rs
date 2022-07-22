#![no_std]
#![no_main]

use bsp::hal;
use bsp::hal::pwm::Channel;
use bsp::pac;

use bsp::Display;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::image::Image;
use embedded_graphics::image::ImageRawLE;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::Point;
use embedded_graphics::prelude::Primitive;
use embedded_graphics::prelude::RgbColor;
use embedded_graphics::prelude::Size;
use embedded_graphics::primitives::PrimitiveStyleBuilder;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::Drawable;
use wio_terminal as bsp;

use bsp::entry;
use bsp::prelude::*;
use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::pwm::{TCC3Pinout, Tcc3Pwm};
use hal::trng::Trng;
use pac::{CorePeripherals, Peripherals};

#[cfg(not(feature = "semihosting"))]
use panic_halt as _;
#[cfg(feature = "semihosting")]
use panic_semihosting as _;

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

    let mut pwm = Tcc3Pwm::new(
        &pwm_clock,
        5.khz(),
        peripherals.TCC3,
        TCC3Pinout::Pb12(pins.gpclk1),
        &mut peripherals.MCLK,
    );
    pwm.set_period(50.hz()); // 20[ms]
    let servo_duty_range = pwm.get_max_duty() / 10; // 0.5[ms] ~ 2.5[ms]
    let servo_duty_offset = pwm.get_max_duty() / 40; // 0.5[ms]

    let tilt_lower = servo_duty_range / 2 - servo_duty_range / 10 + servo_duty_offset;
    let tilt_upper = servo_duty_range / 5 + servo_duty_offset;
    let pan_left = servo_duty_range / 5 + servo_duty_offset;
    let pan_right = servo_duty_range * 4 / 5 + servo_duty_offset;
    let pan_center = servo_duty_range / 2 + servo_duty_offset;

    pwm.set_duty(Channel::_0, tilt_lower);
    pwm.set_duty(Channel::_1, pan_center);
    pwm.enable(Channel::_0);
    pwm.enable(Channel::_1);
    TCC3Pinout::Pb13(pins.gpclk2);

    let (mut display, _backlight) = Display {
        miso: pins.lcd_miso,
        mosi: pins.lcd_mosi,
        sck: pins.lcd_sck,
        cs: pins.lcd_cs,
        dc: pins.lcd_dc,
        reset: pins.lcd_reset,
        backlight: pins.lcd_backlight,
    }
    .init(
        &mut clocks,
        peripherals.SERCOM7,
        &mut peripherals.MCLK,
        58.mhz(),
        &mut delay,
    )
    .unwrap();

    // Clear the screen
    let fill = Rectangle::new(Point::new(0, 0), Size::new(320, 240)).into_styled(
        PrimitiveStyleBuilder::new()
            .fill_color(Rgb565::WHITE)
            .build(),
    );
    fill.draw(&mut display).ok();

    // Initialize wio terminal chan
    let wio_terminal_chan = unsafe {
        WIO_TERMINAL_CHAN = Some(WioTerminalChan::new());
        WIO_TERMINAL_CHAN.as_mut().unwrap()
    };

    let trng = Trng::new(&mut peripherals.MCLK, peripherals.TRNG);

    let mut user_led = pins.user_led.into_push_pull_output();
    user_led.set_low().unwrap();

    let mut animation_counter = 0;
    loop {
        let new_animation_counter = match animation_counter {
            0 => {
                wio_terminal_chan
                    .draw_face(EyeState::Opened, EyeState::Opened, &mut display)
                    .ok();
                None
            }
            1..=59 => None,
            60 => {
                wio_terminal_chan
                    .draw_face(EyeState::HalfOpened, EyeState::HalfOpened, &mut display)
                    .ok();
                None
            }
            61 => {
                wio_terminal_chan
                    .draw_face(EyeState::AlmostClosed, EyeState::AlmostClosed, &mut display)
                    .ok();
                None
            }
            62 => {
                wio_terminal_chan
                    .draw_face(EyeState::Closed, EyeState::Closed, &mut display)
                    .ok();
                None
            }
            63 => {
                wio_terminal_chan
                    .draw_face(EyeState::AlmostClosed, EyeState::AlmostClosed, &mut display)
                    .ok();
                None
            }
            64 => {
                wio_terminal_chan
                    .draw_face(EyeState::HalfOpened, EyeState::HalfOpened, &mut display)
                    .ok();
                None
            }
            65..=70 => {
                wio_terminal_chan
                    .draw_face(EyeState::Opened, EyeState::Opened, &mut display)
                    .ok();
                None
            }
            71 => {
                let tilt_duty =
                    (((tilt_lower - tilt_upper) as f32 * random_f32(&trng)) as u32) + tilt_upper;
                let pan_duty =
                    (((pan_right - pan_left) as f32 * random_f32(&trng)) as u32) + pan_left;
                pwm.set_duty(Channel::_0, tilt_duty);
                pwm.set_duty(Channel::_1, pan_duty);
                user_led.toggle().ok();
                None
            }
            72..=79 => None,
            80 => Some(0),
            _ => Some(0),
        };
        animation_counter = match new_animation_counter {
            Some(value) => value,
            None => animation_counter + 1,
        };
        delay.delay_ms(50u16);
    }
}

static EYE_ALMOAST_CLOSED_BYTES: &[u8] = include_bytes!("asserts/eye_almost_closed.raw");
static EYE_CLOSED_BYTES: &[u8] = include_bytes!("asserts/eye_closed.raw");
static EYE_FULLY_OPENED_BYTES: &[u8] = include_bytes!("asserts/eye_fully_opened.raw");
static EYE_HALF_OPENED_BYTES: &[u8] = include_bytes!("asserts/eye_half_opened.raw");
static MOUTH_BYTES: &[u8] = include_bytes!("asserts/mouth.raw");

const EYE_OFFSET_X: u32 = 60;
const EYE_OFFSET_Y: u32 = 70;
const EYE_RADIUS: u32 = 25;
const MOUTH_RADIUS: u32 = 35;
const MOUTH_OFFSET: u32 = 70;

enum EyeState {
    Opened,
    HalfOpened,
    AlmostClosed,
    Closed,
}

static mut WIO_TERMINAL_CHAN: Option<WioTerminalChan> = None;

struct WioTerminalChan {
    eye_sprites: [ImageRawLE<'static, Rgb565>; 4],
    mouth_sprite: ImageRawLE<'static, Rgb565>,
}

impl WioTerminalChan {
    fn new() -> Self {
        let eye_sprites = [
            ImageRawLE::new(EYE_FULLY_OPENED_BYTES, EYE_RADIUS * 2 + 1),
            ImageRawLE::new(EYE_HALF_OPENED_BYTES, EYE_RADIUS * 2 + 1),
            ImageRawLE::new(EYE_ALMOAST_CLOSED_BYTES, EYE_RADIUS * 2 + 1),
            ImageRawLE::new(EYE_CLOSED_BYTES, EYE_RADIUS * 2 + 1),
        ];
        let mouth_sprite = ImageRawLE::new(MOUTH_BYTES, MOUTH_RADIUS * 2);
        Self {
            eye_sprites,
            mouth_sprite,
        }
    }

    fn draw_eye<Target: DrawTarget<Color = Rgb565>>(
        &self,
        offset: Point,
        state: EyeState,
        target: &mut Target,
    ) -> Result<(), Target::Error> {
        let sprite = match state {
            EyeState::Opened => &self.eye_sprites[0],
            EyeState::HalfOpened => &self.eye_sprites[1],
            EyeState::AlmostClosed => &self.eye_sprites[2],
            EyeState::Closed => &self.eye_sprites[3],
        };
        let image = Image::new(
            sprite,
            offset - Point::new(EYE_RADIUS as i32, EYE_RADIUS as i32),
        );
        image.draw(target)?;
        Ok(())
    }

    fn draw_mouth<Target: DrawTarget<Color = Rgb565>>(
        &mut self,
        offset: Point,
        target: &mut Target,
    ) -> Result<(), Target::Error> {
        let sprite = &self.mouth_sprite;
        let image = Image::new(
            sprite,
            offset - Point::new(MOUTH_RADIUS as i32, MOUTH_RADIUS as i32),
        );
        image.draw(target)?;
        Ok(())
    }

    fn draw_face<Target: DrawTarget<Color = Rgb565>>(
        &mut self,
        left_eye_state: EyeState,
        right_eye_state: EyeState,
        target: &mut Target,
    ) -> Result<(), Target::Error> {
        let screen_size = target.bounding_box().size;
        self.draw_eye(
            Point::new(EYE_OFFSET_X as i32, EYE_OFFSET_Y as i32),
            left_eye_state,
            target,
        )?;
        self.draw_eye(
            Point::new(
                (screen_size.width - EYE_OFFSET_X as u32) as i32,
                EYE_OFFSET_Y as i32,
            ),
            right_eye_state,
            target,
        )?;
        self.draw_mouth(
            Point::new(
                (screen_size.width as i32) / 2,
                (screen_size.height - MOUTH_OFFSET as u32) as i32,
            ),
            target,
        )?;
        Ok(())
    }
}
