use leptos::prelude::*;

use crate::api::{self, ApiErrorKind};
use crate::components::icons::{IconArrowRight, IconLock, IconSettings};
use crate::components::top_bar::BrandMark;
use crate::state::{AppState, ConnectionHealth, SettingsTab};

#[component]
pub fn LoginScreen(state: AppState) -> impl IntoView {
    let username = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let login_busy = RwSignal::new(false);
    let login_error = RwSignal::new(None::<String>);

    let open_settings = {
        let state = state.clone();
        move |_| {
            state.settings_tab.set(SettingsTab::Connection);
            state.settings_open.set(true);
        }
    };

    let on_login = {
        let state = state.clone();
        move |ev: web_sys::SubmitEvent| {
            ev.prevent_default();
            let login_name = username.get();
            if login_name.trim().is_empty() {
                login_error.set(Some("กรุณากรอกชื่อผู้ใช้".to_string()));
                return;
            }
            let login_password = password.get();
            if login_password.is_empty() {
                login_error.set(Some("กรุณากรอกรหัสผ่าน".to_string()));
                return;
            }
            login_busy.set(true);
            login_error.set(None);
            let state = state.clone();
            leptos::task::spawn_local(async move {
                match api::login(login_name.trim(), &login_password).await {
                    Ok(user) => {
                        password.set(String::new());
                        state.finish_login(user);
                    }
                    Err(err) => {
                        if matches!(
                            err.kind,
                            ApiErrorKind::Connection | ApiErrorKind::NotConfigured
                        ) {
                            state.health.set(ConnectionHealth::Disconnected);
                            state.connection_error.set(Some(err.message.clone()));
                        }
                        login_error.set(Some(err.message));
                    }
                }
                login_busy.set(false);
            });
        }
    };

    view! {
        <div class="login">
            <aside class="login__brand">
                <div class="login__brand-top">
                    <BrandMark />
                    <div class="login__brand-titles">
                        <span class="login__brand-name">"SnapHerb"</span>
                        <span class="login__brand-sub">"ระบบเบิกยาสมุนไพร"</span>
                    </div>
                </div>

                <div class="login__stage">
                    <h1 class="login__headline">"ใบเบิกยาสมุนไพร"</h1>
                    <div class="login__art">
                        <span class="login__leaf login__leaf--one" aria-hidden="true"></span>
                        <span class="login__leaf login__leaf--two" aria-hidden="true"></span>
                        <div class="login__art-mark">
                            <BrandMark />
                        </div>
                        <span class="login__leaf login__leaf--three" aria-hidden="true"></span>
                    </div>
                    <div class="login__claims">
                        <span class="login__claim">"คัดตามแพทย์ผู้สั่งจ่าย"</span>
                        <span class="login__claim">"คำนวณจำนวนกล่องอัตโนมัติ"</span>
                        <span class="login__claim">"ล็อกการเบิกซ้ำ"</span>
                    </div>
                </div>

                <p class="login__brand-foot">
                    {let foot_state = state.clone(); move || {
                        foot_state
                            .settings
                            .get()
                            .map(|settings| {
                                format!(
                                    "{} · {}",
                                    settings.report.hospital_name,
                                    settings.report.department_name,
                                )
                            })
                            .unwrap_or_else(|| {
                                "โรงพยาบาลสระโบสถ์ · กลุ่มงานเภสัชกรรม".to_string()
                            })
                    }}
                </p>
            </aside>

            <main class="login__main">
                <div class="login__toolbar">
                    <button class="button-ghost" on:click=open_settings>
                        <IconSettings class="icon" />
                        "ตั้งค่า"
                    </button>
                </div>

                <div class="login__card">
                    <header class="login__card-head">
                        <h2 class="login__card-title">"เข้าสู่ระบบ"</h2>
                        <p class="login__card-sub">"ใช้บัญชีเจ้าหน้าที่ HOSxP"</p>
                    </header>

                    {move || {
                        (!state.configured.get())
                            .then(|| {
                                view! {
                                    <div class="login__notice" role="status">
                                        <div>
                                            <b>"ยังไม่ได้ตั้งค่าการเชื่อมต่อ HOSxP"</b>
                                            <small>
                                                "เปิดการตั้งค่าเพื่อเชื่อมต่อ HOSxP ก่อนเข้าสู่ระบบ"
                                            </small>
                                        </div>
                                        <button
                                            class="button-secondary"
                                            on:click=open_settings
                                        >
                                            "ตั้งค่าการเชื่อมต่อ"
                                        </button>
                                    </div>
                                }
                            })
                    }}

                    {move || {
                        match state.health.get() {
                            ConnectionHealth::Connected => {
                                Some(
                                    view! {
                                        <p class="login__connection login__connection--ok">
                                            <span class="status-dot__dot"></span>
                                            "เชื่อมต่อ HOSxP แล้ว"
                                        </p>
                                    }
                                        .into_any(),
                                )
                            }
                            ConnectionHealth::Disconnected => {
                                Some(
                                    view! {
                                        <div class="login__notice login__notice--error" role="alert">
                                            <div>
                                                <b>"การเชื่อมต่อ HOSxP ขัดข้อง"</b>
                                                <small>
                                                    {state
                                                        .connection_error
                                                        .get()
                                                        .unwrap_or_else(|| {
                                                            "กรุณาตรวจสอบการตั้งค่าการเชื่อมต่อและเครือข่าย"
                                                                .to_string()
                                                        })}
                                                </small>
                                            </div>
                                            <button
                                                class="button-secondary"
                                                on:click=open_settings
                                            >
                                                "ตรวจสอบการตั้งค่า"
                                            </button>
                                        </div>
                                    }
                                        .into_any(),
                                )
                            }
                            ConnectionHealth::Unconfigured => None,
                        }
                    }}

                    <form class="login__form" on:submit=on_login>
                        <label class="login__field">
                            <span>"ชื่อผู้ใช้"</span>
                            <input
                                class="text-input"
                                type="text"
                                autocomplete="username"
                                prop:value=move || username.get()
                                on:input=move |ev| username.set(event_target_value(&ev))
                            />
                        </label>
                        <label class="login__field">
                            <span>"รหัสผ่าน"</span>
                            <input
                                class="text-input"
                                type="password"
                                autocomplete="current-password"
                                prop:value=move || password.get()
                                on:input=move |ev| password.set(event_target_value(&ev))
                            />
                        </label>

                        {move || {
                            login_error
                                .get()
                                .map(|message| {
                                    view! {
                                        <p class="login__error" role="alert">
                                            <IconLock class="icon" />
                                            {message}
                                        </p>
                                    }
                                })
                        }}

                        <div class="login__actions">
                            <button
                                type="submit"
                                class="button-primary login__submit"
                                disabled=move || !state.configured.get() || login_busy.get()
                            >
                                <IconArrowRight class="icon" />
                                {move || {
                                    if login_busy.get() {
                                        "กำลังเข้าสู่ระบบ..."
                                    } else {
                                        "เข้าสู่ระบบ"
                                    }
                                }}
                            </button>
                        </div>
                    </form>
                </div>
            </main>
        </div>
    }
}

#[component]
pub fn StartupSplash() -> impl IntoView {
    view! {
        <div class="startup">
            <BrandMark />
            <p>"กำลังเตรียมระบบ..."</p>
        </div>
    }
}
