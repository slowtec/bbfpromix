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

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub struct bbf_output_t {
    pub r#type: *mut bbf_channel_type,
    pub name_l: &'static str,
    pub name_r: &'static str,
    pub elem_l: *mut snd_mixer_elem_t,
    pub elem_r: *mut snd_mixer_elem_t,
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub struct bbf_channel_t {
    pub name: &'static str,
    pub outputs: [*mut bbf_output_t; BBF_NOF_OUTPUTS],
    pub cur_output: *mut bbf_output_t,
    pub bt_48V: *mut GtkWidget,
    pub bt_PAD: *mut GtkWidget,
    pub cb_Sens: *mut GtkWidget,
    pub sc_vol: *mut GtkWidget,
    pub sc_pan: *mut GtkWidget,
    pub lbl_name: *mut GtkWidget,
    pub no_signals: bool,
    pub r#type: bbf_channel_type,
    pub phantom: *mut snd_mixer_elem_t,
    pub pad: *mut snd_mixer_elem_t,
    pub sens: *mut snd_mixer_elem_t,
}

impl bbf_channel_t {
    pub fn new(name: &'static str, r#type: bbf_channel_type) -> Self {
        let mut outputs = [ptr::null_mut(); BBF_NOF_OUTPUTS];

        for i in 0..BBF_NOF_OUTPUTS {
            let layout = Layout::new::<bbf_output_t>();
            let ptr = unsafe { alloc(layout) as *mut bbf_output_t };
            outputs[i] = ptr;
        }

        Self {
            name,
            outputs,
            cur_output: ptr::null_mut(),
            bt_48V: ptr::null_mut(),
            bt_PAD: ptr::null_mut(),
            cb_Sens: ptr::null_mut(),
            sc_vol: ptr::null_mut(),
            sc_pan: ptr::null_mut(),
            lbl_name: ptr::null_mut(),
            no_signals: false,
            r#type,
            phantom: ptr::null_mut(),
            pad: ptr::null_mut(),
            sens: ptr::null_mut(),
        }
    }
}

unsafe extern "C" fn on_selem_changed(elem: *mut snd_mixer_elem_t, mask: u32) -> i32 {
    let c: &mut bbf_channel_t =
        &mut *(snd_mixer_elem_get_callback_private(elem) as *mut bbf_channel_t);
    if mask == SND_CTL_EVENT_MASK_REMOVE {
        bbf_channel_reset(c);
    } else if mask == SND_CTL_EVENT_MASK_VALUE {
        if c.no_signals {
            return 0;
        }
        bbf_update_sliders(c);
    }
    0
}

#[allow(non_snake_case)]
unsafe extern "C" fn on_selem_changed_48V(elem: *mut snd_mixer_elem_t, mask: u32) -> i32 {
    let c: &mut bbf_channel_t =
        &mut *(snd_mixer_elem_get_callback_private(elem) as *mut bbf_channel_t);
    if mask == SND_CTL_EVENT_MASK_REMOVE {
        c.phantom = ptr::null_mut();
    } else if mask == SND_CTL_EVENT_MASK_VALUE {
        if c.no_signals {
            return 0;
        }
        bbf_update_switches(c);
    }
    0
}

unsafe extern "C" fn on_selem_changed_pad(elem: *mut snd_mixer_elem_t, mask: u32) -> i32 {
    let c: &mut bbf_channel_t =
        &mut *(snd_mixer_elem_get_callback_private(elem) as *mut bbf_channel_t);
    if mask == SND_CTL_EVENT_MASK_REMOVE {
        c.pad = ptr::null_mut();
    } else if mask == SND_CTL_EVENT_MASK_VALUE {
        if c.no_signals {
            return 0;
        }
        bbf_update_switches(c);
    }
    0
}

unsafe extern "C" fn on_selem_changed_sens(elem: *mut snd_mixer_elem_t, mask: u32) -> i32 {
    let c: &mut bbf_channel_t =
        &mut *(snd_mixer_elem_get_callback_private(elem) as *mut bbf_channel_t);

    if mask == SND_CTL_EVENT_MASK_REMOVE {
        c.sens = ptr::null_mut();
    } else if mask == SND_CTL_EVENT_MASK_VALUE {
        if c.no_signals {
            return 0;
        }
        bbf_update_switches(c);
    }
    0
}

#[allow(non_snake_case)]
unsafe extern "C" fn on_bt_toggled_48V(button: *mut GtkWidget, user_data: gpointer) {
    log::debug!("48V toggled");
    let c: &mut bbf_channel_t = &mut *(user_data as *mut bbf_channel_t);

    if c.no_signals || c.phantom.is_null() {
        return;
    }

    let v = gtk_toggle_button_get_active(button as *mut GtkToggleButton);
    c.no_signals = true;
    snd_mixer_selem_set_playback_switch(c.phantom, 0, if v == 1 { 1 } else { 0 });
    c.no_signals = false;
}

#[allow(non_snake_case)]
unsafe extern "C" fn on_bt_toggled_PAD(button: *mut GtkWidget, user_data: gpointer) {
    log::debug!("PAD toggled");
    let c: &mut bbf_channel_t = &mut *(user_data as *mut bbf_channel_t);

    if c.no_signals || c.pad.is_null() {
        return;
    }

    let v = gtk_toggle_button_get_active(button as *mut GtkToggleButton);
    c.no_signals = true;
    snd_mixer_selem_set_playback_switch(c.pad, 0, if v == 1 { 1 } else { 0 });
    c.no_signals = false;
}

unsafe extern "C" fn on_cb_sens(combo: *mut GtkWidget, user_data: gpointer) {
    let c: &mut bbf_channel_t = &mut *(user_data as *mut bbf_channel_t);

    if c.no_signals || c.sens.is_null() {
        return;
    }

    let active = gtk_combo_box_get_active(combo as *mut GtkComboBox);
    if !(0..=1).contains(&active) {
        return;
    }

    c.no_signals = true;
    snd_mixer_selem_set_enum_item(c.sens, 0, active as u32);
    c.no_signals = false;
}

unsafe extern "C" fn on_slider_changed(_slider: *mut GtkWidget, user_data: gpointer) {
    let c: &mut bbf_channel_t = &mut *(user_data as *mut bbf_channel_t);

    log::debug!("Slider changed {}", c.name);

    if c.no_signals
        || c.cur_output.is_null()
        || (*c.cur_output).elem_l.is_null()
        || (*c.cur_output).elem_r.is_null()
    {
        return;
    }

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
        pan *= -1.0; // normalisieren
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
    snd_mixer_selem_set_playback_volume_all((*c.cur_output).elem_l, val_l as i64);
    snd_mixer_selem_set_playback_volume_all((*c.cur_output).elem_r, val_r as i64);
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

    for i in 0..BBF_NOF_OUTPUTS {
        (*channel.outputs[i]).name_l = BBF_OUTPUTS[i][0];
        (*channel.outputs[i]).name_r = BBF_OUTPUTS[i][1];
        (*channel.outputs[i]).elem_l = ptr::null_mut();
        (*channel.outputs[i]).elem_r = ptr::null_mut();
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

pub unsafe fn bbf_channel_reset(channel: *mut bbf_channel_t) {
    for i in 0..BBF_NOF_OUTPUTS {
        (*(*channel).outputs[i]).elem_l = ptr::null_mut();
        (*(*channel).outputs[i]).elem_r = ptr::null_mut();
    }
    (*channel).pad = ptr::null_mut();
    (*channel).phantom = ptr::null_mut();
    (*channel).sens = ptr::null_mut();
}

pub unsafe fn bbf_channel_set_output(channel: *mut bbf_channel_t, output: usize) {
    if output > BBF_NOF_OUTPUTS {
        return;
    }
    (*channel).cur_output = (*channel).outputs[output];
    bbf_update_sliders(channel);
}

pub unsafe fn bbf_channel_find_and_set(
    channel: *mut bbf_channel_t,
    elem: *mut snd_mixer_elem_t,
) -> bool {
    let Ok(elem_name) = CStr::from_ptr(snd_mixer_selem_get_name(elem)).to_str() else {
        return false;
    };

    if (*channel).r#type == bbf_channel_type::MIC {
        if format!("Mic-{} 48V", (*channel).name) == elem_name {
            (*channel).phantom = elem;
            snd_mixer_elem_set_callback(elem, Some(on_selem_changed_48V));
            snd_mixer_elem_set_callback_private(elem, channel as *mut _);
            bbf_update_switches(&mut *channel);
            return true;
        }
        if format!("Mic-{} PAD", (*channel).name) == elem_name {
            (*channel).pad = elem;
            snd_mixer_elem_set_callback(elem, Some(on_selem_changed_pad));
            snd_mixer_elem_set_callback_private(elem, channel as *mut _);
            bbf_update_switches(&mut *channel);
            return true;
        }
    } else if (*channel).r#type == bbf_channel_type::INSTR
        && format!("Line-{} Sens.", (*channel).name) == elem_name
    {
        (*channel).sens = elem;
        snd_mixer_elem_set_callback(elem, Some(on_selem_changed_sens));
        snd_mixer_elem_set_callback_private(elem, channel as *mut _);
        bbf_update_switches(&mut *channel);
        return true;
    }
    for i in 0..BBF_NOF_OUTPUTS {
        for j in 0..2 {
            let channel_type_str = if (*channel).r#type == bbf_channel_type::MIC {
                "Mic"
            } else if (*channel).r#type == bbf_channel_type::PCM {
                "PCM"
            } else {
                "Line"
            };

            let output_name = if j == 0 {
                (*(*channel).outputs[i]).name_l
            } else {
                (*(*channel).outputs[i]).name_r
            };

            let name = format!("{channel_type_str}-{}-{output_name}", (*channel).name);

            if name == elem_name {
                if j == 0 {
                    (*(*channel).outputs[i]).elem_l = elem;
                } else {
                    (*(*channel).outputs[i]).elem_r = elem;
                }

                snd_mixer_elem_set_callback(elem, Some(on_selem_changed));
                snd_mixer_elem_set_callback_private(elem, channel as *mut _);

                return true;
            }
        }
    }
    false
}

