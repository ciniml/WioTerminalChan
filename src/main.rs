#![no_std]
#![no_main]

use bsp::Display;
use bsp::hal;
use bsp::hal::pwm::Channel;
use bsp::hal::trng;
use bsp::pac;
use embedded_graphics::Drawable;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::image::Image;
use embedded_graphics::image::ImageRaw;
use embedded_graphics::image::ImageRawLE;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::Angle;
use embedded_graphics::prelude::AngleUnit;
use embedded_graphics::prelude::ImageDrawable;
use embedded_graphics::prelude::OriginDimensions;
use embedded_graphics::prelude::PixelColor;
use embedded_graphics::prelude::Point;
use embedded_graphics::prelude::Primitive;
use embedded_graphics::prelude::RgbColor;
use embedded_graphics::prelude::Size;
use embedded_graphics::primitives::Arc;
use embedded_graphics::primitives::Circle;
use embedded_graphics::primitives::Ellipse;
use embedded_graphics::primitives::PrimitiveStyle;
use embedded_graphics::primitives::PrimitiveStyleBuilder;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::primitives::StyledDrawable;
use wio_terminal as bsp;

use bsp::entry;
use bsp::prelude::*;
use hal::clock::{GenericClockController};
use hal::delay::Delay;
use hal::pwm::{Tcc3Pwm, TCC3Pinout};
use hal::trng::Trng;
use pac::{CorePeripherals, Peripherals};

#[cfg(not(feature="semihosting"))]
use panic_halt as _;
#[cfg(feature="semihosting")]
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


    let (mut display, _backlight) = Display {
        miso: pins.lcd_miso,
        mosi: pins.lcd_mosi,
        sck: pins.lcd_sck,
        cs: pins.lcd_cs,
        dc: pins.lcd_dc,
        reset: pins.lcd_reset,
        backlight: pins.lcd_backlight,
    }
    .init(&mut clocks, peripherals.SERCOM7, &mut peripherals.MCLK, 58.mhz(), &mut delay)
    .unwrap();

    // Clear the screen
    let fill = Rectangle::new(Point::new(0, 0), Size::new(320, 240)).into_styled(PrimitiveStyleBuilder::new().fill_color(Rgb565::WHITE).build());
    fill.draw(&mut display).ok();

    // Initialize wio terminal chan
    let wio_terminal_chan = unsafe {
        WIO_TERMINAL_CHAN = Some(WioTerminalChan::new());
        WIO_TERMINAL_CHAN.as_mut().unwrap()
    };
    wio_terminal_chan.udpate_eye_sprite(Rgb565::BLUE, Rgb565::WHITE).ok();
    wio_terminal_chan.update_mouth_sprite(Rgb565::BLUE, Rgb565::WHITE).ok();

    wio_terminal_chan.draw_face(EyeState::Opened, EyeState::Opened, &mut display).ok();

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


struct Sprite<C, const SIZE: usize> {
    buffer: [C; SIZE],
    offset: Point,
    size: Size,
}

impl<C: RgbColor, const SIZE: usize> Sprite<C, SIZE> {
    const fn new(width: u32) -> Self {
        Self { 
            buffer: [RgbColor::BLACK; SIZE],  
            offset: Point::zero(),
            size: Size::new(width, (SIZE as u32)/width ),
        }
    }

    fn set_offset(&mut self, offset: Point) {
        self.offset = offset;
    }
}

impl<C, const SIZE: usize> OriginDimensions for Sprite<C, SIZE> {
    fn size(&self) -> Size {
        self.size
    }
}

impl<C: PixelColor, const SIZE: usize> ImageDrawable for Sprite<C, SIZE> {
    type Color = C;

    fn draw<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Self::Color> {
        let area = Rectangle::new(self.offset, self.size);
        target.fill_contiguous(&area, self.buffer.iter().map(|c| *c))?;
        Ok(())
    }

