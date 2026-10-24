use std::time::Duration;

use chrono::{Local, NaiveDate};
use leptos::prelude::*;
use snap_herb_models::ReportRow;

use crate::api::{self, ApiErrorKind};
use crate::components::icons::{IconCheck, IconPlus, IconPrinter, IconRefresh};
use crate::state::{AppState, Banner, ConfirmedRequisition, latest_selectable_date};

const PRINT_DELAY: Duration = Duration::from_millis(80);

fn fire_print() {
    if let Some(window) = web_sys::window() {
        let _ = window.print();
    }
}

#[component]
pub fn CommandBar(state: AppState) -> impl IntoView {
    let start_value = move || state.start_date.get().format("%Y-%m-%d").to_string();
    let end_value = move || state.end_date.get().format("%Y-%m-%d").to_string();
    let max_date = latest_selectable_date().format("%Y-%m-%d").to_string();

    let on_start_change = {
        let state = state.clone();
        move |ev| {
            let raw = event_target_value(&ev);
            if let Ok(date) = NaiveDate::parse_from_str(&raw, "%Y-%m-%d") {
                if date > latest_selectable_date() {
                    state
                        .banner
                        .set(Some(Banner::warn("เลือกช่วงวันที่ได้ถึงวันที่ก่อนหน้าวันที่ปัจจุบันเท่านั้น")));
                    return;
                }
                state.start_date.set(date);
                state.clear_requisition();
                if date > state.end_date.get_untracked() {
                    state
                        .banner
                        .set(Some(Banner::warn("วันที่เริ่มต้องไม่หลังวันที่สิ้นสุด")));
                } else {
                    state.banner.set(None);
                }
            }
        }
    };
    let on_end_change = {
        let state = state.clone();
        move |ev| {
            let raw = event_target_value(&ev);
            if let Ok(date) = NaiveDate::parse_from_str(&raw, "%Y-%m-%d") {
                if date > latest_selectable_date() {
                    state
                        .banner
                        .set(Some(Banner::warn("เลือกช่วงวันที่ได้ถึงวันที่ก่อนหน้าวันที่ปัจจุบันเท่านั้น")));
                    return;
                }
                state.end_date.set(date);
                state.clear_requisition();
                if date < state.start_date.get_untracked() {
                    state
                        .banner
                        .set(Some(Banner::warn("วันที่สิ้นสุดต้องไม่ก่อนวันที่เริ่ม")));
                } else {
                    state.banner.set(None);
                }
            }
        }
    };

    let on_fetch = {
        let state = state.clone();
        move |_| state.fetch_usage_action()
    };
    let on_open_add = {
        let state = state.clone();
        move |_| {
            state.banner.set(None);
            state.add_drug_open.set(true);
        }
    };
    let on_open_confirm = {
        let state = state.clone();
        move |_| {
            state.banner.set(None);
            state.confirm_open.set(true);
        }
    };
    let on_print = {
        let state = state.clone();
        move |_| {
            if state.current_record.get_untracked().is_some() {
                fire_print();
                return;
            }
            if !state.print_snapshot_matches() {
                state.invalidate_confirmation();
                state
                    .banner
                    .set(Some(Banner::warn("ข้อมูลเปลี่ยนแล้ว กรุณายืนยันข้อมูลเบิกใหม่ก่อนพิมพ์")));
                return;
            }
            let (Some(period), Some(rows), Some(settings)) = (
                state.period(),
                state.rows.get_untracked(),
                state.settings.get_untracked(),
            ) else {
                return;
            };
            let report = settings.report;
            state.loading.set(true);
            state.banner.set(None);
            let state = state.clone();
            leptos::task::spawn_local(async move {
                match api::save_and_lock_requisition(&period, &rows, &report).await {
                    Ok(record) => {
                        state.load_history_action();
                        let still_same = state.session.get_untracked().is_some()
                            && state.period() == Some(period);
                        if !still_same {
                            state.loading.set(false);
                            return;
                        }
                        state.rows.set(Some(record.rows.clone()));
                        state.current_record.set(Some(record.clone()));
                        state.confirmed.set(Some(ConfirmedRequisition {
                            rows: record.rows.clone(),
                            period,
                            report: record.report.clone(),
                            approvals: record.approvals.clone(),
                            confirmed_at: Local::now().naive_local(),
                        }));
                        state.set_info_banner(format!(
                            "บันทึกและล็อกช่วงวันที่เบิกแล้ว ใบเบิกเลขที่ {}",
                            record.document_no
                        ));
                        state.loading.set(false);
                        let _ = set_timeout_with_handle(fire_print, PRINT_DELAY);
                    }
                    Err(err) => {
                        state.loading.set(false);
                        if err.kind == ApiErrorKind::Locked {
                            state.confirmed.set(None);
                            state.current_record.set(None);
                            state.banner.set(Some(Banner::warn(err.message.clone())));
                        } else {
                            state.banner.set(Some(Banner::error(err.message)));
                        }
                    }
                }
            });
        }
    };

    view! {
        <div class="command-bar">
            <div class="command-bar__main">
                <div class="command-bar__run">
                    <label class="run-field">
                        <span class="run-field__label">"วันที่เริ่ม"</span>
                        <input
                            class="text-input text-input--compact run-field__input"
                            type="date"
                            max=max_date.clone()
                            prop:value=start_value
                            disabled=move || state.loading.get()
                            on:change=on_start_change
                        />
                    </label>
                    <label class="run-field">
                        <span class="run-field__label">"วันที่สิ้นสุด"</span>
                        <input
                            class="text-input text-input--compact run-field__input"
                            type="date"
                            max=max_date
                            prop:value=end_value
                            disabled=move || state.loading.get()
                            on:change=on_end_change
                        />
                    </label>
                    <button
                        class={let class_state = state.clone(); move || {
                            if class_state.rows.get().is_none() {
                                "button-primary"
                            } else {
                                "button-secondary"
                            }
                        }}
                        disabled={let fetch_state = state.clone(); move || {
                            fetch_state.period().is_none() || fetch_state.loading.get()
                        }}
                        on:click=on_fetch
                    >
                        <IconRefresh class="icon" />
                        {let label_state = state.clone(); move || {
                            if label_state.loading.get() {
                                "กำลังดึงข้อมูล..."
                            } else if label_state.rows.get().is_some() {
                                "ดึงใหม่"
                            } else {
                                "ดึงข้อมูลการใช้ยา"
                            }
                        }}
                    </button>
                </div>

                <div class="command-bar__actions">
                    <div class="command-bar__buttons">
                        <button
                            class="button-secondary"
                            disabled={let add_state = state.clone(); move || {
                                add_state.rows.get().is_none()
                                    || add_state.loading.get()
                                    || add_state.current_record.get().is_some()
                            }}
                            on:click=on_open_add
                        >
                            <IconPlus class="icon" />
                            "เพิ่มรายการยา"
                        </button>
                        {let confirm_state = state.clone(); move || {
                            confirm_state.rows.get()?;
                            if let Some(confirmed) = confirm_state.confirmed.get() {
                                Some(
                                    view! {
                                        <span class="confirm-chip" title="ยืนยันข้อมูลเบิกแล้ว">
                                            <IconCheck class="icon" />
                                            "ยืนยันแล้ว "
                                            {confirmed.confirmed_at.format("%H:%M").to_string()}
                                        </span>
                                    }
                                        .into_any(),
                                )
                            } else {
                                let has_rows = confirm_state
                                    .rows
                                    .get()
                                    .is_some_and(|rows: Vec<ReportRow>| !rows.is_empty());
                                let busy = confirm_state.loading.get();
                                Some(
                                    view! {
                                        <button
                                            class="button-accent"
                                            disabled=!has_rows || busy
                                            on:click=on_open_confirm
                                        >
                                            <IconCheck class="icon" />
                                            "ยืนยันข้อมูลเบิก"
                                        </button>
                                    }
                                        .into_any(),
                                )
                            }
                        }}
                        <button
                            class="button-primary"
                            disabled={let print_state = state.clone(); move || {
                                print_state.confirmed.get().is_none() || print_state.loading.get()
                            }}
                            title="ต้องยืนยันก่อนพิมพ์ ระบบจะพิมพ์ 2 ฟอร์ม และล็อกช่วงวันที่ถาวร"
                            on:click=on_print
                        >
                            <IconPrinter class="icon" />
                            {let print_label_state = state.clone(); move || {
                                if print_label_state.current_record.get().is_some() {
                                    "พิมพ์ซ้ำ"
                                } else {
                                    "พิมพ์และล็อกช่วงวันที่"
                                }
                            }}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}
