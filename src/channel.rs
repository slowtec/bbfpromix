use std::{
    ffi::{c_void, CStr, CString},
    mem, ptr,
};

use gobject_sys::*;
use gtk_sys::*;

use crate::*;

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(non_camel_case_types)]
pub enum bbf_channel_type {
    MIC = 0,
    INSTR,
    LINE,
    PCM,
}

// typedef struct __output {
//     bbf_channel_type type;
//     const char *name_l;
//     const char *name_r;
//     snd_mixer_elem_t *elem_l;
//     snd_mixer_elem_t *elem_r;
// } bbf_output_t;

#[derive(Debug)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub struct bbf_channel_t {
    pub name: &'static str,
    //  bbf_output_t outputs[BBF_NOF_OUTPUTS];
    //  bbf_output_t *cur_output;
    pub bt_48V: *mut GtkWidget,
    pub bt_PAD: *mut GtkWidget,
    pub cb_Sens: *mut GtkWidget,
    pub sc_vol: *mut GtkWidget,
    pub sc_pan: *mut GtkWidget,
    pub lbl_name: *mut GtkWidget,
    pub no_signals: bool,
    pub r#type: bbf_channel_type,
    // snd_mixer_elem_t *phantom;
    // snd_mixer_elem_t *pad;
    // snd_mixer_elem_t *sens;
}

impl bbf_channel_t {
    pub fn new(name: &'static str, r#type: bbf_channel_type) -> Self {
        Self {
            name,
            bt_48V: ptr::null_mut(),
            bt_PAD: ptr::null_mut(),
            cb_Sens: ptr::null_mut(),
            sc_vol: ptr::null_mut(),
            sc_pan: ptr::null_mut(),
            lbl_name: ptr::null_mut(),
            no_signals: false,
            r#type,
        }
    }
}

// static int on_selem_changed(snd_mixer_elem_t *elem, unsigned int mask) {
//     bbf_channel_t *c =
//         (bbf_channel_t*)snd_mixer_elem_get_callback_private(elem);
//
//     if (mask == SND_CTL_EVENT_MASK_REMOVE) {
//         bbf_channel_reset(c);
//     }
//     else if (mask == SND_CTL_EVENT_MASK_VALUE) {
//         if (c->no_signals)
//             return 0;
//         bbf_update_sliders(c);
//     }
//
//     return 0;
// }
//
// static int on_selem_changed_48V(snd_mixer_elem_t *elem, unsigned int mask) {
//     bbf_channel_t *c =
//         (bbf_channel_t*)snd_mixer_elem_get_callback_private(elem);
//
//     if (mask == SND_CTL_EVENT_MASK_REMOVE) {
//         c->phantom = NULL;
//     }
//     else if (mask == SND_CTL_EVENT_MASK_VALUE) {
//         if (c->no_signals)
//             return 0;
//         bbf_update_switches(c);
//     }
//
//     return 0;
// }
//
// static int on_selem_changed_pad(snd_mixer_elem_t *elem, unsigned int mask) {
//     bbf_channel_t *c =
//         (bbf_channel_t*)snd_mixer_elem_get_callback_private(elem);
//
//     if (mask == SND_CTL_EVENT_MASK_REMOVE) {
//         c->pad = NULL;
//     }
//     else if (mask == SND_CTL_EVENT_MASK_VALUE) {
//         if (c->no_signals)
//             return 0;
//         bbf_update_switches(c);
//     }
//
//     return 0;
// }
//
// static int on_selem_changed_sens(snd_mixer_elem_t *elem, unsigned int mask) {
//     bbf_channel_t *c =
//         (bbf_channel_t*)snd_mixer_elem_get_callback_private(elem);
//
//     if (mask == SND_CTL_EVENT_MASK_REMOVE) {
//         c->sens = NULL;
//     }
//     else if (mask == SND_CTL_EVENT_MASK_VALUE) {
//         if (c->no_signals)
//             return 0;
//         bbf_update_switches(c);
//     }
//
//     return 0;
// }

#[allow(non_snake_case)]
unsafe extern "C" fn on_bt_toggled_48V(button: *mut GtkWidget, user_data: gpointer) {
    let c: &mut bbf_channel_t = &mut *(user_data as *mut bbf_channel_t);

    // if (c->no_signals || !c->phantom)
    //     return;

    let _v = gtk_toggle_button_get_active(button as *mut GtkToggleButton);
    c.no_signals = true;
    // snd_mixer_selem_set_playback_switch(c->phantom, 0, v ? 1 : 0);
    c.no_signals = false;
}

