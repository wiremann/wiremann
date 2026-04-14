use gpui::{App, AppContext, KeyBinding, actions};

use crate::controller::{Controller, state::PlaybackStatus};

actions!(player, [PlayPause, Next, Prev, Shuffle, Repeat]);

pub fn register_keybinds(cx: &mut App) {
    cx.on_action(play_pause);
    cx.on_action(next);
    cx.on_action(prev);
    cx.on_action(shuffle);
    cx.on_action(repeat);

    cx.bind_keys([KeyBinding::new("space", PlayPause, None)]);  

    if cfg!(target_os = "macos") {
        cx.bind_keys([KeyBinding::new("cmd-left", Prev, None)]);
        cx.bind_keys([KeyBinding::new("cmd-right", Next, None)]);
    } else {
        cx.bind_keys([KeyBinding::new("ctrl-left", Prev, None)]);
        cx.bind_keys([KeyBinding::new("ctrl-right", Next, None)]);
    }

    cx.bind_keys([KeyBinding::new("shift-s", Shuffle, None)]);
    cx.bind_keys([KeyBinding::new("shift-r", Repeat, None)]);
}

fn play_pause(_: &PlayPause, cx: &mut App) {
    let controller = cx.global::<Controller>();
    let status = controller.state.read(cx).playback.status.clone();

    if status == PlaybackStatus::Paused || status == PlaybackStatus::Stopped {
        controller.play();
    } else {
        controller.pause();
    }
}

fn next(_: &Next, cx: &mut App) {
    let controller = cx.global::<Controller>().clone();
    controller.next(cx);
}

fn prev(_: &Prev, cx: &mut App) {
    let controller = cx.global::<Controller>().clone();
    controller.prev(cx);
}

fn shuffle(_: &Shuffle, cx: &mut App) {
    let controller = cx.global::<Controller>().clone();
    controller.set_shuffle(cx);
}

fn repeat(_: &Repeat, cx: &mut App) {
    let controller = cx.global::<Controller>().clone();
    controller.set_repeat(cx);
}
