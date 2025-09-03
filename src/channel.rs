use std::{
    ffi::{c_void, CStr, CString},
    mem, ptr,
};

use gobject_sys::*;
use gtk_sys::*;

use crate::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChannelType {
    Mic = 0,
    Instr,
    Line,
    Pcm,
}

#[derive(Debug)]
pub struct Output {
    name_l: &'static str,
    name_r: &'static str,
    elem_l: *mut snd_mixer_elem_t,
    elem_r: *mut snd_mixer_elem_t,
}

#[derive(Debug)]
pub struct Channel {
    name: &'static str,
    outputs: [*mut Output; NUMBER_OF_OUTPUTS],
    cur_output: *mut Output,
    pub bt_48v: *mut GtkWidget,
    pub bt_pad: *mut GtkWidget,
    pub cb_sens: *mut GtkWidget,
    pub sc_vol: *mut GtkWidget,
    pub sc_pan: *mut GtkWidget,
    pub lbl_name: *mut GtkWidget,
    no_signals: bool,
    r#type: ChannelType,
    phantom: *mut snd_mixer_elem_t,
    pad: *mut snd_mixer_elem_t,
    sens: *mut snd_mixer_elem_t,
}

impl Channel {
    pub fn new(name: &'static str, r#type: ChannelType) -> Self {
        let mut outputs = [ptr::null_mut(); NUMBER_OF_OUTPUTS];

        for output in outputs.iter_mut().take(NUMBER_OF_OUTPUTS) {
            let layout = Layout::new::<Output>();
            let ptr = unsafe { alloc(layout).cast::<Output>() };
            *output = ptr;
        }

        Self {
            name,
            outputs,
            cur_output: ptr::null_mut(),
            bt_48v: ptr::null_mut(),
            bt_pad: ptr::null_mut(),
            cb_sens: ptr::null_mut(),
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
    let c: &mut Channel = &mut *snd_mixer_elem_get_callback_private(elem).cast::<Channel>();
    if mask == SND_CTL_EVENT_MASK_REMOVE {
        channel_reset(c);
    } else if mask == SND_CTL_EVENT_MASK_VALUE {
        if c.no_signals {
            return 0;
        }
        update_sliders(c);
    }
    0
}

unsafe extern "C" fn on_selem_changed_48v(elem: *mut snd_mixer_elem_t, mask: u32) -> i32 {
    let c: &mut Channel = &mut *snd_mixer_elem_get_callback_private(elem).cast::<Channel>();
    if mask == SND_CTL_EVENT_MASK_REMOVE {
        c.phantom = ptr::null_mut();
    } else if mask == SND_CTL_EVENT_MASK_VALUE {
        if c.no_signals {
            return 0;
        }
        update_switches(c);
    }
    0
}

unsafe extern "C" fn on_selem_changed_pad(elem: *mut snd_mixer_elem_t, mask: u32) -> i32 {
    let c: &mut Channel = &mut *snd_mixer_elem_get_callback_private(elem).cast::<Channel>();
    if mask == SND_CTL_EVENT_MASK_REMOVE {
        c.pad = ptr::null_mut();
    } else if mask == SND_CTL_EVENT_MASK_VALUE {
        if c.no_signals {
            return 0;
        }
        update_switches(c);
    }
    0
}

unsafe extern "C" fn on_selem_changed_sens(elem: *mut snd_mixer_elem_t, mask: u32) -> i32 {
    let c: &mut Channel = &mut *snd_mixer_elem_get_callback_private(elem).cast::<Channel>();

    if mask == SND_CTL_EVENT_MASK_REMOVE {
        c.sens = ptr::null_mut();
    } else if mask == SND_CTL_EVENT_MASK_VALUE {
        if c.no_signals {
            return 0;
        }
        update_switches(c);
    }
    0
}

unsafe extern "C" fn on_bt_toggled_48v(button: *mut GtkWidget, user_data: gpointer) {
    log::debug!("48V toggled");
    let c: &mut Channel = &mut *user_data.cast::<Channel>();

    if c.no_signals || c.phantom.is_null() {
        return;
    }

    let v = gtk_toggle_button_get_active(button.cast::<GtkToggleButton>());
    c.no_signals = true;
    snd_mixer_selem_set_playback_switch(c.phantom, 0, i32::from(v == 1));
    c.no_signals = false;
}

unsafe extern "C" fn on_bt_toggled_pad(button: *mut GtkWidget, user_data: gpointer) {
    log::debug!("PAD toggled");
    let c: &mut Channel = &mut *user_data.cast::<Channel>();

    if c.no_signals || c.pad.is_null() {
        return;
    }

    let v = gtk_toggle_button_get_active(button.cast::<GtkToggleButton>());
    c.no_signals = true;
    snd_mixer_selem_set_playback_switch(c.pad, 0, i32::from(v == 1));
    c.no_signals = false;
}

unsafe extern "C" fn on_cb_sens(combo: *mut GtkWidget, user_data: gpointer) {
    let c: &mut Channel = &mut *user_data.cast::<Channel>();

    if c.no_signals || c.sens.is_null() {
        return;
    }

    let active = gtk_combo_box_get_active(combo.cast::<GtkComboBox>());
    if !(0..=1).contains(&active) {
        return;
    }

    c.no_signals = true;
    snd_mixer_selem_set_enum_item(c.sens, 0, active as u32);
    c.no_signals = false;
}

unsafe extern "C" fn on_slider_changed(_slider: *mut GtkWidget, user_data: gpointer) {
    let c: &mut Channel = &mut *user_data.cast::<Channel>();

    log::debug!("Slider changed {}", c.name);

    if c.no_signals
        || c.cur_output.is_null()
        || (*c.cur_output).elem_l.is_null()
        || (*c.cur_output).elem_r.is_null()
    {
        return;
    }

    let mut pan = gtk_range_get_value(c.sc_pan.cast::<GtkRange>());
    let mut vol = gtk_range_get_value(c.sc_vol.cast::<GtkRange>());

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

pub unsafe fn channel_init(channel: &mut Channel, r#type: ChannelType, name: &'static str) {
    log::debug!("Init channel '{name}' ({type:?})");

    *channel = Channel::new(name, r#type);

    for (i, output) in OUTPUTS.iter().enumerate().take(NUMBER_OF_OUTPUTS) {
        (*channel.outputs[i]).name_l = output[0];
        (*channel.outputs[i]).name_r = output[1];
        (*channel.outputs[i]).elem_l = ptr::null_mut();
        (*channel.outputs[i]).elem_r = ptr::null_mut();
    }

    let label_text = CString::new(name).unwrap();
    channel.lbl_name = gtk_label_new(label_text.as_ptr());
    channel.sc_pan = gtk_scale_new_with_range(GTK_ORIENTATION_HORIZONTAL, -100.0, 100.0, 1.0);

    gtk_range_set_value(channel.sc_pan.cast::<GtkRange>(), 0.0);
    gtk_scale_add_mark(
        channel.sc_pan.cast::<GtkScale>(),
        0.0,
        GTK_POS_TOP,
        ptr::null(),
    );

    g_signal_connect_data(
        channel.sc_pan.cast::<GObject>(),
        c"value-changed".as_ptr(),
        Some(mem::transmute(on_slider_changed as *const ())),
        std::ptr::from_mut(channel).cast::<c_void>(),
        None,
        0,
    );

    channel.sc_vol = gtk_scale_new_with_range(
        GTK_ORIENTATION_VERTICAL,
        BBF_VOL_SLIDER_MIN,
        BBF_VOL_SLIDER_MAX,
        1.0,
    );

    gtk_range_set_inverted(channel.sc_vol.cast::<GtkRange>(), 1);
    gtk_scale_add_mark(
        channel.sc_vol.cast::<GtkScale>(),
        BBF_VOL_SLIDER_ZERO_DB,
        GTK_POS_RIGHT,
        ptr::null(),
    );

    g_signal_connect_data(
        channel.sc_vol.cast::<GObject>(),
        c"value-changed".as_ptr(),
        Some(mem::transmute(on_slider_changed as *const ())),
        std::ptr::from_mut(channel).cast::<c_void>(),
        None,
        0,
    );
    g_signal_connect_data(
        channel.sc_vol.cast::<GObject>(),
        c"format-value".as_ptr(),
        mem::transmute(on_slider_format_value as *const ()),
        std::ptr::from_mut(channel).cast::<c_void>(),
        None,
        0,
    );

    if channel.r#type == ChannelType::Mic {
        let button_text = CString::new("48V").unwrap();
        channel.bt_48v = gtk_toggle_button_new_with_label(button_text.as_ptr());

        g_signal_connect_data(
            channel.bt_48v.cast::<GObject>(),
            c"toggled".as_ptr(),
            Some(mem::transmute(on_bt_toggled_48v as *const ())),
            std::ptr::from_mut(channel).cast::<c_void>(),
            None,
            0,
        );

        let button_text = CString::new("PAD").unwrap();
        channel.bt_pad = gtk_toggle_button_new_with_label(button_text.as_ptr());

        g_signal_connect_data(
            channel.bt_pad.cast::<GObject>(),
            c"toggled".as_ptr(),
            Some(mem::transmute(on_bt_toggled_pad as *const ())),
            std::ptr::from_mut(channel).cast::<c_void>(),
            None,
            0,
        );
    } else if channel.r#type == ChannelType::Instr {
        channel.cb_sens = gtk_combo_box_text_new();
        gtk_combo_box_text_append(
            channel.cb_sens.cast::<GtkComboBoxText>(),
            ptr::null(),
            c"-10 dBV".as_ptr(),
        );
        gtk_combo_box_text_append(
            channel.cb_sens.cast::<GtkComboBoxText>(),
            ptr::null(),
            c"+4 dBu".as_ptr(),
        );

        g_signal_connect_data(
            channel.cb_sens.cast::<GObject>(),
            c"changed".as_ptr(),
            Some(mem::transmute(on_cb_sens as *const ())),
            std::ptr::from_mut(channel).cast::<c_void>(),
            None,
            0,
        );
    }
}

pub unsafe fn channel_reset(channel: *mut Channel) {
    for i in 0..NUMBER_OF_OUTPUTS {
        (*(*channel).outputs[i]).elem_l = ptr::null_mut();
        (*(*channel).outputs[i]).elem_r = ptr::null_mut();
    }
    (*channel).pad = ptr::null_mut();
    (*channel).phantom = ptr::null_mut();
    (*channel).sens = ptr::null_mut();
}

pub unsafe fn channel_set_output(channel: *mut Channel, output: usize) {
    if output > NUMBER_OF_OUTPUTS {
        return;
    }
    (*channel).cur_output = (*channel).outputs[output];
    update_sliders(channel);
}

pub unsafe fn channel_find_and_set(channel: *mut Channel, elem: *mut snd_mixer_elem_t) -> bool {
    let Ok(elem_name) = CStr::from_ptr(snd_mixer_selem_get_name(elem)).to_str() else {
        return false;
    };

    if (*channel).r#type == ChannelType::Mic {
        if format!("Mic-{} 48V", (*channel).name) == elem_name {
            (*channel).phantom = elem;
            snd_mixer_elem_set_callback(elem, Some(on_selem_changed_48v));
            snd_mixer_elem_set_callback_private(elem, channel.cast());
            update_switches(&mut *channel);
            return true;
        }
        if format!("Mic-{} PAD", (*channel).name) == elem_name {
            (*channel).pad = elem;
            snd_mixer_elem_set_callback(elem, Some(on_selem_changed_pad));
            snd_mixer_elem_set_callback_private(elem, channel.cast());
            update_switches(&mut *channel);
            return true;
        }
    } else if (*channel).r#type == ChannelType::Instr
        && format!("Line-{} Sens.", (*channel).name) == elem_name
    {
        (*channel).sens = elem;
        snd_mixer_elem_set_callback(elem, Some(on_selem_changed_sens));
        snd_mixer_elem_set_callback_private(elem, channel.cast());
        update_switches(&mut *channel);
        return true;
    }
    for i in 0..NUMBER_OF_OUTPUTS {
        for j in 0..2 {
            let channel_type_str = match (*channel).r#type {
                ChannelType::Mic => "Mic",
                ChannelType::Pcm => "PCM",
                ChannelType::Line | ChannelType::Instr => "Line",
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
                snd_mixer_elem_set_callback_private(elem, channel.cast());

                return true;
            }
        }
    }
    false
}

