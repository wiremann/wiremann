use gpui::{App, KeyBinding, actions};

use crate::{controller::{Controller, state::PlaybackStatus}, ui::components::Page};

actions!(player, [PlayPause, Next, Prev, Shuffle, Repeat, SeekBack, SeekForward]);
actions!(pages, [CycleNext, CyclePrev, Library, Player, Playlists]);

pub fn register_keybinds(cx: &mut App) {
    // Player actions
    cx.on_action(play_pause);
    cx.on_action(next);
    cx.on_action(prev);
    cx.on_action(shuffle);
    cx.on_action(repeat);
    cx.on_action(seek_forward);
    cx.on_action(seek_back);

    // Page actions
    cx.on_action(cycle_next);
    cx.on_action(cycle_prev);
    cx.on_action(library);
    cx.on_action(player);
    cx.on_action(playlists);

    // Player binds
    cx.bind_keys([KeyBinding::new("space", PlayPause, None), KeyBinding::new("k", PlayPause, None)]);  

    if cfg!(target_os = "macos") {
        cx.bind_keys([KeyBinding::new("cmd-left", Prev, None)]);
        cx.bind_keys([KeyBinding::new("cmd-right", Next, None)]);
    } else {
        cx.bind_keys([KeyBinding::new("ctrl-left", Prev, None)]);
        cx.bind_keys([KeyBinding::new("ctrl-right", Next, None)]);
    }

    cx.bind_keys([KeyBinding::new("left", SeekBack, None)]);
    cx.bind_keys([KeyBinding::new("right", SeekForward, None)]);

    cx.bind_keys([KeyBinding::new("shift-s", Shuffle, None)]);
    cx.bind_keys([KeyBinding::new("shift-r", Repeat, None)]);

    // Page binds
    if cfg!(target_os = "macos") {
        cx.bind_keys([KeyBinding::new("cmd-tab", CycleNext, None)]);
        cx.bind_keys([KeyBinding::new("cmd-shift-tab", CyclePrev, None)]);
        cx.bind_keys([KeyBinding::new("cmd-1", Library, None)]);
        cx.bind_keys([KeyBinding::new("cmd-2", Player, None)]);
        cx.bind_keys([KeyBinding::new("cmd-3", Playlists, None)]);
    } else {
        cx.bind_keys([KeyBinding::new("ctrl-tab", CycleNext, None)]);
        cx.bind_keys([KeyBinding::new("ctrl-shift-tab", CyclePrev, None)]);
        cx.bind_keys([KeyBinding::new("ctrl-1", Library, None)]);
        cx.bind_keys([KeyBinding::new("ctrl-2", Player, None)]);
        cx.bind_keys([KeyBinding::new("ctrl-3", Playlists, None)]);
    }
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

fn seek_forward(_: &SeekForward, cx: &mut App) {
    let controller = cx.global::<Controller>().clone();
    let current = controller.state.read(cx).playback.position.clone();
    controller.seek(current.saturating_add(5));
}

fn seek_back(_: &SeekBack, cx: &mut App) {
    let controller = cx.global::<Controller>().clone();
    let current = controller.state.read(cx).playback.position.clone();
    controller.seek(current.saturating_sub(5));
}

fn cycle_next(_: &CycleNext, cx: &mut App) {
    let current = cx.global::<Page>().clone();

    let next = match current {
        Page::Library => Page::Player,
        Page::Player => Page::Playlists,
        Page::Playlists => Page::Library
    };

    *cx.global_mut::<Page>() = next;
}

fn cycle_prev(_: &CyclePrev, cx: &mut App) {
    let current = cx.global::<Page>().clone();

    let prev = match current {
        Page::Library => Page::Playlists,
        Page::Player => Page::Library,
        Page::Playlists => Page::Player
    };

    *cx.global_mut::<Page>() = prev;
}

fn library(_: &Library, cx: &mut App) {
    *cx.global_mut::<Page>() = Page::Library;
}

fn player(_: &Player, cx: &mut App) {
    *cx.global_mut::<Page>() = Page::Player;
}

fn playlists(_: &Playlists, cx: &mut App) {
    *cx.global_mut::<Page>() = Page::Playlists;
}
