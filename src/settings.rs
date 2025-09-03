use std::{
    ffi::{c_int, c_void, CStr, CString},
    mem, ptr,
};

use alsa_sys::*;
use gobject_sys::*;
use gtk_sys::*;

use crate::{gpointer, SND_CTL_EVENT_MASK_REMOVE, SND_CTL_EVENT_MASK_VALUE};

#[derive(Debug)]
pub struct Settings {
    pub cb_clock: *mut GtkWidget,
    pub bt_spdif: *mut GtkWidget,
    pub bt_spdif_pro: *mut GtkWidget,
    pub bt_spdif_emph: *mut GtkWidget,
    pub clock: *mut snd_mixer_elem_t,
    pub spdif: *mut snd_mixer_elem_t,
    pub spdif_pro: *mut snd_mixer_elem_t,
    pub spdif_emph: *mut snd_mixer_elem_t,
    pub no_signals: bool,
}

impl Settings {
    const fn new() -> Self {
        Self {
            cb_clock: ptr::null_mut(),
            bt_spdif: ptr::null_mut(),
            bt_spdif_pro: ptr::null_mut(),
            bt_spdif_emph: ptr::null_mut(),
            clock: ptr::null_mut(),
            spdif: ptr::null_mut(),
            spdif_pro: ptr::null_mut(),
            spdif_emph: ptr::null_mut(),
            no_signals: false,
        }
    }
}

unsafe extern "C" fn on_bt_toggled_spdif(button: *mut GtkWidget, user_data: gpointer) {
    let gs: &mut Settings = &mut *user_data.cast::<Settings>();

    log::debug!("Button SPDIF toggled");

    if gs.no_signals {
        return;
    }

    let v = gtk_toggle_button_get_active(button.cast::<GtkToggleButton>());
    gs.no_signals = true;
    snd_mixer_selem_set_playback_switch(gs.spdif, 0, i32::from(v == 1));
    gs.no_signals = false;
}

unsafe extern "C" fn on_bt_toggled_spdif_emph(button: *mut GtkWidget, user_data: gpointer) {
    let gs: &mut Settings = &mut *user_data.cast::<Settings>();

    log::debug!("Button SPDIF Emph. toggled");

    if gs.no_signals {
        return;
    }

    let v = gtk_toggle_button_get_active(button.cast::<GtkToggleButton>());
    gs.no_signals = true;
    snd_mixer_selem_set_playback_switch(gs.spdif_emph, 0, i32::from(v == 1));
    gs.no_signals = false;
}

unsafe extern "C" fn on_bt_toggled_spdif_pro(button: *mut GtkWidget, user_data: gpointer) {
    let gs: &mut Settings = &mut *user_data.cast::<Settings>();

    log::debug!("Button SPDIF Pro toggled");

    if gs.no_signals {
        return;
    }

    let v = gtk_toggle_button_get_active(button.cast::<GtkToggleButton>());
    gs.no_signals = true;
    snd_mixer_selem_set_playback_switch(gs.spdif_pro, 0, i32::from(v == 1));
    gs.no_signals = false;
}

unsafe extern "C" fn on_clock_changed(combo: *mut GtkComboBox, user_data: gpointer) {
    let gs: &mut Settings = &mut *user_data.cast::<Settings>();

    log::debug!("Clock changed");

    if gs.no_signals || gs.clock.is_null() {
        return;
    }

    let active = gtk_combo_box_get_active(combo);
    if !(0..=1).contains(&active) {
        return;
    }
    gs.no_signals = true;
    snd_mixer_selem_set_enum_item(gs.clock, 0, active as u32);
    gs.no_signals = false;
}

pub unsafe fn update_settings(gs: &mut Settings) {
    gs.no_signals = true;
    if !gs.clock.is_null() {
        let mut item = 0;
        snd_mixer_selem_get_enum_item(gs.clock, 0, &raw mut item);
        gtk_combo_box_set_active(gs.cb_clock.cast::<GtkComboBox>(), item as i32);
    }
    if !gs.spdif.is_null() {
        let mut spdif = 0;
        snd_mixer_selem_get_playback_switch(gs.spdif, 0, &raw mut spdif);
        gtk_toggle_button_set_active(gs.bt_spdif.cast::<GtkToggleButton>(), i32::from(spdif == 1));
    }
    if !gs.spdif_emph.is_null() {
        let mut emph = 0;
        snd_mixer_selem_get_playback_switch(gs.spdif_emph, 0, &raw mut emph);
        gtk_toggle_button_set_active(
            gs.bt_spdif_emph.cast::<GtkToggleButton>(),
            i32::from(emph == 1),
        );
    }
    if !gs.spdif_pro.is_null() {
        let mut pro = 0;
        snd_mixer_selem_get_playback_switch(gs.spdif_pro, 0, &raw mut pro);
        gtk_toggle_button_set_active(
            gs.bt_spdif_pro.cast::<GtkToggleButton>(),
            i32::from(pro == 1),
        );
    }
    gs.no_signals = false;
}

unsafe extern "C" fn on_selem_changed_clock(elem: *mut snd_mixer_elem_t, mask: u32) -> c_int {
    log::debug!("on_selem_changed_clock");
    let gs_ptr = unsafe { snd_mixer_elem_get_callback_private(elem) }.cast::<Settings>();
    let gs = unsafe { &mut *gs_ptr };

    if mask == SND_CTL_EVENT_MASK_REMOVE {
        gs.clock = ptr::null_mut();
    } else if mask == SND_CTL_EVENT_MASK_VALUE {
        if gs.no_signals {
            return 0;
        }
        update_settings(gs);
    }

    0
}

