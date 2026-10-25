use std::sync::atomic::{AtomicUsize, Ordering};

use leptos::prelude::*;

use crate::api;
use crate::components::icons::{IconHistory, IconLogOut, IconUser};
use crate::state::AppState;

const MASTER_LOGO: &str = include_str!("../../../logo.svg");

static LOGO_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

#[component]
pub fn TopBar(state: AppState) -> impl IntoView {
    let health = state.health;
    let open_settings_from_logo = {
        let state = state.clone();
        move |_| state.settings_open.set(true)
    };
    let open_history = {
        let state = state.clone();
        move |_| state.open_history_action()
    };
    let logout = {
        let state = state.clone();
        move |_| {
            let state = state.clone();
            leptos::task::spawn_local(async move {
                let _ = api::logout().await;
                state.clear_session();
            });
        }
    };

    view! {
        <header class="top-bar">
            <div class="top-bar__left">
                <button
                    class="brand-button"
                    type="button"
                    title=move || format!("เปิดการตั้งค่า · {}", health.get().label_th())
                    aria-label="เปิดการตั้งค่า"
                    on:click=open_settings_from_logo
                >
                    <BrandMark />
                </button>
                <div class="top-bar__titles">
                    <span class="top-bar__title">"SnapHerb"</span>
                    <span class="top-bar__subtitle">"ระบบเบิกยาสมุนไพร"</span>
                    <span
                        class=move || {
                            format!(
                                "status-dot top-bar__health status-dot--{}",
                                health.get().dot_class(),
                            )
                        }
                        role="status"
                        aria-label=move || health.get().label_th()
                    >
                        <span class="status-dot__dot" aria-hidden="true"></span>
                    </span>
                </div>
            </div>
            <div class="top-bar__right">
                {move || {
                    state
                        .session
                        .get()
                        .map(|user| {
                            let title = format!("{} · {}", user.display_name, user.login_name);
                            view! {
                                <span class="user-chip" title=title>
                                    <span class="user-chip__avatar">
                                        <IconUser class="icon" />
                                    </span>
                                    <span class="user-chip__text">
                                        <span class="user-chip__name">{user.display_name}</span>
                                        <span class="user-chip__role">"เจ้าหน้าที่"</span>
                                    </span>
                                </span>
                            }
                        })
                }}
                <button
                    class="button-ghost"
                    title="ประวัติการเบิก"
                    disabled=move || state.loading.get()
                    on:click=open_history
                >
                    <IconHistory class="icon" />
                    "ประวัติการเบิก"
                </button>
                <button
                    class="button-ghost"
                    title="ออกจากระบบ"
                    disabled=move || state.loading.get()
                    on:click=logout
                >
                    <IconLogOut class="icon" />
                    "ออกจากระบบ"
                </button>
            </div>
        </header>
    }
}

fn master_logo_markup() -> String {
    let suffix = LOGO_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let mut markup = MASTER_LOGO.to_string();
    for id in [
        "icon-bg",
        "icon-clip",
        "frogGrad",
        "bellyGrad",
        "pillPink",
        "pillBlue",
        "outer-shadow",
        "emboss-3d",
    ] {
        markup = markup.replace(id, &format!("{id}-{suffix}"));
    }
    markup
}

#[component]
pub fn BrandMark() -> impl IntoView {
    view! {
        <span class="brand-mark" aria-hidden="true" inner_html=master_logo_markup()></span>
    }
}
