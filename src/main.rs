use std::{
    alloc::{alloc, Layout},
    env,
    ffi::{c_char, c_void, CStr, CString},
    mem, ptr,
};

use alsa_sys::*;
use gio_sys::*;
use glib_sys::*;
use gobject_sys::*;
use gtk_sys::*;

mod channel;
mod settings;

use self::{channel::*, settings::*};

const G_APPLICATION_FLAGS_NONE: u32 = 0;

const NUMBER_OF_INPUTS: usize = 12;
const NUMBER_OF_OUTPUTS: usize = 6;

const BBF_VOL_MAX: usize = 65536;
const BBF_VOL_MIN: usize = 0;
const BBF_VOL_SLIDER_MAX: f64 = 120.0;
const BBF_VOL_SLIDER_MIN: f64 = 0.0;
const BBF_VOL_SLIDER_ZERO_DB: f64 = 100.0;
const BBF_VOL_ZERO_DB: f64 = BBF_VOL_MAX as f64 / 2.0;

const SND_CTL_EVENT_MASK_REMOVE: u32 = !0;
const SND_CTL_EVENT_MASK_VALUE: u32 = 1 << 0;

const TRUE: i32 = 1;

const INPUTS: [&str; NUMBER_OF_INPUTS] = [
    "AN1", "AN2", "IN3", "IN4", "AS1", "AS2", "ADAT3", "ADAT4", "ADAT5", "ADAT6", "ADAT7", "ADAT8",
];

const OUTPUTS: [[&str; 2]; NUMBER_OF_OUTPUTS] = [
    ["AN1", "AN2"],
    ["PH3", "PH4"],
    ["AS1", "AS2"],
    ["ADAT3", "ADAT4"],
    ["ADAT5", "ADAT6"],
    ["ADAT7", "ADAT8"],
];

#[derive(Debug)]
struct AppData {
    input_channels: [*mut Channel; NUMBER_OF_INPUTS],
    playback_channels: [*mut Channel; NUMBER_OF_INPUTS],
    general_settings: *mut Settings,
    mixer: *mut snd_mixer_t,
}

impl AppData {
    fn new() -> Self {
        let mut input_channels = [ptr::null_mut(); NUMBER_OF_INPUTS];
        let mut playback_channels = [ptr::null_mut(); NUMBER_OF_INPUTS];

        for input_channel in input_channels.iter_mut().take(NUMBER_OF_INPUTS) {
            let layout = Layout::new::<Channel>();
            let channel_ptr = unsafe { alloc(layout).cast::<Channel>() };
            *input_channel = channel_ptr;
        }

        for playback_channel in playback_channels.iter_mut().take(NUMBER_OF_INPUTS) {
            let layout = Layout::new::<Channel>();
            let channel_ptr = unsafe { alloc(layout).cast::<Channel>() };
            *playback_channel = channel_ptr;
        }

        let layout = Layout::new::<Settings>();
        let general_settings_ptr = unsafe { alloc(layout).cast::<Settings>() };
        let general_settings = general_settings_ptr;

        let mixer = ptr::null_mut();

        Self {
            input_channels,
            playback_channels,
            general_settings,
            mixer,
        }
    }
}

unsafe fn connect_alsa_mixer(app_data: &mut AppData) -> isize {
    log::debug!("Connect ALSA mixer");
    let mut err;
    let mut card = None;
    let mut info: *mut snd_ctl_card_info_t = ptr::null_mut();
    snd_ctl_card_info_malloc(&raw mut info);
    let mut number = -1;
    while card.is_none() {
        err = snd_card_next(&raw mut number);
        if err < 0 || number < 0 {
            break;
        }
        let mut ctl: *mut snd_ctl_t = ptr::null_mut();
        let buf = CString::new(format!("hw:{number}")).unwrap();
        log::debug!("Try to open sound card {buf:?}");
        err = snd_ctl_open(&raw mut ctl, buf.as_ptr(), 0);
        if err < 0 {
            log::warn!("Unable to open {buf:?}");
            continue;
        }
        err = snd_ctl_card_info(ctl, info);
        snd_ctl_close(ctl);
        if err < 0 {
            continue;
        }
        let card_name_ptr = snd_ctl_card_info_get_name(info);
        if card_name_ptr.is_null() {
            continue;
        }
        let card_name_str = CStr::from_ptr(card_name_ptr).to_str().unwrap();
        log::debug!("Card name is: {card_name_str:?}");

        if card_name_str.contains("Babyface Pro") {
            log::info!("Found {card_name_str:?}");
            card = Some(buf);
        }
    }
    log::debug!("Card is {card:?}");

    let Some(card) = card else {
        return -1;
    };

    log::debug!("Open ALSA mixer");
    err = snd_mixer_open(&raw mut app_data.mixer, 0);
    if err < 0 {
        return -2;
    }

    log::debug!("Attach ALSA mixer");
    err = snd_mixer_attach(app_data.mixer, card.as_ptr());
    if err < 0 {
        snd_mixer_close(app_data.mixer);
        app_data.mixer = ptr::null_mut();
        return -3;
    }

    err = snd_mixer_selem_register(app_data.mixer, ptr::null_mut(), ptr::null_mut());
    if err < 0 {
        snd_mixer_close(app_data.mixer);
        app_data.mixer = ptr::null_mut();
        return -4;
    }

    err = snd_mixer_load(app_data.mixer);
    if err < 0 {
        snd_mixer_close(app_data.mixer);
        app_data.mixer = ptr::null_mut();
        return -5;
    }
    0
}