unsafe extern "C" fn on_selem_changed_spdif(elem: *mut snd_mixer_elem_t, mask: u32) -> c_int {
    let gs_ptr = unsafe { snd_mixer_elem_get_callback_private(elem) }.cast::<Settings>();
    let gs = unsafe { &mut *gs_ptr };

    if mask == SND_CTL_EVENT_MASK_REMOVE {
        gs.spdif = ptr::null_mut();
    } else if mask == SND_CTL_EVENT_MASK_VALUE {
        if gs.no_signals {
            return 0;
        }
        update_settings(gs);
    }

    0
}

unsafe extern "C" fn on_selem_changed_spdif_emph(elem: *mut snd_mixer_elem_t, mask: u32) -> c_int {
    let gs_ptr = unsafe { snd_mixer_elem_get_callback_private(elem) }.cast::<Settings>();
    let gs = unsafe { &mut *gs_ptr };

    if mask == SND_CTL_EVENT_MASK_REMOVE {
        gs.spdif_emph = ptr::null_mut();
    } else if mask == SND_CTL_EVENT_MASK_VALUE {
        if gs.no_signals {
            return 0;
        }
        update_settings(gs);
    }

    0
}

unsafe extern "C" fn on_selem_changed_spdif_pro(elem: *mut snd_mixer_elem_t, mask: u32) -> c_int {
    let gs_ptr = unsafe { snd_mixer_elem_get_callback_private(elem) }.cast::<Settings>();
    let gs = unsafe { &mut *gs_ptr };

    if mask == SND_CTL_EVENT_MASK_REMOVE {
        gs.spdif_pro = ptr::null_mut();
    } else if mask == SND_CTL_EVENT_MASK_VALUE {
        if gs.no_signals {
            return 0;
        }
        update_settings(gs);
    }
    0
}

pub unsafe fn settings_find_and_set(gs: &mut Settings, elem: *mut snd_mixer_elem_t) -> bool {
    let Ok(name) = CStr::from_ptr(snd_mixer_selem_get_name(elem)).to_str() else {
        return false;
    };

    let gs_ptr: *mut c_void = std::ptr::from_mut(gs).cast::<c_void>();

    match name {
        "Sample Clock Source" => {
            gs.clock = elem;
            snd_mixer_elem_set_callback(elem, Some(on_selem_changed_clock));
            snd_mixer_elem_set_callback_private(elem, gs_ptr);
            update_settings(gs);
            true
        }
        "IEC958" => {
            gs.spdif = elem;
            snd_mixer_elem_set_callback(elem, Some(on_selem_changed_spdif));
            snd_mixer_elem_set_callback_private(elem, gs_ptr);
            update_settings(gs);
            true
        }
        "IEC958 Emphasis" => {
            gs.spdif_emph = elem;
            snd_mixer_elem_set_callback(elem, Some(on_selem_changed_spdif_emph));
            snd_mixer_elem_set_callback_private(elem, gs_ptr);
            update_settings(gs);
            true
        }
        "IEC958 Pro Mask" => {
            gs.spdif_pro = elem;
            snd_mixer_elem_set_callback(elem, Some(on_selem_changed_spdif_pro));
            snd_mixer_elem_set_callback_private(elem, gs_ptr);
            update_settings(gs);
            true
        }
        _ => false,
    }
}

pub unsafe fn settings_init(gs: &mut Settings) {
    *gs = Settings::new();
    log::debug!("Initialize settings");

    gs.cb_clock = gtk_combo_box_text_new();
    gtk_combo_box_text_append(
        gs.cb_clock.cast::<GtkComboBoxText>(),
        ptr::null(),
        c"AutoSync".as_ptr(),
    );
    gtk_combo_box_text_append(
        gs.cb_clock.cast::<GtkComboBoxText>(),
        ptr::null(),
        c"Internal".as_ptr(),
    );
    g_signal_connect_data(
        gs.cb_clock.cast(),
        c"changed".as_ptr(),
        Some(mem::transmute(on_clock_changed as *const ())),
        std::ptr::from_mut(gs).cast::<c_void>(),
        None,
        0,
    );

    let label_text = CString::new("SPDIF").unwrap();
    gs.bt_spdif = gtk_toggle_button_new_with_label(label_text.as_ptr());
    g_signal_connect_data(
        gs.bt_spdif.cast(),
        c"toggled".as_ptr(),
        Some(mem::transmute(on_bt_toggled_spdif as *const ())),
        std::ptr::from_mut(gs).cast::<c_void>(),
        None,
        0,
    );

    let label_text = CString::new("SPDIF Emph.").unwrap();
    gs.bt_spdif_emph = gtk_toggle_button_new_with_label(label_text.as_ptr());
    g_signal_connect_data(
        gs.bt_spdif_emph.cast(),
        c"toggled".as_ptr(),
        Some(mem::transmute(on_bt_toggled_spdif_emph as *const ())),
        std::ptr::from_mut(gs).cast::<c_void>(),
        None,
        0,
    );

    let label_text = CString::new("SPDIF Pro").unwrap();
    gs.bt_spdif_pro = gtk_toggle_button_new_with_label(label_text.as_ptr());
    g_signal_connect_data(
        gs.bt_spdif_pro.cast(),
        c"toggled".as_ptr(),
        Some(mem::transmute(on_bt_toggled_spdif_pro as *const ())),
        std::ptr::from_mut(gs).cast::<c_void>(),
        None,
        0,
    );
}
