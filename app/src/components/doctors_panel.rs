use leptos::prelude::*;
use snap_herb_models::DoctorRef;

use crate::api;
use crate::components::icons::{IconPlus, IconSearch, IconTrash, IconUser};
use crate::components::settings_modal::{StatusLine, StatusMessage};
use crate::state::AppState;

#[component]
pub fn DoctorsPanel(state: AppState) -> impl IntoView {
    let draft = RwSignal::new(
        state
            .settings
            .get_untracked()
            .map(|settings| settings.doctors)
            .unwrap_or_default(),
    );
    let term = RwSignal::new(String::new());
    let results = RwSignal::new(Vec::<DoctorRef>::new());
    let searching = RwSignal::new(false);
    let manual_code = RwSignal::new(String::new());
    let manual_name = RwSignal::new(String::new());
    let status = RwSignal::new(None::<StatusMessage>);
    let busy = RwSignal::new(false);
    let dirty = RwSignal::new(false);

    let add_doctor = move |doctor: DoctorRef| {
        let code = doctor.code.trim().to_string();
        let name = doctor.name.trim().to_string();
        if code.is_empty() || name.is_empty() {
            status.set(Some(StatusMessage::error(
                "รหัสและชื่อแพทย์ต้องไม่ว่าง".to_string(),
            )));
            return;
        }
        if draft
            .get_untracked()
            .iter()
            .any(|existing| existing.code == code)
        {
            status.set(Some(StatusMessage::error(format!(
                "แพทย์รหัส {code} ถูกเลือกอยู่แล้ว"
            ))));
            return;
        }
        draft.update(|doctors| doctors.push(DoctorRef::new(code, name)));
        dirty.set(true);
        status.set(None);
    };

    let on_search = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let value = term.get();
        if value.trim().chars().count() < 2 {
            status.set(Some(StatusMessage::error(
                "พิมพ์อย่างน้อย 2 ตัวอักษรเพื่อค้นหาแพทย์".to_string(),
            )));
            return;
        }
        searching.set(true);
        status.set(None);
        leptos::task::spawn_local(async move {
            match api::search_doctors(&value).await {
                Ok(hits) => results.set(hits),
                Err(err) => {
                    results.set(Vec::new());
                    status.set(Some(StatusMessage::error(err.message)));
                }
            }
            searching.set(false);
        });
    };

    let on_manual_add = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let code = manual_code.get_untracked().trim().to_string();
        let name = manual_name.get_untracked().trim().to_string();
        if code.is_empty() || name.is_empty() {
            status.set(Some(StatusMessage::error(
                "กรุณากรอกรหัสและชื่อแพทย์".to_string(),
            )));
            return;
        }
        add_doctor(DoctorRef::new(code, name));
        manual_code.set(String::new());
        manual_name.set(String::new());
    };

    let on_remove = move |code: String| {
        draft.update(|doctors| doctors.retain(|doctor| doctor.code != code));
        dirty.set(true);
        status.set(None);
    };

    let on_save = {
        let state = state.clone();
        move |_| {
            let doctors = draft.get_untracked();
            busy.set(true);
            status.set(None);
            let state = state.clone();
            leptos::task::spawn_local(async move {
                match api::save_doctors(&doctors).await {
                    Ok(()) => {
                        state.settings.update(|settings| {
                            if let Some(settings) = settings {
                                settings.doctors = doctors;
                            }
                        });
                        state.clear_requisition();
                        dirty.set(false);
                        status.set(Some(StatusMessage::ok("บันทึกแพทย์ที่ตรวจจับแล้ว")));
                    }
                    Err(err) => status.set(Some(StatusMessage::error(err.message))),
                }
                busy.set(false);
            });
        }
    };

    let on_reset = {
        let state = state.clone();
        move |_| {
            draft.set(
                state
                    .settings
                    .get_untracked()
                    .map(|settings| settings.doctors)
                    .unwrap_or_default(),
            );
            results.set(Vec::new());
            dirty.set(false);
            status.set(None);
        }
    };

    view! {
        <div class="panel">
            <h3 class="panel__subtitle">"แพทย์ที่ต้องการตรวจจับการสั่งจ่ายยา"</h3>
            <p class="panel__note">
                "ระบบจะนับเฉพาะรายการยาที่สั่งจ่ายโดยแพทย์ในรายการนี้ จากฟิลด์ doctor ของ opitemrece ใน HOSxP"
            </p>

            <form class="picker" on:submit=on_search>
                <span class="panel__label">"ค้นหาแพทย์จาก HOSxP"</span>
                <div class="picker__search">
                    <span class="picker__icon">
                        <IconSearch class="icon" />
                    </span>
                    <input
                        class="text-input"
                        type="search"
                        placeholder="พิมพ์ชื่อหรือรหัสแพทย์ (อย่างน้อย 2 ตัวอักษร)"
                        prop:value=move || term.get()
                        on:input=move |ev| term.set(event_target_value(&ev))
                    />
                    <button class="button-secondary" type="submit" disabled=move || searching.get()>
                        {move || if searching.get() { "กำลังค้นหา..." } else { "ค้นหา" }}
                    </button>
                </div>
                <ul class="picker__results">
                    {move || {
                        results
                            .get()
                            .into_iter()
                            .map(|doctor| {
                                let selected = draft
                                    .get()
                                    .iter()
                                    .any(|existing| existing.code == doctor.code);
                                let add = doctor.clone();
                                view! {
                                    <li class="picker__result">
                                        <div class="picker__result-info">
                                            <span class="picker__result-name">
                                                {doctor.name.clone()}
                                            </span>
                                            <span class="picker__result-meta">
                                                <span class="badge-icode">
                                                    {doctor.code.clone()}
                                                </span>
                                            </span>
                                        </div>
                                        <button
                                            class="button-ghost button-ghost--inline"
                                            type="button"
                                            disabled=selected
                                            on:click=move |_| add_doctor(add.clone())
                                        >
                                            <IconPlus class="icon" />
                                            {if selected { "เลือกแล้ว" } else { "เพิ่ม" }}
                                        </button>
                                    </li>
                                }
                            })
                            .collect_view()
                    }}
                </ul>
            </form>

            <form class="manual-row" on:submit=on_manual_add>
                <span class="panel__label">"หรือเพิ่มด้วยรหัสแพทย์เอง"</span>
                <div class="manual-row__fields">
                    <input
                        class="text-input"
                        type="text"
                        placeholder="รหัสแพทย์"
                        prop:value=move || manual_code.get()
                        on:input=move |ev| manual_code.set(event_target_value(&ev))
                    />
                    <input
                        class="text-input"
                        type="text"
                        placeholder="ชื่อแพทย์"
                        prop:value=move || manual_name.get()
                        on:input=move |ev| manual_name.set(event_target_value(&ev))
                    />
                    <button class="button-secondary" type="submit">
                        <IconPlus class="icon" />
                        "เพิ่ม"
                    </button>
                </div>
            </form>

            <div class="selected-list">
                <div class="selected-list__head">
                    <span class="panel__label">
                        {move || format!("แพทย์ที่เลือก {} คน", draft.get().len())}
                    </span>
                </div>
                {move || {
                    let doctors = draft.get();
                    if doctors.is_empty() {
                        return view! {
                            <p class="dialog__empty">
                                "ยังไม่ได้เลือกแพทย์ ระบบจะไม่สามารถดึงข้อมูลการใช้ยาได้"
                            </p>
                        }
                            .into_any();
                    }
                    doctors
                        .into_iter()
                        .map(|doctor| {
                            let code = doctor.code.clone();
                            view! {
                                <div class="selected-row">
                                    <span class="selected-row__icon">
                                        <IconUser class="icon" />
                                    </span>
                                    <span class="selected-row__text">
                                        <span class="selected-row__name">{doctor.name}</span>
                                        <span class="selected-row__meta">{doctor.code}</span>
                                    </span>
                                    <button
                                        class="button-icon-circular button-icon-circular--small"
                                        aria-label="ลบแพทย์"
                                        on:click=move |_| on_remove(code.clone())
                                    >
                                        <IconTrash class="icon" />
                                    </button>
                                </div>
                            }
                        })
                        .collect_view()
                        .into_any()
                }}
            </div>

            <StatusLine message=status />
            <div class="panel__actions">
                <span class="panel__dirty">
                    {move || dirty.get().then_some("มีการแก้ไขที่ยังไม่บันทึก")}
                </span>
                <button
                    class="button-secondary"
                    disabled=move || busy.get() || !dirty.get()
                    on:click=on_reset
                >
                    "ยกเลิกการแก้ไข"
                </button>
                <button
                    class="button-accent"
                    disabled=move || busy.get() || !dirty.get()
                    on:click=on_save
                >
                    "บันทึกแพทย์ที่ตรวจจับ"
                </button>
            </div>
        </div>
    }
}