#[allow(non_snake_case)]
unsafe extern "C" fn on_bt_toggled_PAD(button: *mut GtkWidget, user_data: gpointer) {
    let c: &mut bbf_channel_t = &mut *(user_data as *mut bbf_channel_t);

    // if (c->no_signals || !c->pad)
    //     return;

    let _v = gtk_toggle_button_get_active(button as *mut GtkToggleButton);
    c.no_signals = true;
    // snd_mixer_selem_set_playback_switch(c->pad, 0, v ? 1 : 0);
    c.no_signals = false;
}

unsafe extern "C" fn on_cb_sens(combo: *mut GtkWidget, user_data: gpointer) {
    let c: &mut bbf_channel_t = &mut *(user_data as *mut bbf_channel_t);

    // if (c->no_signals || !c->sens)
    //     return;

    let active = gtk_combo_box_get_active(combo as *mut GtkComboBox);
    if active < 0 || active > 1 {
        return;
    }

    c.no_signals = true;
    // snd_mixer_selem_set_enum_item(c->sens, 0, active);
    c.no_signals = false;
}

unsafe extern "C" fn on_slider_changed(_slider: *mut GtkWidget, user_data: gpointer) {
    let c: &mut bbf_channel_t = &mut *(user_data as *mut bbf_channel_t);

    log::debug!("Slider changed {c:?}");

    // if (c->no_signals || !c->cur_output || !c->cur_output->elem_l ||
    //      !c->cur_output->elem_r)
    //     return;

    let mut pan = gtk_range_get_value(c.sc_pan as *mut GtkRange);
    let mut vol = gtk_range_get_value(c.sc_vol as *mut GtkRange);

    if vol >= BBF_VOL_SLIDER_ZERO_DB {
        vol = (vol - BBF_VOL_SLIDER_ZERO_DB)
            * ((BBF_VOL_MAX as f64 - BBF_VOL_ZERO_DB)
                / (BBF_VOL_SLIDER_MAX - BBF_VOL_SLIDER_ZERO_DB))
            + BBF_VOL_ZERO_DB;
    } else {
        vol *=
            (BBF_VOL_ZERO_DB - BBF_VOL_MIN as f64) / (BBF_VOL_SLIDER_ZERO_DB - BBF_VOL_SLIDER_MIN);
    }

    let val_l;
    let val_r;

    if pan < 0.0 {
        // Rechts reduzieren
        pan = pan * -1.0; // normalisieren
        let diff = vol / 100. * pan;
        val_r = vol - diff;
        val_l = vol;
    } else if pan > 0.0 {
        // links reduzieren
        let diff = vol / 100. * pan;
        val_l = vol - diff;
        val_r = vol;
    } else {
        val_l = vol;
        val_r = vol;
    }

    log::debug!("Translated fader value: {vol:.2}");
    log::debug!("Value for left channel: {}", val_l as isize);
    log::debug!("Value for right channel: {}", val_r as isize);

    c.no_signals = true;
    // snd_mixer_selem_set_playback_volume_all(c->cur_output->elem_l, (int)val_l);
    // snd_mixer_selem_set_playback_volume_all(c->cur_output->elem_r, (int)val_r);
    c.no_signals = false;
}

unsafe extern "C" fn on_slider_format_value(
    _slider: *mut GtkWidget,
    value: f64,
    _user_data: gpointer,
) -> *mut c_char {
    let formatted_string = if value > BBF_VOL_SLIDER_ZERO_DB {
        let calculated_value = 20.0
            * ((value - BBF_VOL_SLIDER_ZERO_DB) / (BBF_VOL_SLIDER_MAX - BBF_VOL_SLIDER_ZERO_DB)
                + 1.0)
                .log10();
        format!("+{calculated_value:.1} dB",)
    } else {
        let calculated_value = 20.0 * (value / BBF_VOL_SLIDER_ZERO_DB).log10();
        format!("{calculated_value:.1} dB")
    };
    let c_str = CString::new(formatted_string).unwrap();
    c_str.into_raw()
}

