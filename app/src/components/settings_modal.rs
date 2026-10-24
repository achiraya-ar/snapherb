use leptos::prelude::*;

use crate::components::approval_panel::ApprovalPanel;
use crate::components::catalog_panel::CatalogPanel;
use crate::components::connection_panel::ConnectionPanel;
use crate::components::doctors_panel::DoctorsPanel;
use crate::components::icons::{
    IconDatabase, IconFileText, IconLeaf, IconStethoscope, IconUser, IconX,
};
use crate::components::report_panel::ReportPanel;
use crate::state::{AppState, SettingsTab};

#[derive(Debug, Clone)]
pub struct StatusMessage {
    pub ok: bool,
    pub text: String,
}

impl StatusMessage {
    pub fn ok(text: impl Into<String>) -> Self {
        Self {
            ok: true,
            text: text.into(),
        }
    }

    pub fn error(text: impl Into<String>) -> Self {
        Self {
            ok: false,
            text: text.into(),
        }
    }
}

#[component]
pub fn StatusLine(message: RwSignal<Option<StatusMessage>>) -> impl IntoView {
    view! {
        {move || {
            message
                .get()
                .map(|message| {
                    let class = if message.ok {
                        "status-line status-line--ok"
                    } else {
                        "status-line status-line--error"
                    };
                    view! { <p class=class>{message.text}</p> }
                })
        }}
    }
}

const TABS: [(SettingsTab, &str); 5] = [
    (SettingsTab::Connection, "การเชื่อมต่อ"),
    (SettingsTab::Doctors, "แพทย์ที่ตรวจจับ"),
    (SettingsTab::Catalog, "ยาสมุนไพร"),
    (SettingsTab::Approvals, "ผู้ลงนาม"),
    (SettingsTab::Report, "หัวใบเบิก"),
];

fn tab_icon(tab: SettingsTab) -> AnyView {
    match tab {
        SettingsTab::Connection => view! { <IconDatabase class="icon" /> }.into_any(),
        SettingsTab::Doctors => view! { <IconStethoscope class="icon" /> }.into_any(),
        SettingsTab::Catalog => view! { <IconLeaf class="icon" /> }.into_any(),
        SettingsTab::Approvals => view! { <IconUser class="icon" /> }.into_any(),
        SettingsTab::Report => view! { <IconFileText class="icon" /> }.into_any(),
    }
}

#[component]
pub fn SettingsModal(state: AppState) -> impl IntoView {
    let _escape = window_event_listener(leptos::ev::keydown, {
        let state = state.clone();
        move |ev| {
            if ev.key() == "Escape" {
                state.settings_open.set(false);
            }
        }
    });

    view! {
        {move || {
            if !state.settings_open.get() {
                return None;
            }
            let close_backdrop = {
                let state = state.clone();
                move |_| state.settings_open.set(false)
            };
            let close_button = {
                let state = state.clone();
                move |_| state.settings_open.set(false)
            };
            let stop_propagation = move |ev: web_sys::MouseEvent| ev.stop_propagation();
            let nav_state = state.clone();
            let body_state = state.clone();
            let footer_state = state.clone();
            let signed_in = Memo::new({
                let state = state.clone();
                move |_| state.session.get().is_some()
            });
            let nav_items = move || {
                TABS
                    .into_iter()
                    .filter(|(tab, _)| signed_in.get() || *tab == SettingsTab::Connection)
                    .map(|(tab, label)| {
                        let active = Memo::new({
                            let state = nav_state.clone();
                            move |_| state.settings_tab.get() == tab
                        });
                        let on_click = {
                            let state = nav_state.clone();
                            move |_| state.settings_tab.set(tab)
                        };
                        view! {
                            <button
                                class=move || if active.get() {
                                    "settings-nav__item settings-nav__item--active"
                                } else {
                                    "settings-nav__item"
                                }
                                role="tab"
                                aria-selected=move || active.get().to_string()
                                on:click=on_click
                            >
                                {tab_icon(tab)}
                                <span>{label}</span>
                            </button>
                        }
                    })
                    .collect_view()
            };
            Some(
                view! {
                    <div class="modal-backdrop" on:click=close_backdrop>
                        <div
                            class="settings-modal"
                            role="dialog"
                            aria-modal="true"
                            aria-label="ตั้งค่า"
                            on:click=stop_propagation
                        >
                            <header class="settings-modal__header">
                                <h2 class="settings-modal__title">
                                    {move || {
                                        if signed_in.get() {
                                            "ตั้งค่า SnapHerb"
                                        } else {
                                            "ตั้งค่าการเชื่อมต่อ"
                                        }
                                    }}
                                </h2>
                                <button
                                    class="button-icon-circular"
                                    aria-label="ปิด"
                                    on:click=close_button
                                >
                                    <IconX class="icon" />
                                </button>
                            </header>
                            <div class="settings-layout">
                                <nav class="settings-nav" role="tablist">
                                    {nav_items}
                                </nav>
                                <div class="settings-content">
                                    {move || {
                                        let tab = if body_state.session.get().is_some() {
                                            body_state.settings_tab.get()
                                        } else {
                                            SettingsTab::Connection
                                        };
                                        match tab {
                                            SettingsTab::Connection => {
                                                view! { <ConnectionPanel state=body_state.clone() /> }
                                                    .into_any()
                                            }
                                            SettingsTab::Doctors => {
                                                view! { <DoctorsPanel state=body_state.clone() /> }
                                                    .into_any()
                                            }
                                            SettingsTab::Catalog => {
                                                view! { <CatalogPanel state=body_state.clone() /> }
                                                    .into_any()
                                            }
                                            SettingsTab::Approvals => {
                                                view! { <ApprovalPanel state=body_state.clone() /> }
                                                    .into_any()
                                            }
                                            SettingsTab::Report => {
                                                view! { <ReportPanel state=body_state.clone() /> }
                                                    .into_any()
                                            }
                                        }
                                    }}
                                </div>
                            </div>
                            {move || {
                                footer_state
                                    .session
                                    .get()
                                    .is_some()
                                    .then(|| {
                                        view! { <SettingsFooter state=footer_state.clone() /> }
                                    })
                            }}
                        </div>
                    </div>
                },
            )
        }}
    }
}

