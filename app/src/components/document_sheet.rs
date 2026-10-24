use std::time::Duration;

use chrono::{Local, NaiveDateTime};
use leptos::prelude::*;
use snap_herb_core::{ROWS_PER_PAGE, SheetPage, paginate_rows};
use snap_herb_models::{
    ApprovalSettings, ReportMeta, ReportRow, RequisitionPeriod, format_thai_date,
    format_thai_datetime_full,
};

use crate::components::icons::IconX;
use crate::components::top_bar::BrandMark;
use crate::state::{AppState, BannerKind, PendingEdit};

#[derive(Debug, Clone, PartialEq)]
pub struct SheetData {
    pub rows: Vec<ReportRow>,
    pub period: Option<RequisitionPeriod>,
    pub report: ReportMeta,
    pub approvals: ApprovalSettings,
    pub document_no: Option<String>,
    pub printed_at: Option<NaiveDateTime>,
    pub requested_by: Option<String>,
    pub editable: bool,
}

#[component]
pub fn DocumentSheet(state: AppState, data: SheetData) -> impl IntoView {
    let adjustments: Vec<(String, String)> = data
        .rows
        .iter()
        .filter(|row| row.has_adjustment())
        .map(|row| (row.name.clone(), row.reason.trim().to_string()))
        .collect();
    let mut pages = paginate_rows(&data.rows, ROWS_PER_PAGE);
    if pages.is_empty() {
        pages.push(SheetPage {
            number: 1,
            count: 1,
            is_last: true,
            rows: Vec::new(),
        });
    }

    view! {
        {pages
            .into_iter()
            .map(|page| {
                view! {
                    <SheetPageView
                        state=state.clone()
                        page=page
                        data=data.clone()
                        adjustments=adjustments.clone()
                    />
                }
            })
            .collect_view()}
    }
}

