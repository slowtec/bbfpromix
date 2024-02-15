use std::{
    ffi::{CStr, CString},
    ptr,
};

use gtk_sys::*;

use crate::*;

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub struct bbf_settings_t {
    pub cb_clock: *mut GtkWidget,
    pub bt_spdif: *mut GtkWidget,
    pub bt_spdif_pro: *mut GtkWidget,
    pub bt_spdif_emph: *mut GtkWidget,
    // snd_mixer_elem_t *clock;
    // snd_mixer_elem_t *spdif;
    // snd_mixer_elem_t *spdif_pro;
    // snd_mixer_elem_t *spdif_emph;
    no_signals: bool,
}

impl bbf_settings_t {
    fn new() -> Self {
        Self {
            cb_clock: ptr::null_mut(),
            bt_spdif: ptr::null_mut(),
            bt_spdif_pro: ptr::null_mut(),
            bt_spdif_emph: ptr::null_mut(),
            no_signals: false,
        }
    }
}

unsafe extern "C" fn on_bt_toggled_spdif(_button: *mut GtkWidget, user_data: gpointer) {
    let gs: &mut bbf_settings_t = &mut *(user_data as *mut bbf_settings_t);

    log::debug!("Button SPDIF toggled: {gs:?}");

    if gs.no_signals {
        return;
    }

    // gboolean v = gtk_toggle_button_get_active(GTK_TOGGLE_BUTTON(button));
    // gs->no_signals = true;
    // snd_mixer_selem_set_playback_switch(gs->spdif, 0, v ? 1 : 0);
    // gs->no_signals = false;
}

unsafe extern "C" fn on_bt_toggled_spdif_emph(_button: *mut GtkWidget, user_data: gpointer) {
    let gs: &mut bbf_settings_t = &mut *(user_data as *mut bbf_settings_t);

    log::debug!("Button SPDIF Emph. toggled: {gs:?}");

    if gs.no_signals {
        return;
    }

    // gboolean v = gtk_toggle_button_get_active(GTK_TOGGLE_BUTTON(button));
    // gs->no_signals = true;
    // snd_mixer_selem_set_playback_switch(gs->spdif_emph, 0, v ? 1 : 0);
    // gs->no_signals = false;
}

unsafe extern "C" fn on_bt_toggled_spdif_pro(_button: *mut GtkWidget, user_data: gpointer) {
    let gs: &mut bbf_settings_t = &mut *(user_data as *mut bbf_settings_t);

    log::debug!("Button SPDIF Pro toggled: {gs:?}");

    if gs.no_signals {
        return;
    }

    // gboolean v = gtk_toggle_button_get_active(GTK_TOGGLE_BUTTON(button));
    // gs->no_signals = true;
    // snd_mixer_selem_set_playback_switch(gs->spdif_pro, 0, v ? 1 : 0);
    // gs->no_signals = false;
}

unsafe extern "C" fn on_clock_changed(_comobo: *mut GtkComboBox, user_data: gpointer) {
    let gs: &mut bbf_settings_t = &mut *(user_data as *mut bbf_settings_t);

    log::debug!("Clock changed: {gs:?}");

    // if (gs->no_signals || !gs->clock)
    //     return;
    //
    // gint active = gtk_combo_box_get_active(GTK_COMBO_BOX(combo));
    // if (active < 0 || active > 1)
    //     return;
    //
    // gs->no_signals = true;
    // snd_mixer_selem_set_enum_item(gs->clock, 0, active);
    // gs->no_signals = false;
}

