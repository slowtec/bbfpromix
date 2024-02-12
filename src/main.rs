use std::{
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

#[allow(non_camel_case_types)]
type gpointer = *mut c_void;

#[allow(non_camel_case_types)]
type gint = i32;

const G_APPLICATION_FLAGS_NONE: u32 = 0;

// #define BBF_NOF_INPUTS 12
// #define BBF_NOF_OUTPUTS 6
// #define BBF_VOL_MAX 65536
// #define BBF_VOL_MIN 0
// #define BBF_VOL_SLIDER_MAX 120
// #define BBF_VOL_SLIDER_MIN 0
// #define BBF_VOL_SLIDER_ZERO_DB 100
// #define BBF_VOL_ZERO_DB (BBF_VOL_MAX/2.f)

const TRUE: i32 = 1;

// static const char * const BBF_INPUTS[BBF_NOF_INPUTS] = {
//     "AN1", "AN2", "IN3", "IN4", "AS1", "AS2", "ADAT3", "ADAT4", "ADAT5",
//     "ADAT6", "ADAT7", "ADAT8"
// };
//
// static const char * const BBF_OUTPUTS[BBF_NOF_OUTPUTS][2] = {
//     { "AN1", "AN2" },
//     { "PH3", "PH4" },
//     { "AS1", "AS2" },
//     { "ADAT3", "ADAT4" },
//     { "ADAT5", "ADAT6" },
//     { "ADAT7", "ADAT8" }
// };

#[derive(Debug)]
#[allow(non_camel_case_types)]
struct bbf_app_data_t {
    // bbf_channel_t input_channels[BBF_NOF_INPUTS];
    // bbf_channel_t playback_channels[BBF_NOF_INPUTS];
    // bbf_settings_t general_settings;
    mixer: *mut snd_mixer_t, // bool no_signals;
}

impl bbf_app_data_t {
    fn new() -> Self {
        let mixer = ptr::null_mut();
        Self { mixer }
    }
}

// static int connect_alsa_mixer(bbf_app_data_t *app_data) {
//     int err;
//     const char* card = NULL;
//     snd_ctl_card_info_t* info;
//     snd_ctl_card_info_alloca(&info);
//     int number = -1;
//     while (!card) {
//         int err = snd_card_next(&number);
//         if (err < 0 || number < 0) {
//             break;
//         }
//         snd_ctl_t* ctl;
//         char buf[16];
//         sprintf (buf, "hw:%d", number);
//         err = snd_ctl_open(&ctl, buf, 0);
//         if (err < 0) {
//             continue;
//         }
//         err = snd_ctl_card_info(ctl, info);
//         snd_ctl_close(ctl);
//         if (err < 0) {
//             continue;
//         }
//         const char* card_name = snd_ctl_card_info_get_name (info);
//         if (!card_name) {
//             continue;
//         }
//         if (strstr(card_name, "Babyface Pro") != NULL) {
//             // card found
//             card = buf;
//         }
//     }
//     if (!card)
//         return -1;
//     err = snd_mixer_open(&app_data->mixer, 0);
//     if (err < 0)
//         return -2;
//
//     err = snd_mixer_attach(app_data->mixer, card);
//     if (err < 0) {
//         snd_mixer_close(app_data->mixer);
//         app_data->mixer = NULL;
//         return -3;
//     }
//
//     err = snd_mixer_selem_register(app_data->mixer, NULL, NULL);
//     if (err < 0) {
//         snd_mixer_close(app_data->mixer);
//         app_data->mixer = NULL;
//         return -4;
//     }
//
//     err = snd_mixer_load(app_data->mixer);
//     if (err < 0) {
//         snd_mixer_close(app_data->mixer);
//         app_data->mixer = NULL;
//         return -5;
//     }
//     return 0;
// }
//
// static void connect_alsa_mixer_elems(bbf_app_data_t *app_data) {
//     snd_mixer_elem_t* elem;
//     for (elem = snd_mixer_first_elem (app_data->mixer); elem;
//          elem = snd_mixer_elem_next (elem)) {
//
//         if (bbf_settings_find_and_set(&app_data->general_settings, elem))
//             continue;
//
//         for (int i = 0 ; i < BBF_NOF_INPUTS ; ++i) {
//             if (bbf_channel_find_and_set(&app_data->input_channels[i], elem))
//                 continue;
//             if (bbf_channel_find_and_set(&app_data->playback_channels[i], elem))
//                 continue;
//         }
//     }
// }
//
// static void reset_alsa_mixer_elems(bbf_app_data_t *app_data) {
//     for (int i = 0 ; i < BBF_NOF_INPUTS ; ++i) {
//         bbf_channel_reset(&app_data->input_channels[i]);
//         bbf_channel_reset(&app_data->playback_channels[i]);
//     }
// }
//
// static void on_output_changed(GtkComboBox* combo, gpointer user_data) {
//     bbf_app_data_t *app_data = (bbf_app_data_t*)user_data;
//     gint entry_id = gtk_combo_box_get_active(combo);
//     for (int i = 0 ; i < BBF_NOF_INPUTS ; ++i) {
//         bbf_channel_set_output(&app_data->input_channels[i], entry_id);
//         bbf_channel_set_output(&app_data->playback_channels[i], entry_id);
//     }
// }

unsafe extern "C" fn on_timeout(user_data: gpointer) -> gint {
    let app_data: &mut bbf_app_data_t = &mut *(user_data as *mut bbf_app_data_t);

    if app_data.mixer.is_null() {
        // int r = connect_alsa_mixer(app_data);
        // if (r == 0) {
        //     printf("Connected.\n");
        //     connect_alsa_mixer_elems(app_data);
        // }
    } else {
        // int r = snd_mixer_handle_events(app_data->mixer);
        // if (r < 0) {
        //     snd_mixer_close(app_data->mixer);
        //     app_data->mixer = NULL;
        //     printf("disonnected.\n");
        //     reset_alsa_mixer_elems(app_data);
        // }
    }

    return 1;
}

unsafe extern "C" fn activate(app: *mut GtkApplication, user_data: gpointer) {
    log::debug!("Activate GTK application");
    let _app_data: &mut bbf_app_data_t = &mut *(user_data as *mut bbf_app_data_t);

    // GtkWidget *label_inputs, *label_playbacks, *label_output, *label_clock;
    // GtkWidget *separator;
    // GtkWidget *cb_output;

    // Initialize the main window
    let main_window = unsafe { gtk_application_window_new(app) };
    let title = CString::new("Babyface Pro Mixer").unwrap();
    gtk_window_set_title(main_window as *mut GtkWindow, title.as_ptr());
    gtk_window_set_default_size(main_window as *mut GtkWindow, 800, 600);

    // add the main grid
    let main_grid = gtk_grid_new() as *mut GtkGrid;
    gtk_grid_set_column_homogeneous(main_grid, 1);

    // // Inputs
    // label_inputs = gtk_label_new("Hardware Inputs");
    // gtk_widget_set_hexpand(label_inputs, TRUE);
    // gtk_grid_attach(main_grid, label_inputs, 0, 0, 24, 1);
    //
    // for (int i = 0 ; i < BBF_NOF_INPUTS ; ++i) {
    //     bbf_channel_t *ic = &app_data->input_channels[i];
    //     if (i < 2) {
    //         // Mic channel
    //         bbf_channel_init(ic, MIC, BBF_INPUTS[i]);
    //         gtk_grid_attach(main_grid, ic->lbl_name, i*2, 1, 2, 1);
    //         gtk_grid_attach(main_grid, ic->bt_PAD, i*2, 2, 1, 1);
    //         gtk_grid_attach(main_grid, ic->bt_48V, i*2+1, 2, 1, 1);
    //     } else if (i > 1 && i < 4) {
    //         // Instrument channel
    //         bbf_channel_init(ic, INSTR, BBF_INPUTS[i]);
    //         gtk_grid_attach(main_grid, ic->lbl_name, i*2, 1, 2, 1);
    //         gtk_grid_attach(main_grid, ic->cb_Sens, i*2, 2, 2, 1);
    //     } else {
    //         // Line channel
    //         bbf_channel_init(ic, LINE, BBF_INPUTS[i]);
    //         gtk_grid_attach(main_grid, ic->lbl_name, i*2, 1, 2, 1);
    //     }
    //     gtk_grid_attach(main_grid, ic->sc_pan, i*2, 3, 2, 1);
    //     gtk_widget_set_vexpand(ic->sc_vol, TRUE);
    //     gtk_grid_attach(main_grid, ic->sc_vol, i*2, 4, 2, 2);
    // }
    //
    // separator = gtk_separator_new(GTK_ORIENTATION_HORIZONTAL);
    // gtk_grid_attach(main_grid, separator, 0, 6, 24, 1);
    //
    // // Playbacks
    // label_playbacks = gtk_label_new("Software Playback");
    // gtk_widget_set_hexpand(label_playbacks, TRUE);
    // gtk_grid_attach(main_grid, label_playbacks, 0, 7, 24, 1);
    //
    // int pc_count = 0;
    // for (int i = 0 ; i < BBF_NOF_OUTPUTS ; ++i) {
    //     bbf_channel_t *pc = &app_data->playback_channels[pc_count];
    //     bbf_channel_init(pc, PCM, BBF_OUTPUTS[i][0]);
    //     gtk_grid_attach(main_grid, pc->lbl_name, pc_count*2, 8, 2, 1);
    //     gtk_grid_attach(main_grid, pc->sc_pan, pc_count*2, 9, 2, 1);
    //     gtk_widget_set_vexpand(pc->sc_vol, TRUE);
    //     gtk_grid_attach(main_grid, pc->sc_vol, pc_count*2, 10, 2, 2);
    //     pc_count++;
    //
    //     pc = &app_data->playback_channels[pc_count];
    //     bbf_channel_init(pc, PCM, BBF_OUTPUTS[i][1]);
    //     gtk_grid_attach(main_grid, pc->lbl_name, pc_count*2, 8, 2, 1);
    //     gtk_grid_attach(main_grid, pc->sc_pan, pc_count*2, 9, 2, 1);
    //     gtk_widget_set_vexpand(pc->sc_vol, TRUE);
    //     gtk_grid_attach(main_grid, pc->sc_vol, pc_count*2, 10, 2, 2);
    //     pc_count++;
    // }
    //
    // // Output selector
    // label_output = gtk_label_new("Hardware Output:");
    // gtk_grid_attach(main_grid, label_output, 0, 12, 2, 1);
    // cb_output = gtk_combo_box_text_new();
    // for (int i = 0 ; i < BBF_NOF_OUTPUTS ; ++i) {
    //     gtk_combo_box_text_append(GTK_COMBO_BOX_TEXT(cb_output), NULL,
    //                               g_strdup_printf("%s/%s", BBF_OUTPUTS[i][0],
    //                                               BBF_OUTPUTS[i][1]));
    // }
    // g_signal_connect(cb_output, "changed", *G_CALLBACK(on_output_changed),
    //                  app_data);
    // gtk_grid_attach(main_grid, cb_output, 2, 12, 2, 1);
    //
    // // Settings
    // bbf_settings_init(&app_data->general_settings);
    //
    // // Clock
    // label_clock = gtk_label_new("Clock Mode:");
    // gtk_grid_attach(main_grid, label_clock, 4, 12, 2, 1);
    // gtk_grid_attach(main_grid, app_data->general_settings.cb_clock,
    //                 6, 12, 2, 1);
    //
    // // SPDIF
    // gtk_grid_attach(main_grid, app_data->general_settings.bt_spdif,
    //                 10, 12, 2, 1);
    // // SPDIF Emph
    // gtk_grid_attach(main_grid, app_data->general_settings.bt_spdif_emph,
    //                 12, 12, 2, 1);
    // // SPDIF Pro
    // gtk_grid_attach(main_grid, app_data->general_settings.bt_spdif_pro,
    //                 14, 12, 2, 1);

    gtk_widget_set_hexpand(main_grid as *mut GtkWidget, TRUE);
    gtk_container_add(
        main_window as *mut GtkContainer,
        main_grid as *mut GtkWidget,
    );
    gtk_widget_show_all(main_window);
    g_timeout_add(10, Some(on_timeout), user_data);
}

const APP_ID: &str = "de.slowtec.babymixpro";

fn main() -> anyhow::Result<()> {
    env_logger::init();
    log::debug!("Start bbfpromix");

    let mut app_data = bbf_app_data_t::new();

    log::debug!("Create GTK application");
    let app_id = CString::new(APP_ID)?;

    let app = unsafe { gtk_application_new(app_id.as_ptr(), G_APPLICATION_FLAGS_NONE) };

    unsafe {
        g_signal_connect_data(
            app as *mut _,
            CStr::from_bytes_with_nul_unchecked(b"activate\0").as_ptr(),
            Some(mem::transmute(activate as *const ())),
            &mut app_data as *mut _ as *mut c_void,
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
            app as *mut GApplication,
            argc,
            args.as_ptr() as *mut *mut c_char,
        )
    };

    if !app_data.mixer.is_null() {
        unsafe { snd_mixer_close(app_data.mixer) };
    }

    unsafe { g_object_unref(app as *mut GObject) };

    std::process::exit(status);
}
