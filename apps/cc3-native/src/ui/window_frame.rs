//! Frameless window chrome: rounded corners with a 1px #252525 outline, title bar with window
//! controls, drag by title bar (double click maximizes), edge and corner resize handles.
//! Mirrors the Tauri shell (apps/cc3-desktop/src-tauri/src/bridge.js `installChrome`).
use dioxus::prelude::*;
use dioxus::native::use_window;
use winit::window::ResizeDirection;

pub const TITLEBAR_HEIGHT: u32 = 40;
pub const CONTROLS_WIDTH: u32 = 138;
const RADIUS: u32 = 14;
const EDGE: u32 = 6;

const MIN: &str = r#"<path d="M0 5.5h10"/>"#;
const MAX: &str = r#"<rect x="0.5" y="0.5" width="9" height="9"/>"#;
const RESTORE: &str = r#"<path d="M2.5 2.5v-2h7v7h-2M0.5 2.5h7v7h-7z"/>"#;
const CLOSE: &str = r#"<path d="M0 0l10 10M10 0L0 10"/>"#;

#[component]
fn CaptionButton(class: &'static str, icon: &'static str, label: &'static str, onclick: EventHandler<()>) -> Element {
    let svg = format!(r#"<svg viewBox="0 0 10 10" width="10" height="10" fill="none" stroke="currentColor" stroke-width="1">{icon}</svg>"#);
    rsx! {
        button {
            class: "cc3-caption {class}",
            title: label,
            onclick: move |_| onclick.call(()),
            dangerous_inner_html: "{svg}",
        }
    }
}

#[component]
fn ResizeHandle(direction: ResizeDirection, style: String) -> Element {
    let window = use_window();
    rsx! {
        div {
            style: "position:absolute;z-index:50;{style}",
            onmousedown: move |event| {
                if event.trigger_button() == Some(dioxus::html::input_data::MouseButton::Primary) {
                    let _ = window.drag_resize_window(direction);
                }
            },
        }
    }
}

/// `children` fill the area below the title bar. `title_content` is placed left of the controls.
#[component]
pub fn WindowFrame(dark: bool, background: String, title_content: Element, children: Element) -> Element {
    let window = use_window();
    let mut maximized = use_signal(|| false);
    let square = maximized();
    let radius = if square { 0 } else { RADIUS };
    let fg = if dark { "#f8fafc" } else { "#1f2937" };
    let w_drag = window.clone();
    let w_min = window.clone();
    let w_max = window.clone();
    let w_state = window.clone();
    let max_icon = if maximized() { RESTORE } else { MAX };
    rsx! {
        style { {format!(
            "html, body {{ margin: 0; background: transparent; }} \
             .cc3-caption {{ all: unset; box-sizing: border-box; width: 46px; height: 100%; display: flex; align-items: center; justify-content: center; cursor: default; color: {fg}; }} \
             .cc3-caption:hover {{ background: rgba(127,127,127,.22); }} \
             .cc3-caption.close:hover {{ background: #e81123; color: #fff; }}"
        )} }
        div {
            style: "position:relative;width:100vw;height:100vh;border-radius:{radius}px;overflow:hidden;box-sizing:border-box;background:{background};",
            // Title bar: drag region plus controls.
            div {
                style: "position:absolute;top:0;left:0;right:0;height:{TITLEBAR_HEIGHT}px;z-index:40;display:flex;",
                div {
                    style: "flex:1;min-width:0;height:100%;",
                    onmousedown: move |event| {
                        if event.trigger_button() == Some(dioxus::html::input_data::MouseButton::Primary) {
                            let _ = w_drag.drag_window();
                        }
                    },
                    ondoubleclick: move |_| {
                        let next = !w_state.is_maximized();
                        w_state.set_maximized(next);
                        maximized.set(next);
                    },
                    {title_content}
                }
                div { style: "display:flex;height:100%;user-select:none;",
                    CaptionButton { class: "min", icon: MIN, label: "Minimize", onclick: move |_| w_min.set_minimized(true) }
                    CaptionButton {
                        class: "max",
                        icon: max_icon,
                        label: "Maximize",
                        onclick: move |_| {
                            let next = !w_max.is_maximized();
                            w_max.set_maximized(next);
                            maximized.set(next);
                        },
                    }
                    CaptionButton { class: "close", icon: CLOSE, label: "Close", onclick: move |_| { std::process::exit(0); #[allow(unreachable_code)] () } }
                }
            }
            div { style: "position:absolute;top:0;left:0;right:0;bottom:0;", {children} }
            // 1px outline above everything, like `html::after` in the Tauri shell.
            if !square {
                div { style: format!("position:absolute;inset:0;border:1px solid #252525;border-radius:{radius}px;pointer-events:none;z-index:60;") }
            }
            if !square {
                ResizeHandle { direction: ResizeDirection::North, style: format!("top:0;left:{RADIUS}px;right:{RADIUS}px;height:{EDGE}px;cursor:ns-resize;") }
                ResizeHandle { direction: ResizeDirection::South, style: format!("bottom:0;left:{RADIUS}px;right:{RADIUS}px;height:{EDGE}px;cursor:ns-resize;") }
                ResizeHandle { direction: ResizeDirection::West, style: format!("left:0;top:{RADIUS}px;bottom:{RADIUS}px;width:{EDGE}px;cursor:ew-resize;") }
                ResizeHandle { direction: ResizeDirection::East, style: format!("right:0;top:{RADIUS}px;bottom:{RADIUS}px;width:{EDGE}px;cursor:ew-resize;") }
                ResizeHandle { direction: ResizeDirection::NorthWest, style: format!("top:0;left:0;width:{RADIUS}px;height:{RADIUS}px;cursor:nwse-resize;") }
                ResizeHandle { direction: ResizeDirection::NorthEast, style: format!("top:0;right:0;width:{RADIUS}px;height:{RADIUS}px;cursor:nesw-resize;") }
                ResizeHandle { direction: ResizeDirection::SouthWest, style: format!("bottom:0;left:0;width:{RADIUS}px;height:{RADIUS}px;cursor:nesw-resize;") }
                ResizeHandle { direction: ResizeDirection::SouthEast, style: format!("bottom:0;right:0;width:{RADIUS}px;height:{RADIUS}px;cursor:nwse-resize;") }
            }
        }
    }
}
