use leptos::prelude::*;
use snap_herb_models::{RequisitionRecord, format_qty, format_thai_datetime_full};

use crate::components::icons::{IconArrowLeft, IconHistory, IconPrinter, IconX};
use crate::state::AppState;

#[component]
pub fn HistoryModal(state: AppState) -> impl IntoView {
    view! {
        {move || {
            if !state.history_open.get() {
                return None;
            }
            let close = {
                let state = state.clone();
                move |_| state.history_open.set(false)
            };
            let stop_propagation = move |ev: web_sys::MouseEvent| ev.stop_propagation();
            let body_state = state.clone();
            Some(
                view! {
                    <div class="modal-backdrop" on:click=close>
                        <div
                            class="settings-modal history-modal"
                            role="dialog"
                            aria-modal="true"
                            aria-label="ประวัติการเบิก"
                            on:click=stop_propagation
                        >
                            <header class="settings-modal__header">
                                <div class="panel-head">
                                    <span class="panel-head__icon">
                                        <IconHistory class="icon" />
                                    </span>
                                    <div>
                                        <h2 class="settings-modal__title">"ประวัติการเบิก"</h2>
                                        <p class="panel__note">
                                            "ใบเบิกที่สร้างแล้วจะถูกล็อกถาวร ไม่สามารถแก้ไขหรือเบิกซ้ำได้ แต่ยังพิมพ์ซ้ำได้จากที่นี่"
                                        </p>
                                    </div>
                                </div>
                                <button class="button-icon-circular" aria-label="ปิด" on:click=close>
                                    <IconX class="icon" />
                                </button>
                            </header>

                            <div class="history-body">
                                {move || {
                                    let records = body_state.history.get();
                                    if body_state.history_loading.get() {
                                        return view! {
                                            <p class="dialog__empty">"กำลังโหลดประวัติ..."</p>
                                        }
                                            .into_any();
                                    }
                                    if records.is_empty() {
                                        return view! {
                                            <div class="empty-inline">
                                                <IconHistory class="icon icon--large" />
                                                <p>"ยังไม่มีประวัติการเบิกในเครื่องนี้"</p>
                                            </div>
                                        }
                                            .into_any();
                                    }
                                    records
                                        .into_iter()
                                        .map(|record| {
                                            let reprint_record = record.clone();
                                            let on_reprint = {
                                                let state = body_state.clone();
                                                move |_| {
                                                    state.reprint.set(Some(reprint_record.clone()));
                                                    state.history_open.set(false);
                                                }
                                            };
                                            view! {
                                                <article class="history-row">
                                                    <div class="history-row__main">
                                                        <div class="history-row__identity">
                                                            <span class="history-row__doc">
                                                                {record.document_no.clone()}
                                                            </span>
                                                            <span class="status-badge status-badge--locked">
                                                                "ล็อก"
                                                            </span>
                                                        </div>
                                                        <p class="history-row__period">
                                                            {record.period.thai_range()}
                                                        </p>
                                                        <p class="history-row__meta">
                                                            {format!(
                                                                "พิมพ์เมื่อ {} · {} รายการ · {} หน่วยเบิก · ผู้เบิก {}",
                                                                format_thai_datetime_full(record.printed_at),
                                                                record.rows.len(),
                                                                format_qty(record.total_requested()),
                                                                record.requested_by,
                                                            )}
                                                        </p>
                                                    </div>
                                                    <div class="history-row__actions">
                                                        <button
                                                            class="button-secondary"
                                                            on:click=on_reprint
                                                        >
                                                            <IconPrinter class="icon" />
                                                            "ดู / พิมพ์ซ้ำ"
                                                        </button>
                                                    </div>
                                                </article>
                                            }
                                        })
                                        .collect_view()
                                        .into_any()
                                }}
                            </div>
                        </div>
                    </div>
                },
            )
        }}
    }
}

#[component]
pub fn ReprintBar(state: AppState, record: RequisitionRecord) -> impl IntoView {
    let close_state = state.clone();
    let close = move |_| close_state.reprint.set(None);
    let print = move |_| {
        if let Some(window) = web_sys::window() {
            let _ = window.print();
        }
    };

    view! {
        <div class="reprint-bar">
            <div class="reprint-bar__info">
                <span class="reprint-bar__icon">
                    <IconPrinter class="icon" />
                </span>
                <div>
                    <div class="reprint-bar__title">
                        "กำลังดูใบเบิกที่บันทึกแล้ว "
                        <span class="history-row__doc">{record.document_no.clone()}</span>
                        <span class="status-badge status-badge--locked">"ล็อก"</span>
                    </div>
                    <p class="reprint-bar__meta">
                        {format!(
                            "ช่วงวันที่ {} · พิมพ์เมื่อ {} · ผู้เบิก {}",
                            record.period.thai_range(),
                            format_thai_datetime_full(record.printed_at),
                            record.requested_by,
                        )}
                    </p>
                </div>
            </div>
            <div class="reprint-bar__actions">
                <button class="button-secondary" on:click=close>
                    <IconArrowLeft class="icon" />
                    "กลับไปสร้างใบเบิก"
                </button>
                <button class="button-primary" on:click=print>
                    <IconPrinter class="icon" />
                    "พิมพ์ซ้ำ"
                </button>
            </div>
        </div>
    }
}