    fn draw_sub_image<D>(&self, target: &mut D, area: &Rectangle) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Self::Color> {
        unimplemented!()
    }
}

impl<C: PixelColor, const SIZE: usize> DrawTarget for Sprite<C, SIZE> {
    type Color = C;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>> {
        for pixel in pixels {
            if pixel.0.x < 0 || self.size.width <= pixel.0.x as u32 || pixel.0.y < 0 || self.size.height <= pixel.0.y as u32 {
                continue;
            }
            let index = (pixel.0.x as u32) + (pixel.0.y as u32) * self.size.width;
            self.buffer[index as usize] = pixel.1;
        }
        Ok(())
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        // Get clamped top left and bottom right coordinate.
        let top_left = area.top_left;
        let bottom_right = if let Some(area) = area.bottom_right() { area } else { return Ok(()); };
        let mut colors = colors.into_iter();
        for y in top_left.y..=bottom_right.y {
            let line_base_index = if y >= 0 { y as u32 * self.size.width } else { 0 };
            for x in top_left.x..=bottom_right.x {
                let c = if let Some(c) = colors.next() { c } else { continue };
                if 0 <= x && (x as u32) < self.size.width && 0 <= y && (y as u32) < self.size.height {
                    let index = line_base_index + x as u32;
                    self.buffer[index as usize] = c;
                }
            }    
        }
        Ok(())
    }    
}

const EYE_OFFSET_X: usize = 60;
const EYE_OFFSET_Y: usize = 70;
const EYE_RADIUS: usize = 25;
const MOUTH_RADIUS: usize = 35;
const MOUTH_OFFSET: usize = 70;

const EYE_SIZE: Size = Size::new((EYE_RADIUS*2+1) as u32, (EYE_RADIUS*2+1) as u32);
const MOUTH_SIZE: Size = Size::new((MOUTH_RADIUS*2+1) as u32, (MOUTH_RADIUS*2+1) as u32);

const EYE_BUFFER_SIZE: usize = (EYE_RADIUS*2+1)*(EYE_RADIUS*2+1);
const MOUTH_BUFFER_SIZE: usize = (MOUTH_RADIUS*2+1)*(MOUTH_RADIUS*2+1);

enum EyeState {
    Opened,
    HalfOpened,
    AlmostClosed,
    Closed,
}

static mut WIO_TERMINAL_CHAN: Option<WioTerminalChan> = None;

struct WioTerminalChan {
    eye_sprites: [Sprite<Rgb565, EYE_BUFFER_SIZE>; 4],
    mouth_sprite: Sprite<Rgb565, MOUTH_BUFFER_SIZE>,
}

impl WioTerminalChan {
    const fn new() -> Self {
        let eye_sprites = [
            Sprite::new((EYE_RADIUS*2+1) as u32),
            Sprite::new((EYE_RADIUS*2+1) as u32),
            Sprite::new((EYE_RADIUS*2+1) as u32),
            Sprite::new((EYE_RADIUS*2+1) as u32),
        ];
        let mouth_sprite = Sprite::new((MOUTH_RADIUS*2+1) as u32);
        Self {
            eye_sprites,
            mouth_sprite,
        }
    }

