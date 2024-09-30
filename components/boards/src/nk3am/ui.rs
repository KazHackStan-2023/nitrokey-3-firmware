use nrf52840_hal::{gpio::Level, pac, prelude::InputPin, pwm, pwm::Pwm};
use trussed::platform::consent;

use super::OutPin;
use crate::ui::{
    buttons::{Button, Press, UserPresence},
    rgb_led::{self, Color},
};

pub struct HardwareButtons {
    touch_button: Option<OutPin>,
}

impl HardwareButtons {
    pub fn new(pin: OutPin) -> Self {
        Self {
            touch_button: Some(pin),
        }
    }
}

impl UserPresence for HardwareButtons {
    fn check_user_presence(&mut self) -> consent::Level {
        if self.is_pressed(Button::A) {
            consent::Level::Normal
        } else {
            consent::Level::None
        }
    }
}

impl Press for HardwareButtons {
    fn is_pressed(&mut self, but: Button) -> bool {
        // As we do not have other buttons,
        // we simply ignore requests for them.
        // Like this they also don't block our time!
        if but == Button::B || but == Button::Middle {
            return false;
        }
        // @TODO: to be discussed how this is intended

        let mut ticks = 0;
        let need_ticks = 100;
        if let Some(touch) = self.touch_button.take() {
            let floating = touch.into_floating_input();

            for idx in 0..need_ticks + 1 {
                match floating.is_low() {
                    Err(_e) => {
                        trace!("is_pressed: err!");
                    }
                    Ok(v) => match v {
                        true => {
                            ticks = idx;
                            break;
                        }
                        false => {
                            if idx >= need_ticks {
                                ticks = idx;
                                break;
                            }
                        }
                    },
                }
            }
            self.touch_button = Some(floating.into_push_pull_output(Level::High));
        }
        ticks >= need_ticks
    }
}

pub struct RgbLed {
    pwm_red: Pwm<pac::PWM0>,
    pwm_green: Pwm<pac::PWM1>,
    pwm_blue: Pwm<pac::PWM2>,
}

impl RgbLed {
    pub fn init_led<T: pwm::Instance>(led: OutPin, raw_pwm: T, channel: pwm::Channel) -> Pwm<T> {
        let pwm = Pwm::new(raw_pwm);
        pwm.set_output_pin(channel, led);
        pwm.set_max_duty(u8::MAX as u16);
        pwm
    }

    pub fn set_led(&mut self, color: Color, channel: pwm::Channel, intensity: u8) {
        let intensity: f32 = intensity as f32;
        match color {
            Color::Red => {
                let duty: u16 = (intensity / 4f32) as u16;
                self.pwm_red.set_duty_on(channel, duty);
            }
            Color::Green => {
                let duty: u16 = (intensity / 2f32) as u16;
                self.pwm_green.set_duty_on(channel, duty);
            }
            Color::Blue => {
                let duty: u16 = (intensity) as u16;
                self.pwm_blue.set_duty_on(channel, duty);
            }
        }
    }
}

impl RgbLed {
    pub fn new(
        leds: [OutPin; 3],
        pwm_red: pac::PWM0,
        pwm_green: pac::PWM1,
        pwm_blue: pac::PWM2,
    ) -> RgbLed {
        let [red, green, blue] = leds;

        let red_pwm_obj = RgbLed::init_led(red, pwm_red, pwm::Channel::C0);
        let green_pwm_obj = RgbLed::init_led(green, pwm_green, pwm::Channel::C1);
        let blue_pwm_obj = RgbLed::init_led(blue, pwm_blue, pwm::Channel::C2);

        Self {
            pwm_red: red_pwm_obj,
            pwm_green: green_pwm_obj,
            pwm_blue: blue_pwm_obj,
        }
    }
}

impl rgb_led::RgbLed for RgbLed {
    fn set_panic_led() {
        unsafe {
            let pac = nrf52840_pac::Peripherals::steal();
            let p0 = nrf52840_hal::gpio::p0::Parts::new(pac.P0);
            let p1 = nrf52840_hal::gpio::p1::Parts::new(pac.P1);

            // red
            p0.p0_08.into_push_pull_output(Level::Low).degrade();
            // green
            p1.p1_09.into_push_pull_output(Level::High).degrade();
            // blue
            p0.p0_12.into_push_pull_output(Level::High).degrade();
        }
    }

    fn red(&mut self, intensity: u8) {
        self.set_led(Color::Red, pwm::Channel::C0, intensity);
    }

    fn green(&mut self, intensity: u8) {
        self.set_led(Color::Green, pwm::Channel::C1, intensity);
    }

    fn blue(&mut self, intensity: u8) {
        self.set_led(Color::Blue, pwm::Channel::C2, intensity);
    }
}
