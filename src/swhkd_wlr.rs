use std::{env, env::args, process::exit};
use wayland_client::{protocol::wl_seat::WlSeat, Display, GlobalManager};
use wayland_protocols::misc::zwp_input_method_v2::client::{
    zwp_input_method_keyboard_grab_v2, zwp_input_method_manager_v2::ZwpInputMethodManagerV2,
};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var("RUST_LOG", "swhkd_wlr=info");
    let mut args = args();
    if let Some(arg) = args.nth(1) {
        match arg.as_str() {
            "-d" => env::set_var("RUST_LOG", "swhkd_wlr=trace"),
            _ => {
                println!("Usage:\nswhkd-wlr [FLAGS]\n\nFlags:\n-d -- debug",);
                exit(1);
            }
        }
    }
    env_logger::init();
    log::trace!("Logger initialized.");

    let display = Display::connect_to_env()
        .map_err(|e| log::error!("Failed to connect to wayland display: {}", e))
        .unwrap();
    log::debug!("Connected to wayland display.");

    let mut event_queue = display.create_event_queue();
    log::debug!("Created the wayland event queue.");

    let attached_display = display.attach(event_queue.token());
    log::debug!("Created the attached display with respect to the queue token.");

    let globals = GlobalManager::new(&attached_display);
    log::debug!("Created the Global manager from attached display.");
    log::debug!("Dispatching all buffered requests from the event queue.");
    event_queue.sync_roundtrip(&mut (), |_, _, _| unreachable!())?;

    // Get the current seat from the compositor.
    let wl_seat = match globals.instantiate_exact::<WlSeat>(1) {
        Ok(seat) => seat,
        Err(e) => {
            log::error!("Failed to get the current seat from your compositor: {}", e);
            exit(1);
        }
    };
    log::debug!("Received the current WlSeat from the compositor.");

    // Get the input method manager of version 1 from the globalmanager. ( Coereced type with turbofish )
    let input_method_manager = match globals.instantiate_exact::<ZwpInputMethodManagerV2>(1) {
        Ok(input_method_manager) => input_method_manager,
        Err(e) => {
            log::error!("Failed to find input manager. Does your compositor support zwp_input_method_manager_v2?");
            panic!("{:#?}", e);
        }
    };
    log::debug!("Received the ZwpInputMethodManagerV2 from the compositor.");

    // Grab the keyboard.
    let input_method = input_method_manager.get_input_method(&wl_seat);
    let keyboard_grab = input_method.grab_keyboard();

    // Assign callbacks to the keyboard_grab object.
    keyboard_grab.quick_assign({
        move |_, event, _| match event {
            zwp_input_method_keyboard_grab_v2::Event::Keymap { format, fd, size } => {
                log::info!("Keymap event fired!");
                log::debug!("Format: {:#?} Fd: {:#?} Size: {:#?}", format, fd, size);
            }

            zwp_input_method_keyboard_grab_v2::Event::Key { serial, time, key, state, } => {
                log::info!("Key event fired!");
                log::debug!("Serial: {:#?} Time: {:#?} Key: {:#?} State: {:#?}", serial, time, key, state);
            }

            zwp_input_method_keyboard_grab_v2::Event::Modifiers { serial, mods_depressed, mods_latched, mods_locked, group,} => {
                log::info!("Modifiers event fired!");
                log::debug!("Serial: {:#?} Mods Depressed: {:#?} Mods Latched: {:#?} Mods locked: {:#?} Group: {:#?}", serial , mods_depressed , mods_latched , mods_locked , group);
            }

            zwp_input_method_keyboard_grab_v2::Event::RepeatInfo { rate, delay } => {
                log::info!("RepeatInfo event fired!");
                log::debug!("Rate: {:#?} Delay: {:#?}", rate, delay);
            }
            _ => unreachable!(),
        }
    });
    log::debug!("Dispatching all buffered requests from the event queue.");
    event_queue.sync_roundtrip(&mut (), |_, _, _| {}).unwrap();

    log::debug!("Releasing the WlSeat.");
    keyboard_grab.detach().release();
    event_queue.sync_roundtrip(&mut (), |_, _, _| {}).unwrap();

    Ok(())
}