    fn udpate_eye_sprite<C: PixelColor + Into<Rgb565>>(&mut self, eye_color: C, background_color: C) -> Result<(), core::convert::Infallible> {
        let background_style = PrimitiveStyleBuilder::new().fill_color(background_color.into()).build();
        let fill_eye_background = Rectangle::new(Point::zero(), self.eye_sprites[0].size).into_styled(background_style);
        for sprite in &mut self.eye_sprites {
            fill_eye_background.draw(sprite)?;
        }
        let background_style = fill_eye_background.style;

        let eye_style = PrimitiveStyleBuilder::new().fill_color(eye_color.into()).build();

        // Eye pattern 0: fully opened
        Circle::new(Point::zero(), (EYE_RADIUS*2+1) as u32).draw_styled(&eye_style, &mut self.eye_sprites[0])?;
        // Eye pattern 1: half opened
        Arc::new(Point::zero(), (EYE_RADIUS*2+1) as u32, Angle::from_degrees(0f32), Angle::from_degrees(180f32)).draw_styled(&eye_style, &mut self.eye_sprites[1])?;
        // Eye pattern 2: almost closed
        Arc::new(Point::zero(), (EYE_RADIUS*2+1) as u32, Angle::from_degrees(0f32), Angle::from_degrees(180f32)).draw_styled(&eye_style, &mut self.eye_sprites[2])?;
        Ellipse::new(Point::zero(), Size::new((EYE_RADIUS*2+1) as u32, ((EYE_RADIUS*2+1)*3/4) as u32)).draw_styled(&background_style, &mut self.eye_sprites[2])?;

        // Eye pattern 3: closed
        let closed_eye_style = PrimitiveStyleBuilder::new().stroke_color(eye_color.into()).stroke_width(1).build();
        Arc::new(Point::zero(), (EYE_RADIUS*2+1) as u32, Angle::from_degrees(0f32), Angle::from_degrees(180f32)).draw_styled(&closed_eye_style, &mut self.eye_sprites[3])?;
        
        Ok(())
    }

    fn update_mouth_sprite<C: PixelColor + Into<Rgb565>>(&mut self, mouth_color: C, background_color: C) -> Result<(), core::convert::Infallible> {
        let background_style = PrimitiveStyleBuilder::new().fill_color(background_color.into()).build();
        let mouth_style = PrimitiveStyleBuilder::new().fill_color(mouth_color.into()).build();
        Rectangle::new(Point::zero(), self.mouth_sprite.size).draw_styled(&background_style, &mut self.mouth_sprite)?;
        Arc::new(Point::zero(), (MOUTH_RADIUS*2+1) as u32, 0.0.deg(), -90.0.deg() ).draw_styled(&mouth_style, &mut self.mouth_sprite)?;
        Ok(())
    }

    fn draw_eye<Target: DrawTarget<Color = Rgb565>>(&mut self, offset: Point, state: EyeState, target: &mut Target) -> Result<(), Target::Error> {
        let sprite = match state {
            EyeState::Opened => &mut self.eye_sprites[0],
            EyeState::HalfOpened => &mut self.eye_sprites[1],
            EyeState::AlmostClosed => &mut self.eye_sprites[2],
            EyeState::Closed => &mut self.eye_sprites[3],
        };
        let prev_offset = sprite.offset;
        sprite.set_offset(offset - Point::new(EYE_RADIUS as i32, EYE_RADIUS as i32));
        sprite.draw(target)?;
        sprite.set_offset(prev_offset);
        Ok(())
    }

    fn draw_mouth<Target: DrawTarget<Color = Rgb565>>(&mut self, offset: Point, target: &mut Target) -> Result<(), Target::Error> {
        let sprite = &mut self.mouth_sprite;
        let prev_offset = sprite.offset;
        sprite.set_offset(offset - Point::new(MOUTH_RADIUS as i32, MOUTH_RADIUS as i32));
        sprite.draw(target)?;
        sprite.set_offset(prev_offset);
        Ok(())
    }

    fn draw_face<Target: DrawTarget<Color = Rgb565>>(&mut self, left_eye_state: EyeState, right_eye_state: EyeState, target: &mut Target) -> Result<(), Target::Error> {
        let screen_size = target.bounding_box().size;
        self.draw_eye(Point::new(EYE_OFFSET_X as i32, EYE_OFFSET_Y as i32), left_eye_state, target)?;
        self.draw_eye(Point::new((screen_size.width -  EYE_OFFSET_X as u32) as i32, EYE_OFFSET_Y as i32), right_eye_state, target)?;
        self.draw_mouth(Point::new((screen_size.width as i32) / 2, (screen_size.height - MOUTH_OFFSET as u32) as i32), target)?;
        Ok(())
    }
}