#[component]
fn SettingsFooter(state: AppState) -> impl IntoView {
    let status = RwSignal::new(None::<StatusMessage>);
    let file_input = NodeRef::<leptos::html::Input>::new();

    let on_export = {
        let state = state.clone();
        move |_| {
            let Some(settings) = state.settings.get() else {
                return;
            };
            let json = match crate::settings_io::export_json(&settings) {
                Ok(json) => json,
                Err(message) => {
                    status.set(Some(StatusMessage::error(message)));
                    return;
                }
            };
            let file_name = crate::settings_io::export_file_name();
            status.set(None);
            leptos::task::spawn_local(async move {
                match crate::api::export_settings_to_file(&json, &file_name).await {
                    Ok(Some(path)) => status.set(Some(StatusMessage::ok(format!(
                        "บันทึกไฟล์ตั้งค่าแล้ว: {path} (ไม่รวมการเชื่อมต่อและรหัสผ่าน)"
                    )))),
                    Ok(None) => status.set(Some(StatusMessage::ok("ยกเลิกการส่งออก"))),
                    Err(err) => status.set(Some(StatusMessage::error(err.message))),
                }
            });
        }
    };

    let open_file_picker = move |_: web_sys::MouseEvent| {
        if let Some(input) = file_input.get() {
            input.click();
        }
    };

    let on_import_file = {
        let state = state.clone();
        move |ev| {
            let input = event_target::<web_sys::HtmlInputElement>(&ev);
            let Some(file) = input.files().and_then(|files| files.get(0)) else {
                return;
            };
            status.set(None);
            let state = state.clone();
            leptos::task::spawn_local(async move {
                let message = match wasm_bindgen_futures::JsFuture::from(file.text()).await {
                    Ok(value) => match value.as_string() {
                        Some(raw) => match crate::settings_io::parse_import(&raw) {
                            Ok(settings) => match persist_import(&state, &settings).await {
                                Ok(summary) => StatusMessage::ok(summary),
                                Err(message) => StatusMessage::error(message),
                            },
                            Err(message) => StatusMessage::error(message),
                        },
                        None => StatusMessage::error("อ่านไฟล์ไม่สำเร็จ"),
                    },
                    Err(_) => StatusMessage::error("อ่านไฟล์ไม่สำเร็จ"),
                };
                status.set(Some(message));
                input.set_value("");
            });
        }
    };

    view! {
        <footer class="settings-modal__footer">
            <div class="settings-modal__io">
                <button class="button-secondary" on:click=on_export>
                    "ส่งออกการตั้งค่า"
                </button>
                <button class="button-secondary" on:click=open_file_picker>
                    "นำเข้าการตั้งค่า"
                </button>
                <input
                    node_ref=file_input
                    class="settings-modal__file"
                    type="file"
                    accept="application/json,.json"
                    on:change=on_import_file
                    tabindex="-1"
                />
            </div>
            <p class="settings-modal__io-note">
                "ไฟล์ส่งออกเป็น JSON (แพทย์ ยาสมุนไพร ผู้ลงนาม และหัวใบเบิก) สำหรับใช้กับเครื่องอื่น ส่วนการเชื่อมต่อและรหัสผ่านต้องตั้งค่าในแต่ละเครื่อง"
            </p>
            <StatusLine message=status />
        </footer>
    }
}

async fn persist_import(
    state: &AppState,
    settings: &snap_herb_models::AppSettings,
) -> Result<String, String> {
    crate::api::save_doctors(&settings.doctors)
        .await
        .map_err(|err| err.message)?;
    crate::api::save_catalog(&settings.catalog)
        .await
        .map_err(|err| err.message)?;
    crate::api::save_report(&settings.report)
        .await
        .map_err(|err| err.message)?;
    crate::api::save_approvals(&settings.approvals)
        .await
        .map_err(|err| err.message)?;
    state.settings.set(Some(settings.clone()));
    state.clear_requisition();
    Ok(format!(
        "นำเข้าการตั้งค่าแล้ว: {} แพทย์, {} รายการยา",
        settings.doctors.len(),
        settings.catalog.drugs.len()
    ))
}
