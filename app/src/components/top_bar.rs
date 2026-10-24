use leptos::prelude::*;

use crate::api;
use crate::components::icons::{IconHistory, IconLogOut, IconSettings};
use crate::state::AppState;

#[component]
pub fn TopBar(state: AppState) -> impl IntoView {
    let health = state.health;
    let open_settings = {
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
                <BrandMark />
                <div class="top-bar__titles">
                    <span class="top-bar__title">"SnapHerb"</span>
                    <span class="top-bar__subtitle">"ระบบเบิกยาสมุนไพร"</span>
                </div>
            </div>
            <div class="top-bar__right">
                <span
                    class=move || format!("status-dot status-dot--{}", health.get().dot_class())
                    role="status"
                >
                    <span class="status-dot__dot" aria-hidden="true"></span>
                    <span class="status-dot__label">{move || health.get().label_th()}</span>
                </span>
                {move || {
                    state
                        .session
                        .get()
                        .map(|user| {
                            let initial = user
                                .display_name
                                .chars()
                                .next()
                                .or_else(|| user.login_name.chars().next())
                                .map(|c| c.to_string())
                                .unwrap_or_default();
                            let title = format!("{} · {}", user.display_name, user.login_name);
                            view! {
                                <span class="user-chip" title=title>
                                    <span class="user-chip__avatar">{initial}</span>
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
                <button
                    class="button-ghost"
                    title="ตั้งค่า"
                    disabled=move || state.loading.get()
                    on:click=open_settings
                >
                    <IconSettings class="icon" />
                    "ตั้งค่า"
                </button>
            </div>
        </header>
    }
}

#[component]
pub fn BrandMark() -> impl IntoView {
    view! {
        <svg class="brand-mark" viewBox="0 0 48 48" width="28" height="28" aria-hidden="true">
            <defs>
                <linearGradient id="snap-frog" x1="0%" y1="0%" x2="0%" y2="100%">
                    <stop offset="0%" stop-color="#86EFAC" />
                    <stop offset="50%" stop-color="#22C55E" />
                    <stop offset="100%" stop-color="#15803D" />
                </linearGradient>
                <linearGradient id="snap-belly" x1="0%" y1="0%" x2="0%" y2="100%">
                    <stop offset="0%" stop-color="#FEF08A" />
                    <stop offset="100%" stop-color="#FDE047" />
                </linearGradient>
                <linearGradient id="snap-pill-pink" x1="0%" y1="0%" x2="0%" y2="100%">
                    <stop offset="0%" stop-color="#FDA4AF" />
                    <stop offset="100%" stop-color="#F43F5E" />
                </linearGradient>
                <linearGradient id="snap-pill-blue" x1="0%" y1="0%" x2="0%" y2="100%">
                    <stop offset="0%" stop-color="#7DD3FC" />
                    <stop offset="100%" stop-color="#0284C7" />
                </linearGradient>
                <filter id="snap-emboss" x="-30%" y="-30%" width="160%" height="160%">
                    <feDropShadow dx="0" dy="0.47" stdDeviation="0.56" flood-color="#000000" flood-opacity="0.3" />
                    <feDropShadow dx="0" dy="-0.28" stdDeviation="0.38" flood-color="#FFFFFF" flood-opacity="0.4" />
                </filter>
                <clipPath id="snap-clip">
                    <rect width="48" height="48" rx="12.75" />
                </clipPath>
            </defs>
            <rect width="48" height="48" rx="12.75" fill="#ECFDF5" />
            <g clip-path="url(#snap-clip)">
                <g fill="url(#snap-frog)" filter="url(#snap-emboss)">
                    <circle cx="14.25" cy="13.875" r="6.75" />
                    <circle cx="33.75" cy="13.875" r="6.75" />
                </g>
                <ellipse cx="24" cy="24.375" rx="18" ry="14.0625" fill="url(#snap-frog)" filter="url(#snap-emboss)" />
                <g filter="url(#snap-emboss)">
                    <circle cx="14.25" cy="13.875" r="4.5" fill="#FFFFFF" />
                    <circle cx="33.75" cy="13.875" r="4.5" fill="#FFFFFF" />
                </g>
                <circle cx="14.625" cy="13.875" r="2.25" fill="#064E3B" />
                <circle cx="33.375" cy="13.875" r="2.25" fill="#064E3B" />
                <circle cx="13.6875" cy="12.9375" r="0.84" fill="#FFFFFF" />
                <circle cx="32.4375" cy="12.9375" r="0.84" fill="#FFFFFF" />
                <ellipse cx="24" cy="29.0625" rx="9.375" ry="6.5625" fill="url(#snap-belly)" />
                <ellipse cx="11.25" cy="25.3125" rx="2.25" ry="1.40625" fill="#F43F5E" opacity="0.35" />
                <ellipse cx="36.75" cy="25.3125" rx="2.25" ry="1.40625" fill="#F43F5E" opacity="0.35" />
                <ellipse cx="22.125" cy="22.6875" rx="0.375" ry="0.5625" fill="#064E3B" />
                <ellipse cx="25.875" cy="22.6875" rx="0.375" ry="0.5625" fill="#064E3B" />
                <path d="M 19.3125 25.3125 Q 24 29.25 28.6875 25.3125" stroke="#064E3B" stroke-width="1.125" fill="none" stroke-linecap="round" />
                <g filter="url(#snap-emboss)" transform="translate(21.09375, 36.09375) rotate(-22)">
                    <path d="M -2.0625 -4.21875 A 2.0625 2.0625 0 0 1 2.0625 -4.21875 L 2.0625 0 L -2.0625 0 Z" fill="url(#snap-pill-pink)" />
                    <path d="M -2.0625 0 L 2.0625 0 L 2.0625 4.21875 A 2.0625 2.0625 0 0 1 -2.0625 4.21875 Z" fill="#FFFFFF" />
                </g>
                <g filter="url(#snap-emboss)" transform="translate(26.90625, 36.09375) rotate(22)">
                    <path d="M -2.0625 -4.21875 A 2.0625 2.0625 0 0 1 2.0625 -4.21875 L 2.0625 0 L -2.0625 0 Z" fill="url(#snap-pill-blue)" />
                    <path d="M -2.0625 0 L 2.0625 0 L 2.0625 4.21875 A 2.0625 2.0625 0 0 1 -2.0625 4.21875 Z" fill="#FFFFFF" />
                </g>
                <circle cx="11.8125" cy="37.6875" r="5.0625" fill="url(#snap-frog)" filter="url(#snap-emboss)" />
                <circle cx="36.1875" cy="37.6875" r="5.0625" fill="url(#snap-frog)" filter="url(#snap-emboss)" />
            </g>
        </svg>
    }
}