// static void update_settings(bbf_settings_t* gs) {
//     gs->no_signals = true;
//
//     if (gs->clock) {
//         unsigned int item = 0;
//         snd_mixer_selem_get_enum_item(gs->clock, 0, &item);
//         gtk_combo_box_set_active(GTK_COMBO_BOX(gs->cb_clock), item);
//     }
//     if (gs->spdif) {
//         int spdif = 0;
//         snd_mixer_selem_get_playback_switch(gs->spdif, 0, &spdif);
//         gtk_toggle_button_set_active(GTK_TOGGLE_BUTTON(gs->bt_spdif),
//                                      spdif == 1);
//     }
//     if (gs->spdif_emph) {
//         int emph = 0;
//         snd_mixer_selem_get_playback_switch(gs->spdif_emph, 0, &emph);
//         gtk_toggle_button_set_active(GTK_TOGGLE_BUTTON(gs->bt_spdif_emph),
//                                      emph == 1);
//     }
//     if (gs->spdif_pro) {
//         int pro = 0;
//         snd_mixer_selem_get_playback_switch(gs->spdif_pro, 0, &pro);
//         gtk_toggle_button_set_active(GTK_TOGGLE_BUTTON(gs->bt_spdif_pro),
//                                      pro == 1);
//     }
//
//     gs->no_signals = false;
// }
//
// static int on_selem_changed_clock(snd_mixer_elem_t *elem, unsigned int mask) {
//     bbf_settings_t *gs =
//         (bbf_settings_t*)snd_mixer_elem_get_callback_private(elem);
//
//     if (mask == SND_CTL_EVENT_MASK_REMOVE) {
//         gs->clock = NULL;
//     }
//     else if (mask == SND_CTL_EVENT_MASK_VALUE) {
//         if (gs->no_signals)
//             return 0;
//         update_settings(gs);
//     }
//
//     return 0;
// }
//
// static int on_selem_changed_spdif(snd_mixer_elem_t *elem,
//                                        unsigned int mask) {
//     bbf_settings_t *gs =
//         (bbf_settings_t*)snd_mixer_elem_get_callback_private(elem);
//
//     if (mask == SND_CTL_EVENT_MASK_REMOVE) {
//         gs->spdif = NULL;
//     }
//     else if (mask == SND_CTL_EVENT_MASK_VALUE) {
//         if (gs->no_signals)
//             return 0;
//         update_settings(gs);
//     }
//
//     return 0;
// }
//
// static int on_selem_changed_spdif_emph(snd_mixer_elem_t *elem,
//                                        unsigned int mask) {
//     bbf_settings_t *gs =
//         (bbf_settings_t*)snd_mixer_elem_get_callback_private(elem);
//
//     if (mask == SND_CTL_EVENT_MASK_REMOVE) {
//         gs->spdif_emph = NULL;
//     }
//     else if (mask == SND_CTL_EVENT_MASK_VALUE) {
//         if (gs->no_signals)
//             return 0;
//         update_settings(gs);
//     }
//
//     return 0;
// }
//
// static int on_selem_changed_spdif_pro(snd_mixer_elem_t *elem,
//                                        unsigned int mask) {
//     bbf_settings_t *gs =
//         (bbf_settings_t*)snd_mixer_elem_get_callback_private(elem);
//
//     if (mask == SND_CTL_EVENT_MASK_REMOVE) {
//         gs->spdif_pro = NULL;
//     }
//     else if (mask == SND_CTL_EVENT_MASK_VALUE) {
//         if (gs->no_signals)
//             return 0;
//         update_settings(gs);
//     }
//
//     return 0;
// }
//
// bool bbf_settings_find_and_set(bbf_settings_t* gs, snd_mixer_elem_t* elem) {
//     if (strcmp("Sample Clock Source", snd_mixer_selem_get_name(elem)) == 0) {
//         gs->clock = elem;
//         snd_mixer_elem_set_callback(elem, on_selem_changed_clock);
//         snd_mixer_elem_set_callback_private(elem, gs);
//         update_settings(gs);
//         return true;
//     }
//     else if (strcmp("IEC958", snd_mixer_selem_get_name(elem)) == 0) {
//         gs->spdif = elem;
//         snd_mixer_elem_set_callback(elem, on_selem_changed_spdif);
//         snd_mixer_elem_set_callback_private(elem, gs);
//         update_settings(gs);
//         return true;
//     }
//     else if (strcmp("IEC958 Emphasis", snd_mixer_selem_get_name(elem)) == 0) {
//         gs->spdif_emph = elem;
//         snd_mixer_elem_set_callback(elem, on_selem_changed_spdif_emph);
//         snd_mixer_elem_set_callback_private(elem, gs);
//         update_settings(gs);
//         return true;
//     }
//     else if (strcmp("IEC958 Pro Mask", snd_mixer_selem_get_name(elem)) == 0) {
//         gs->spdif_pro = elem;
//         snd_mixer_elem_set_callback(elem, on_selem_changed_spdif_pro);
//         snd_mixer_elem_set_callback_private(elem, gs);
//         update_settings(gs);
//         return true;
//     }
//
//     return false;
// }

pub unsafe fn bbf_settings_init(gs: &mut bbf_settings_t) {
    *gs = bbf_settings_t::new();
    log::debug!("Initialize settings: {gs:?}");

    gs.cb_clock = gtk_combo_box_text_new();
    gtk_combo_box_text_append(
        gs.cb_clock as *mut GtkComboBoxText,
        ptr::null(),
        CStr::from_bytes_with_nul_unchecked(b"AutoSync\0").as_ptr(),
    );
    gtk_combo_box_text_append(
        gs.cb_clock as *mut GtkComboBoxText,
        ptr::null(),
        CStr::from_bytes_with_nul_unchecked(b"Internal\0").as_ptr(),
    );
    g_signal_connect_data(
        gs.cb_clock as *mut _,
        CStr::from_bytes_with_nul_unchecked(b"changed\0").as_ptr(),
        Some(mem::transmute(on_clock_changed as *const ())),
        gs as *mut _ as *mut c_void,
        None,
        0,
    );

    let label_text = CString::new("SPDIF").unwrap();
    gs.bt_spdif = gtk_toggle_button_new_with_label(label_text.as_ptr());
    g_signal_connect_data(
        gs.bt_spdif as *mut _,
        CStr::from_bytes_with_nul_unchecked(b"toggled\0").as_ptr(),
        Some(mem::transmute(on_bt_toggled_spdif as *const ())),
        gs as *mut _ as *mut c_void,
        None,
        0,
    );

    let label_text = CString::new("SPDIF Emph.").unwrap();
    gs.bt_spdif_emph = gtk_toggle_button_new_with_label(label_text.as_ptr());
    g_signal_connect_data(
        gs.bt_spdif_emph as *mut _,
        CStr::from_bytes_with_nul_unchecked(b"toggled\0").as_ptr(),
        Some(mem::transmute(on_bt_toggled_spdif_emph as *const ())),
        gs as *mut _ as *mut c_void,
        None,
        0,
    );

    let label_text = CString::new("SPDIF Pro").unwrap();
    gs.bt_spdif_pro = gtk_toggle_button_new_with_label(label_text.as_ptr());
    g_signal_connect_data(
        gs.bt_spdif_pro as *mut _,
        CStr::from_bytes_with_nul_unchecked(b"toggled\0").as_ptr(),
        Some(mem::transmute(on_bt_toggled_spdif_pro as *const ())),
        gs as *mut _ as *mut c_void,
        None,
        0,
    );
}