#[component]
fn SheetPageView(
    state: AppState,
    page: SheetPage,
    data: SheetData,
    adjustments: Vec<(String, String)>,
) -> impl IntoView {
    let is_last = page.is_last;
    let page_label = format!("หน้า {} / {}", page.number, page.count);
    let has_rows = !page.rows.is_empty();
    let editable = data.editable;
    let document_no = data.document_no.clone();
    let printed_at = data.printed_at;
    let period = data.period;
    let report = data.report.clone();
    let approvals = data.approvals.clone();
    let requested_by = data.requested_by.clone();
    let signature_date = printed_at
        .map(|printed_at| printed_at.date())
        .unwrap_or_else(|| Local::now().date_naive());
    let signature_date_text = format!("วันที่ {}", format_thai_date(signature_date));

    view! {
        <article class="sheet">
            <header class="sheet__header">
                <div class="sheet__logo">
                    <BrandMark />
                </div>
                <div class="sheet__heading">
                    <h1 class="sheet__hospital">{report.hospital_name}</h1>
                    <h2 class="sheet__title">{report.document_title}</h2>
                    <p class="sheet__department">{report.department_name}</p>
                </div>
            </header>

            <p class="sheet__period">
                {document_no
                    .map(|no| format!("เลขที่ใบเบิก {no} · "))
                    .unwrap_or_default()}
                "ช่วงวันที่เบิก: "
                {period
                    .map(|period| {
                        format!(
                            "{} ถึง {}",
                            format_thai_date(period.start_date),
                            format_thai_date(period.end_date)
                        )
                    })
                    .unwrap_or_default()}
            </p>

            <table class="sheet__table sheet__table--herb">
                <colgroup>
                    <col class="sheet__col-no" />
                    <col class="sheet__col-name" />
                    <col class="sheet__col-usage" />
                    <col class="sheet__col-pack" />
                    <col class="sheet__col-max" />
                    <col class="sheet__col-request" />
                </colgroup>
                <thead>
                    <tr>
                        <th>"ลำดับ"</th>
                        <th>"รายการยาสมุนไพร"</th>
                        <th class="sheet__th-num">"จำนวนใช้"</th>
                        <th>"หน่วยเบิก"</th>
                        <th class="sheet__th-num">"สูงสุด/รอบ"</th>
                        <th class="sheet__th-num">"จำนวนเบิก"</th>
                    </tr>
                </thead>
                <tbody>
                    {page
                        .rows
                        .into_iter()
                        .map(|placed| {
                            let row = placed.row;
                            let edit_icode = row.icode.clone();
                            let edit_name = row.name.clone();
                            let edit_strength = row.strength_text.clone();
                            let edit_usage_unit = row.usage_unit.clone();
                            let edit_pack_unit = row.pack_unit.clone();
                            let edit_usage = row.usage;
                            let edit_max = row.max_packs;
                            let edit_requested = row.requested;
                            let on_edit = {
                                let state = state.clone();
                                move |_| {
                                    if !editable {
                                        return;
                                    }
                                    state.pending_edit.set(Some(PendingEdit {
                                        icode: edit_icode.clone(),
                                        name: edit_name.clone(),
                                        strength_text: edit_strength.clone(),
                                        usage_unit: edit_usage_unit.clone(),
                                        pack_unit: edit_pack_unit.clone(),
                                        max_packs: edit_max,
                                        usage: edit_usage,
                                        old_requested: edit_requested,
                                        new_requested: edit_requested,
                                    }));
                                }
                            };
                            view! {
                                <tr class="sheet__row">
                                    <td class="sheet__no">{placed.index + 1}</td>
                                    <td class="sheet__name-cell">
                                        <span class="sheet__drug">{row.name.clone()}</span>
                                        {(!row.strength_text.trim().is_empty())
                                            .then(|| {
                                                view! {
                                                    <span class="sheet__strength">
                                                        {row.strength_text.clone()}
                                                    </span>
                                                }
                                            })}
                                    </td>
                                    <td class="sheet__num">{row.usage_label()}</td>
                                    <td class="sheet__pack-cell">{row.pack_label()}</td>
                                    <td class="sheet__num">{row.max_label()}</td>
                                    <td class="sheet__request-cell">
                                        {if editable {
                                            view! {
                                                <button
                                                    type="button"
                                                    class="sheet__request-button"
                                                    title="แก้ไขจำนวนเบิก (ต้องระบุเหตุผล)"
                                                    aria-label=format!(
                                                        "แก้ไขจำนวนเบิก {}",
                                                        row.name,
                                                    )
                                                    on:click=on_edit
                                                >
                                                    {row.requested_label()}
                                                </button>
                                            }
                                                .into_any()
                                        } else {
                                            view! {
                                                <span class="sheet__request-readonly">
                                                    {row.requested_label()}
                                                </span>
                                            }
                                                .into_any()
                                        }}
                                    </td>
                                </tr>
                            }
                        })
                        .collect_view()}
                    {(!has_rows)
                        .then(|| {
                            view! {
                                <tr class="sheet__empty-row">
                                    <td colspan="6">
                                        "ยังไม่มีรายการยาสมุนไพรในช่วงวันที่นี้ — กด \"เพิ่มรายการยา\" เพื่อขอเบิก"
                                    </td>
                                </tr>
                            }
                        })}
                </tbody>
            </table>

            {is_last
                .then(|| {
                    view! {
                        {(!adjustments.is_empty())
                            .then(|| {
                                let rows = adjustments
                                    .into_iter()
                                    .map(|(name, reason)| {
                                        view! {
                                            <tr class="sheet__row">
                                                <td class="sheet__adjustment-name">{name}</td>
                                                <td class="sheet__adjustment-reason">{reason}</td>
                                            </tr>
                                        }
                                    })
                                    .collect_view();
                                view! {
                                    <h3 class="sheet__adjustments-title">
                                        "ยาที่ขอเพิ่มหรือขอแก้ไขจำนวนเบิก"
                                    </h3>
                                    <table class="sheet__table sheet__adjustments">
                                        <colgroup>
                                            <col class="sheet__col-adjust-name" />
                                            <col class="sheet__col-adjust-reason" />
                                        </colgroup>
                                        <thead>
                                            <tr>
                                                <th>"รายการยา"</th>
                                                <th>"เหตุผล"</th>
                                            </tr>
                                        </thead>
                                        <tbody>{rows}</tbody>
                                    </table>
                                }
                            })}
                        <div class="sheet__signatures">
                            <div class="sheet__signature">
                                <div class="sheet__signature-inner">
                                    <p>"ลงชื่อ .................................................. ผู้เบิก"</p>
                                    <p class="sheet__signer-name">
                                        "( "
                                        {requested_by
                                            .clone()
                                            .map(|name| name.trim().to_string())
                                            .filter(|name| !name.is_empty())
                                            .unwrap_or_else(|| {
                                                ".................................................."
                                                    .to_string()
                                            })}
                                        " )"
                                    </p>
                                    <p>{signature_date_text.clone()}</p>
                                </div>
                            </div>
                            <div class="sheet__signature">
                                <div class="sheet__signature-inner">
                                    <p>"ลงชื่อ .................................................. หัวหน้างาน"</p>
                                    <p class="sheet__signer-name">
                                        "( "
                                        {if approvals.supervisor_name.trim().is_empty() {
                                            "..................................................".to_string()
                                        } else {
                                            approvals.supervisor_name.clone()
                                        }}
                                        " )"
                                    </p>
                                    <p>{signature_date_text.clone()}</p>
                                </div>
                            </div>
                        </div>
                    }
                })}
            <footer class="sheet__footer">
                <span>{page_label}</span>
                {is_last
                    .then(|| {
                        printed_at
                            .map(|printed_at| {
                                format!("พิมพ์เมื่อ {}", format_thai_datetime_full(printed_at))
                            })
                            .unwrap_or_default()
                    })}
            </footer>
        </article>
    }
}

#[component]
pub fn SheetBanner(state: AppState) -> impl IntoView {
    let banner_state = state.clone();
    let auto_dismiss_state = state.clone();
    Effect::new(move |_| {
        let Some(banner) = auto_dismiss_state.banner.get() else {
            return;
        };
        if banner.kind != BannerKind::Info {
            return;
        }
        let message = banner.message.clone();
        let state = auto_dismiss_state.clone();
        let _ = set_timeout_with_handle(
            move || {
                let still_same = state.banner.get_untracked().is_some_and(|current| {
                    current.kind == BannerKind::Info && current.message == message
                });
                if still_same {
                    state.banner.set(None);
                }
            },
            Duration::from_secs(5),
        );
    });

    view! {
        {move || {
            banner_state
                .banner
                .get()
                .map(|banner| {
                    let class = match banner.kind {
                        BannerKind::Error => "banner banner--error",
                        BannerKind::Warn => "banner banner--warn",
                        BannerKind::Info => "banner banner--info",
                    };
                    let close_state = banner_state.clone();
                    view! {
                        <div class=class role="status">
                            <span class="banner__text">{banner.message}</span>
                            <button
                                class="banner__close"
                                aria-label="ปิด"
                                on:click=move |_| close_state.banner.set(None)
                            >
                                <IconX class="icon" />
                            </button>
                        </div>
                    }
                })
        }}
    }
}