unsafe fn connect_alsa_mixer_elems(app_data: &mut AppData) {
    let mut elem = snd_mixer_first_elem(app_data.mixer);

    while !elem.is_null() {
        if settings_find_and_set(&mut *app_data.general_settings, elem) {
            elem = snd_mixer_elem_next(elem);
            continue;
        }

        for i in 0..NUMBER_OF_INPUTS {
            if channel_find_and_set(app_data.input_channels[i], elem)
                || channel_find_and_set(app_data.playback_channels[i], elem)
            {
                break;
            }
        }

        elem = snd_mixer_elem_next(elem);
    }
}

unsafe fn reset_alsa_mixer_elems(app_data: &mut AppData) {
    for i in 0..NUMBER_OF_INPUTS {
        channel_reset(app_data.input_channels[i]);
        channel_reset(app_data.playback_channels[i]);
    }
}

unsafe extern "C" fn on_output_changed(combo: *mut GtkComboBox, user_data: gpointer) {
    let app_data: &mut AppData = &mut *user_data.cast::<AppData>();
    let entry_id = gtk_combo_box_get_active(combo) as usize;
    for i in 0..NUMBER_OF_INPUTS {
        channel_set_output(app_data.input_channels[i], entry_id);
        channel_set_output(app_data.playback_channels[i], entry_id);
    }
}

unsafe extern "C" fn on_timeout(user_data: gpointer) -> gint {
    let app_data: &mut AppData = &mut *user_data.cast::<AppData>();

    if app_data.mixer.is_null() {
        let r = connect_alsa_mixer(app_data);
        if r == 0 {
            println!("Connected.");
            connect_alsa_mixer_elems(app_data);
        } else {
            log::warn!("Unable to connect ALSA mixer: {r}");
        }
    } else {
        let r = snd_mixer_handle_events(app_data.mixer);
        if r < 0 {
            snd_mixer_close(app_data.mixer);
            app_data.mixer = ptr::null_mut();
            println!("disonnected.");
            reset_alsa_mixer_elems(app_data);
        }
    }

    1
}

