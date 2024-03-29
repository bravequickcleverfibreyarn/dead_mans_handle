#![no_std]
#![no_main]

#[cfg(feature = "panic_halt")]
use panic_halt as _;

use microbit::hal::gpio::p0::{P0_02, P0_03};
use microbit::hal::gpio::Level;
use microbit::hal::gpio::{Input, PullDown};
use microbit::hal::gpio::{Output, PushPull};

use microbit::hal::prelude::{InputPin, OutputPin};

use microbit::board::Board;
use microbit::{hal::Timer, pac::TIMER2};

use cortex_m::prelude::_embedded_hal_blocking_delay_DelayMs;
use cortex_m_rt::entry;

#[entry]
fn entry() -> ! {
    let brd = Board::take().unwrap();
    let (mut tmr, mut drv_pp, photo_pd) = prp(brd);

    let hand_ok = move || {
        let low = photo_pd.is_low();
        if let Ok(low) = low {
            low
        } else {
            false
        }
    };

    const DIT: u16 = 195;
    const DAH: u16 = 3 * DIT;
    const INT_SP: u16 = DIT;
    const WRD_SP: u16 = 7 * DIT;

    loop {
        if hand_ok() {
            continue;
        }
        
        // SOS in Morse code: ··· ––– ··· 
        for unit in [DIT, DAH, DIT] {
            for _ in 0..3 {
                if let Ok(_) = drv_pp.set_high() {
                    if hand_ok() {
                        let _ = drv_pp.set_low();
                        break;
                    }

                    tmr.delay_ms(unit);
                }
                if let Ok(_) = drv_pp.set_low() {
                    if hand_ok() {
                        break;
                    }

                    tmr.delay_ms(INT_SP);
                }
            }
        }

        if hand_ok() {
            continue;
        }

        tmr.delay_ms(WRD_SP);
    }
}

fn prp(
    brd: Board,
) -> (
    Timer<TIMER2>,
    P0_02<Output<PushPull>>,
    P0_03<Input<PullDown>>,
) {
    let tmr = Timer::new(brd.TIMER2);

    let pins = brd.pins;

    let p002 = pins.p0_02;
    let drv_pp = p002.into_push_pull_output(Level::Low);

    let p003 = pins.p0_03;
    let photo_pd = p003.into_pulldown_input();

    (tmr, drv_pp, photo_pd)
}

#[cfg(feature = "panic_abort")]
mod panic_abort {
    use core::panic::PanicInfo;

    #[panic_handler]
    fn panic(_info: &PanicInfo) -> ! {
        loop {}
    }
}

// cargo flash --target thumbv7em-none-eabihf --chip nRF52833_xxAA --release --features panic_abort
// cargo flash --target thumbv7em-none-eabihf --chip nRF52833_xxAA --features panic_halt
// cargo build --release  --target thumbv7em-none-eabihf --features panic_abort
// cargo build --target thumbv7em-none-eabihf --features panic_halt