unsafe fn update_switches(channel: &mut Channel) {
    channel.no_signals = true;
    if channel.r#type == ChannelType::Mic {
        if !channel.phantom.is_null() {
            let mut phantom = 0;
            snd_mixer_selem_get_playback_switch(channel.phantom, 0, &raw mut phantom);
            gtk_toggle_button_set_active(channel.bt_48v.cast(), i32::from(phantom == 1));
        }

        if !channel.pad.is_null() {
            let mut pad = 0;
            snd_mixer_selem_get_playback_switch(channel.pad, 0, &raw mut pad);
            gtk_toggle_button_set_active(channel.bt_pad.cast(), i32::from(pad == 1));
        }
    } else if channel.r#type == ChannelType::Instr && !channel.sens.is_null() {
        let mut item = 0;
        snd_mixer_selem_get_enum_item(channel.sens, 0, &raw mut item);
        gtk_combo_box_set_active(channel.cb_sens.cast(), /*GTK_COMBO_BOX*/ item as i32);
    }
    channel.no_signals = false;
}

unsafe fn update_sliders(channel: *mut Channel) {
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
    snd_mixer_selem_get_playback_volume((*(*channel).cur_output).elem_l, cid, &raw mut val_l);
    snd_mixer_selem_get_playback_volume((*(*channel).cur_output).elem_r, cid, &raw mut val_r);

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
    gtk_range_set_value((*channel).sc_pan.cast::<GtkRange>(), pan);

    let fader = if fader >= BBF_VOL_ZERO_DB {
        ((BBF_VOL_SLIDER_MAX - BBF_VOL_SLIDER_ZERO_DB) / (BBF_VOL_MAX as f64 - BBF_VOL_ZERO_DB))
            * (fader - BBF_VOL_ZERO_DB)
            + BBF_VOL_SLIDER_ZERO_DB
    } else {
        ((BBF_VOL_SLIDER_ZERO_DB - BBF_VOL_SLIDER_MIN) / (BBF_VOL_ZERO_DB - BBF_VOL_MIN as f64))
            * fader
    };

    gtk_range_set_value((*channel).sc_vol.cast::<GtkRange>(), fader);
    (*channel).no_signals = false;
}