unsafe fn bbf_update_switches(channel: &mut bbf_channel_t) {
    channel.no_signals = true;
    if channel.r#type == bbf_channel_type::MIC {
        if !channel.phantom.is_null() {
            let mut phantom = 0;
            snd_mixer_selem_get_playback_switch(channel.phantom, 0, &mut phantom);
            gtk_toggle_button_set_active(
                channel.bt_48V as *mut _,
                if phantom == 1 { 1 } else { 0 },
            );
        }

        if !channel.pad.is_null() {
            let mut pad = 0;
            snd_mixer_selem_get_playback_switch(channel.pad, 0, &mut pad);
            gtk_toggle_button_set_active(channel.bt_PAD as *mut _, if pad == 1 { 1 } else { 0 });
        }
    } else if channel.r#type == bbf_channel_type::INSTR && !channel.sens.is_null() {
        let mut item = 0;
        snd_mixer_selem_get_enum_item(channel.sens, 0, &mut item);
        gtk_combo_box_set_active(
            channel.cb_Sens as *mut _,
            /*GTK_COMBO_BOX*/ item as i32,
        );
    }
    channel.no_signals = false;
}

unsafe fn bbf_update_sliders(channel: *mut bbf_channel_t) {
    if (*channel).cur_output.is_null()
        || (*(*channel).cur_output).elem_l.is_null()
        || (*(*channel).cur_output).elem_r.is_null()
    {
        return;
    }

    (*channel).no_signals = true;

    let mut val_r = 0;
    let mut val_l = 0;
    let cid: snd_mixer_selem_channel_id_t = 0;
    snd_mixer_selem_get_playback_volume((*(*channel).cur_output).elem_l, cid, &mut val_l);
    snd_mixer_selem_get_playback_volume((*(*channel).cur_output).elem_r, cid, &mut val_r);

    let diff = val_r - val_l;
    let pan;
    let fader;
    if diff < 0 {
        pan = 100. / val_l as f64 * diff as f64;
        fader = val_l as f64;
    } else if diff > 0 {
        pan = 100. / val_r as f64 * diff as f64;
        fader = val_r as f64;
    } else {
        pan = 0.0;
        fader = val_l as f64;
    }
    gtk_range_set_value((*channel).sc_pan as *mut GtkRange, pan);

    let fader = if fader >= BBF_VOL_ZERO_DB {
        ((BBF_VOL_SLIDER_MAX - BBF_VOL_SLIDER_ZERO_DB) / (BBF_VOL_MAX as f64 - BBF_VOL_ZERO_DB))
            * (fader - BBF_VOL_ZERO_DB)
            + BBF_VOL_SLIDER_ZERO_DB
    } else {
        ((BBF_VOL_SLIDER_ZERO_DB - BBF_VOL_SLIDER_MIN) / (BBF_VOL_ZERO_DB - BBF_VOL_MIN as f64))
            * fader
    };

    gtk_range_set_value((*channel).sc_vol as *mut GtkRange, fader);
    (*channel).no_signals = false;
}