unsafe extern "C" fn activate(app: *mut GtkApplication, user_data: gpointer) {
    log::debug!("Activate GTK application");
    let app_data: &mut AppData = &mut *user_data.cast::<AppData>();

    // Initialize the main window
    let main_window = gtk_application_window_new(app);
    let title = c"Babyface Pro Mixer";
    gtk_window_set_title(main_window.cast::<GtkWindow>(), title.as_ptr());
    gtk_window_set_default_size(main_window.cast::<GtkWindow>(), 800, 600);

    // add the main grid
    let main_grid = gtk_grid_new().cast::<GtkGrid>();
    gtk_grid_set_column_homogeneous(main_grid, 1);

    // Inputs
    let label_text = c"Hardware Inputs";
    let label_inputs = gtk_label_new(label_text.as_ptr());
    gtk_widget_set_hexpand(label_inputs, TRUE);
    gtk_grid_attach(main_grid, label_inputs, 0, 0, 24, 1);

    for (i, input) in INPUTS.iter().enumerate().take(NUMBER_OF_INPUTS) {
        let ic: &mut Channel = &mut *app_data.input_channels[i];

        if i < 2 {
            // Mic channel
            channel_init(ic, ChannelType::Mic, input);
            gtk_grid_attach(main_grid, ic.lbl_name, i as i32 * 2, 1, 2, 1);
            gtk_grid_attach(main_grid, ic.bt_pad, i as i32 * 2, 2, 1, 1);
            gtk_grid_attach(main_grid, ic.bt_48v, i as i32 * 2 + 1, 2, 1, 1);
        } else if i > 1 && i < 4 {
            // Instrument channel
            channel_init(ic, ChannelType::Instr, input);
            gtk_grid_attach(main_grid, ic.lbl_name, i as i32 * 2, 1, 2, 1);
            gtk_grid_attach(main_grid, ic.cb_sens, i as i32 * 2, 2, 2, 1);
        } else {
            // Line channel
            channel_init(ic, ChannelType::Line, input);
            gtk_grid_attach(main_grid, ic.lbl_name, i as i32 * 2, 1, 2, 1);
        }
        gtk_grid_attach(main_grid, ic.sc_pan, i as i32 * 2, 3, 2, 1);
        gtk_widget_set_vexpand(ic.sc_vol, TRUE);
        gtk_grid_attach(main_grid, ic.sc_vol, i as i32 * 2, 4, 2, 2);
    }

    let separator = gtk_separator_new(GTK_ORIENTATION_HORIZONTAL);
    gtk_grid_attach(main_grid, separator, 0, 6, 24, 1);

    // Playbacks
    let label_text = c"Software Playback";
    let label_playbacks = gtk_label_new(label_text.as_ptr());
    gtk_widget_set_hexpand(label_playbacks, TRUE);
    gtk_grid_attach(main_grid, label_playbacks, 0, 7, 24, 1);

    let mut pc_count = 0;

    for output in OUTPUTS.iter().take(NUMBER_OF_OUTPUTS) {
        let pc: &mut Channel = &mut *app_data.playback_channels[pc_count];
        channel_init(pc, ChannelType::Pcm, output[0]);
        gtk_grid_attach(main_grid, pc.lbl_name, pc_count as i32 * 2, 8, 2, 1);
        gtk_grid_attach(main_grid, pc.sc_pan, pc_count as i32 * 2, 9, 2, 1);
        gtk_widget_set_vexpand(pc.sc_vol, TRUE);
        gtk_grid_attach(main_grid, pc.sc_vol, pc_count as i32 * 2, 10, 2, 2);
        pc_count += 1;

        let pc: &mut Channel = &mut *app_data.playback_channels[pc_count];
        channel_init(pc, ChannelType::Pcm, output[1]);
        gtk_grid_attach(main_grid, pc.lbl_name, pc_count as i32 * 2, 8, 2, 1);
        gtk_grid_attach(main_grid, pc.sc_pan, pc_count as i32 * 2, 9, 2, 1);
        gtk_widget_set_vexpand(pc.sc_vol, TRUE);
        gtk_grid_attach(main_grid, pc.sc_vol, pc_count as i32 * 2, 10, 2, 2);
        pc_count += 1;
    }

    // Output selector
    let label_text = c"Hardware Output:";
    let label_output = gtk_label_new(label_text.as_ptr());
    gtk_grid_attach(main_grid, label_output, 0, 12, 2, 1);
    let cb_output = gtk_combo_box_text_new();

    for output in OUTPUTS.iter().take(NUMBER_OF_OUTPUTS) {
        let txt = CString::new(format!("{}/{}", output[0], output[1])).unwrap();
        gtk_combo_box_text_append(
            cb_output.cast::<GtkComboBoxText>(),
            ptr::null(),
            txt.as_ptr(),
        );
    }

    g_signal_connect_data(
        cb_output.cast(),
        c"changed".as_ptr(),
        Some(mem::transmute(on_output_changed as *const ())),
        user_data,
        None,
        0,
    );
    gtk_grid_attach(main_grid, cb_output, 2, 12, 2, 1);

    // Settings
    settings_init(&mut *app_data.general_settings);

    // Clock
    let label_text = c"Clock Mode:";
    let label_clock = gtk_label_new(label_text.as_ptr());
    gtk_grid_attach(main_grid, label_clock, 4, 12, 2, 1);
    gtk_grid_attach(
        main_grid,
        (*app_data.general_settings).cb_clock,
        6,
        12,
        2,
        1,
    );

    // SPDIF
    gtk_grid_attach(
        main_grid,
        (*app_data.general_settings).bt_spdif,
        10,
        12,
        2,
        1,
    );

    // SPDIF Emph
    gtk_grid_attach(
        main_grid,
        (*app_data.general_settings).bt_spdif_emph,
        12,
        12,
        2,
        1,
    );

    // SPDIF Pro
    gtk_grid_attach(
        main_grid,
        (*app_data.general_settings).bt_spdif_pro,
        14,
        12,
        2,
        1,
    );

    gtk_widget_set_hexpand(main_grid.cast::<GtkWidget>(), TRUE);
    gtk_container_add(
        main_window.cast::<GtkContainer>(),
        main_grid.cast::<GtkWidget>(),
    );
    gtk_widget_show_all(main_window);
    g_timeout_add(10, Some(on_timeout), user_data);
}

const APP_ID: &str = "de.slowtec.babymixpro";

fn main() -> anyhow::Result<()> {
    env_logger::init();
    log::debug!("Start bbfpromix");

    let mut app_data = AppData::new();

    log::debug!("Create GTK application");
    let app_id = CString::new(APP_ID)?;

    let app = unsafe { gtk_application_new(app_id.as_ptr(), G_APPLICATION_FLAGS_NONE) };

    unsafe {
        g_signal_connect_data(
            app.cast(),
            c"activate".as_ptr(),
            Some(mem::transmute(activate as *const ())),
            (&raw mut app_data).cast::<c_void>(),
            None,
            0,
        )
    };

    // Convert Rust's argc and argv to C's argc and argv
    let args: Vec<CString> = env::args().map(|arg| CString::new(arg).unwrap()).collect();
    let argv: Vec<*const c_char> = args.iter().map(|arg| arg.as_ptr()).collect();
    let argc = argv.len() as i32;

    log::debug!("Run GTK application");
    let status = unsafe {
        g_application_run(
            app.cast::<GApplication>(),
            argc,
            args.as_ptr() as *mut *mut c_char,
        )
    };

    if !app_data.mixer.is_null() {
        unsafe { snd_mixer_close(app_data.mixer) };
    }

    unsafe { g_object_unref(app.cast::<GObject>()) };

    std::process::exit(status);
}