pub unsafe fn bbf_channel_init(
    channel: &mut bbf_channel_t,
    r#type: bbf_channel_type,
    name: &'static str,
) {
    log::debug!("Init channel '{name}' ({type:?})");

    *channel = bbf_channel_t::new(name, r#type);

    for _i in 0..BBF_NOF_OUTPUTS {
        // channel->outputs[i].name_l = BBF_OUTPUTS[i][0];
        // channel->outputs[i].name_r = BBF_OUTPUTS[i][1];
        // channel->outputs[i].elem_l = NULL;
        // channel->outputs[i].elem_r = NULL;
    }

    let label_text = CString::new(name).unwrap();
    channel.lbl_name = gtk_label_new(label_text.as_ptr());
    channel.sc_pan = gtk_scale_new_with_range(GTK_ORIENTATION_HORIZONTAL, -100.0, 100.0, 1.0);

    gtk_range_set_value(channel.sc_pan as *mut GtkRange, 0.0);
    gtk_scale_add_mark(
        channel.sc_pan as *mut GtkScale,
        0.0,
        GTK_POS_TOP,
        ptr::null(),
    );

    g_signal_connect_data(
        channel.sc_pan as *mut GObject,
        CStr::from_bytes_with_nul_unchecked(b"value-changed\0").as_ptr(),
        Some(mem::transmute(on_slider_changed as *const ())),
        channel as *mut _ as *mut c_void,
        None,
        0,
    );

    channel.sc_vol = gtk_scale_new_with_range(
        GTK_ORIENTATION_VERTICAL,
        BBF_VOL_SLIDER_MIN,
        BBF_VOL_SLIDER_MAX,
        1.0,
    );

    gtk_range_set_inverted(channel.sc_vol as *mut GtkRange, 1);
    gtk_scale_add_mark(
        channel.sc_vol as *mut GtkScale,
        BBF_VOL_SLIDER_ZERO_DB,
        GTK_POS_RIGHT,
        ptr::null(),
    );

    g_signal_connect_data(
        channel.sc_vol as *mut GObject,
        CStr::from_bytes_with_nul_unchecked(b"value-changed\0").as_ptr(),
        Some(mem::transmute(on_slider_changed as *const ())),
        channel as *mut _ as *mut c_void,
        None,
        0,
    );
    g_signal_connect_data(
        channel.sc_vol as *mut GObject,
        CStr::from_bytes_with_nul_unchecked(b"format-value\0").as_ptr(),
        Some(mem::transmute(on_slider_format_value as *const ())),
        channel as *mut _ as *mut c_void,
        None,
        0,
    );

    if channel.r#type == bbf_channel_type::MIC {
        let button_text = CString::new("48V").unwrap();
        channel.bt_48V = gtk_toggle_button_new_with_label(button_text.as_ptr());

        g_signal_connect_data(
            channel.bt_48V as *mut GObject,
            CStr::from_bytes_with_nul_unchecked(b"toggled\0").as_ptr(),
            Some(mem::transmute(on_bt_toggled_48V as *const ())),
            channel as *mut _ as *mut c_void,
            None,
            0,
        );

        let button_text = CString::new("PAD").unwrap();
        channel.bt_PAD = gtk_toggle_button_new_with_label(button_text.as_ptr());

        g_signal_connect_data(
            channel.bt_PAD as *mut GObject,
            CStr::from_bytes_with_nul_unchecked(b"toggled\0").as_ptr(),
            Some(mem::transmute(on_bt_toggled_PAD as *const ())),
            channel as *mut _ as *mut c_void,
            None,
            0,
        );
    } else if channel.r#type == bbf_channel_type::INSTR {
        channel.cb_Sens = gtk_combo_box_text_new();
        gtk_combo_box_text_append(
            channel.cb_Sens as *mut GtkComboBoxText,
            ptr::null(),
            CStr::from_bytes_with_nul_unchecked(b"-10 dBV\0").as_ptr(),
        );
        gtk_combo_box_text_append(
            channel.cb_Sens as *mut GtkComboBoxText,
            ptr::null(),
            CStr::from_bytes_with_nul_unchecked(b"+4 dBu\0").as_ptr(),
        );

        g_signal_connect_data(
            channel.cb_Sens as *mut GObject,
            CStr::from_bytes_with_nul_unchecked(b"changed\0").as_ptr(),
            Some(mem::transmute(on_cb_sens as *const ())),
            channel as *mut _ as *mut c_void,
            None,
            0,
        );
    }
}

// void bbf_channel_reset(bbf_channel_t *channel) {
//     for (int i = 0 ; i < BBF_NOF_OUTPUTS ; ++i) {
//         channel->outputs[i].elem_l = NULL;
//         channel->outputs[i].elem_r = NULL;
//     }
//     channel->pad = NULL;
//     channel->phantom = NULL;
//     channel->sens = NULL;
// }
//
// void bbf_channel_set_output(bbf_channel_t *channel, unsigned int output) {
//     if (output > BBF_NOF_OUTPUTS)
//         return;
//     channel->cur_output = &channel->outputs[output];
//     bbf_update_sliders(channel);
// }
//
//
//
// /** \brief finds and sets the channels mixer elements
//  *
//  * \param pointer to the channel to be checked/modified
//  * \param mixer element to be assigned
//  * \return true if the matching output was found and set
//  *
//  */
// bool bbf_channel_find_and_set(bbf_channel_t *channel, snd_mixer_elem_t *elem) {
//     char name[32];
//     if (channel->type == MIC) {
//         snprintf(name, 32, "Mic-%s 48V", channel->name);
//         if (strcmp(name, snd_mixer_selem_get_name(elem)) == 0) {
//             channel->phantom = elem;
//             snd_mixer_elem_set_callback(elem, on_selem_changed_48V);
//             snd_mixer_elem_set_callback_private(elem, channel);
//             bbf_update_switches(channel);
//             return true;
//         }
//         snprintf(name, 32, "Mic-%s PAD", channel->name);
//         if (strcmp(name, snd_mixer_selem_get_name(elem)) == 0) {
//             channel->pad = elem;
//             snd_mixer_elem_set_callback(elem, on_selem_changed_pad);
//             snd_mixer_elem_set_callback_private(elem, channel);
//             bbf_update_switches(channel);
//             return true;
//         }
//     } else if (channel->type == INSTR) {
//         snprintf(name, 32, "Line-%s Sens.", channel->name);
//         if (strcmp(name, snd_mixer_selem_get_name(elem)) == 0) {
//             channel->sens = elem;
//             snd_mixer_elem_set_callback(elem, on_selem_changed_sens);
//             snd_mixer_elem_set_callback_private(elem, channel);
//             bbf_update_switches(channel);
//             return true;
//         }
//     }
//     for (int i = 0 ; i < BBF_NOF_OUTPUTS ; ++i) {
//         for (int j = 0 ; j < 2 ; ++j) {
//             snprintf(name, 32, "%s-%s-%s",
//                      (channel->type == MIC ? "Mic" :
//                       (channel->type == PCM ? "PCM" : "Line")),
//                      channel->name,
//                      (j == 0 ? channel->outputs[i].name_l :
//                       channel->outputs[i].name_r)
//             );
//             if (strcmp(name, snd_mixer_selem_get_name(elem)) == 0) {
//                 if (j == 0)
//                     channel->outputs[i].elem_l = elem;
//                 else
//                     channel->outputs[i].elem_r = elem;
//
//                 snd_mixer_elem_set_callback(elem, on_selem_changed);
//                 snd_mixer_elem_set_callback_private(elem, channel);
//
//                 return true;
//             }
//         }
//     }
//     return false;
// }
//
// void bbf_update_switches(bbf_channel_t *channel) {
//     channel->no_signals = true;
//     if (channel->type == MIC) {
//         if (channel->phantom) {
//             int phantom = 0;
//             snd_mixer_selem_get_playback_switch(channel->phantom, 0, &phantom);
//             gtk_toggle_button_set_active(GTK_TOGGLE_BUTTON(channel->bt_48V),
//                                          phantom == 1);
//         }
//
//         if (channel->pad) {
//             int pad = 0;
//             snd_mixer_selem_get_playback_switch(channel->pad, 0, &pad);
//             gtk_toggle_button_set_active(GTK_TOGGLE_BUTTON(channel->bt_PAD),
//                                          pad == 1);
//         }
//     } else if (channel->type == INSTR) {
//         if (channel->sens) {
//             unsigned int item = 0;
//             snd_mixer_selem_get_enum_item(channel->sens, 0, &item);
//             gtk_combo_box_set_active(GTK_COMBO_BOX(channel->cb_Sens), item);
//         }
//     }
//     channel->no_signals = false;
// }
//
// void bbf_update_sliders(bbf_channel_t *channel) {
//     if (!channel->cur_output || !channel->cur_output->elem_l ||
//         !channel->cur_output->elem_r)
//         return;
//
//     long val_r = 0;
//     long val_l = 0;
//     channel->no_signals = true;
//     snd_mixer_selem_channel_id_t cid = (snd_mixer_selem_channel_id_t)0;
//     snd_mixer_selem_get_playback_volume(channel->cur_output->elem_l, cid,
//                                         &val_l);
//     snd_mixer_selem_get_playback_volume(channel->cur_output->elem_r, cid,
//                                         &val_r);
//     int32_t diff = val_r - val_l;
//     int8_t pan = 0;
//     uint32_t fader = 0;
//     if (diff < 0) {
//         pan = (int8_t)(100./val_l * diff);
//         fader = val_l;
//     }
//     else if (diff > 0) {
//         pan = (int8_t)(100./val_r * diff);
//         fader = val_r;
//     }
//     else {
//         pan = 0;
//         fader = val_l;
//     }
//     gtk_range_set_value(GTK_RANGE(channel->sc_pan), pan);
//
//     if (fader >= BBF_VOL_ZERO_DB) {
//         fader = ((BBF_VOL_SLIDER_MAX - BBF_VOL_SLIDER_ZERO_DB) /
//                  (BBF_VOL_MAX - BBF_VOL_ZERO_DB)) *
//                 (fader - BBF_VOL_ZERO_DB) + BBF_VOL_SLIDER_ZERO_DB;
//     }
//     else {
//         fader = ((BBF_VOL_SLIDER_ZERO_DB - BBF_VOL_SLIDER_MIN) /
//                  (BBF_VOL_ZERO_DB - BBF_VOL_MIN)) *
//                 fader;
//     }
//
//     gtk_range_set_value(GTK_RANGE(channel->sc_vol), fader);
//     channel->no_signals = false;
// }
