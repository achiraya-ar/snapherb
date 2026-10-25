use chrono::Local;
use leptos::prelude::*;
use snap_herb_core::{PageRow, SheetPage, paginate_rows};
use snap_herb_models::{format_qty, format_thai_date};

use crate::components::document_sheet::SheetData;

const OFFICIAL_ROWS_PER_PAGE: usize = 20;

fn clean(value: &str) -> String {
    value.trim().to_string()
}

#[component]
pub fn OfficialSheet(data: SheetData) -> impl IntoView {
    let mut pages = paginate_rows(&data.rows, OFFICIAL_ROWS_PER_PAGE);
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
                    <OfficialPage data=data.clone() page=page />
                }
            })
            .collect_view()}
    }
}

#[component]
fn OfficialPage(data: SheetData, page: SheetPage) -> impl IntoView {
    let form_date = data
        .printed_at
        .map(|printed_at| printed_at.date())
        .unwrap_or_else(|| Local::now().date_naive());
    let date_text = format_thai_date(form_date);
    let work_unit = clean(&data.approvals.work_unit);
    let receiver_name = clean(&data.approvals.receiver_name);
    let requester_name = clean(data.requested_by.as_deref().unwrap_or(""));
    let requester_position = clean(&data.approvals.requester_position);
    let receiver_position = clean(&data.approvals.receiver_position);
    let orderer_name = clean(&data.approvals.orderer_name);
    let orderer_position = clean(&data.approvals.orderer_position);
    let payer_name = clean(&data.approvals.payer_name);
    let payer_position = clean(&data.approvals.payer_position);
    let document_no = String::new();
    let page_label = format!("หน้า {} / {}", page.number, page.count);
    let start = page.number.saturating_sub(1) * OFFICIAL_ROWS_PER_PAGE;
    let mut slots: Vec<Option<PageRow>> = (0..OFFICIAL_ROWS_PER_PAGE).map(|_| None).collect();
    for placed in page.rows {
        let local = placed.index.saturating_sub(start);
        if local < OFFICIAL_ROWS_PER_PAGE {
            slots[local] = Some(placed);
        }
    }
    let rows = slots
        .into_iter()
        .enumerate()
        .map(|(offset, slot)| {
            let number = start + offset + 1;
            match slot {
                Some(placed) => {
                    let row = placed.row;
                    let strength = row.strength_text.trim().to_string();
                    view! {
                        <tr class="official-table__row">
                            <td class="official-table__no">{format!("{number}.")}</td>
                            <td class="official-table__item">
                                <span>{row.name.clone()}</span>
                                {(!strength.is_empty())
                                    .then(|| {
                                        view! {
                                            <span class="official-table__strength">{strength}</span>
                                        }
                                    })}
                            </td>
                            <td class="official-table__center">{row.pack_unit.clone()}</td>
                            <td class="official-table__center"></td>
                            <td class="official-table__center">
                                {format_qty(row.requested)}
                            </td>
                            <td class="official-table__center"></td>
                            <td class="official-table__center"></td>
                        </tr>
                    }
                    .into_any()
                }
                None => view! {
                    <tr class="official-table__row">
                        <td class="official-table__no">{format!("{number}.")}</td>
                        <td></td>
                        <td></td>
                        <td></td>
                        <td></td>
                        <td></td>
                        <td></td>
                    </tr>
                }
                .into_any(),
            }
        })
        .collect_view();

    view! {
        <article class="sheet official-sheet">
            <header class="official-header">
                <h1 class="official-title">"ใบเบิกพัสดุ"</h1>
                <h2 class="official-subtitle">"(ยาและวัสดุเภสัชกรรม)"</h2>
            </header>

            <div class="official-top">
                <div class="official-top__field official-top__field--work">
                    <span class="official-top__label">"งาน"</span>
                    <span class="official-value official-value--grow">{work_unit.clone()}</span>
                </div>
                <div class="official-top__field official-top__field--doc">
                    <span class="official-top__label">"ใบเบิกเลขที่"</span>
                    <span class="official-value official-value--doc">{document_no}</span>
                </div>
            </div>

            <p class="official-to">"ถึง หัวหน้าหน่วยงานพัสดุ"</p>

            <p class="official-request">
                "ข้าพเจ้าขอเบิกพัสดุ (ยาและเวชภัณฑ์ทางการแพทย์) ตามรายการข้างล่างนี้เพื่อใช้ในงาน "
                <span class="official-value official-value--request-work">{work_unit.clone()}</span>
                " และต้องการใช้สิ่งของตามใบเบิกนี้ ภายในวันที่ "
                <span class="official-value official-value--needed-date"></span>
                " และมอบให้ "
                <span class="official-value official-value--receiver">{receiver_name.clone()}</span>
                " เป็นผู้รับสิ่งของตามใบเบิกนี้"
            </p>

            <table class="official-table">
                <colgroup>
                    <col class="official-col-no" />
                    <col class="official-col-item" />
                    <col class="official-col-unit" />
                    <col class="official-col-remain" />
                    <col class="official-col-request" />
                    <col class="official-col-pay" />
                    <col class="official-col-note" />
                </colgroup>
                <thead>
                    <tr>
                        <th rowspan="2">"ลำดับ"</th>
                        <th rowspan="2">"รายการ"</th>
                        <th rowspan="2">"หน่วยนับ"</th>
                        <th rowspan="2">"คงเหลือ"</th>
                        <th colspan="2">"จำนวน"</th>
                        <th rowspan="2">"หมายเหตุ สำหรับ จนท.ห้องยา"</th>
                    </tr>
                    <tr>
                        <th>"เบิกครั้งนี้"</th>
                        <th>"จ่าย"</th>
                    </tr>
                </thead>
                <tbody>{rows}</tbody>
            </table>

            <div class="official-signatures">
                <SignatureBlock
                    title="ผู้เบิก".to_string()
                    name=requester_name
                    position=requester_position
                    date=date_text.clone()
                />
                <SignatureBlock
                    title="ผู้รับ".to_string()
                    name=receiver_name
                    position=receiver_position
                    date=String::new()
                />
                <SignatureBlock
                    title="ผู้สั่งจ่าย".to_string()
                    name=orderer_name
                    position=orderer_position
                    date=String::new()
                />
                <SignatureBlock
                    title="ผู้จ่าย".to_string()
                    name=payer_name
                    position=payer_position
                    date=String::new()
                />
            </div>

            <footer class="official-footer">{page_label}</footer>
        </article>
    }
}

#[component]
fn SignatureBlock(title: String, name: String, position: String, date: String) -> impl IntoView {
    view! {
        <div class="official-sign">
            <div class="official-sign__line">
                <span class="official-sign__label">"ลงชื่อ"</span>
                <span class="official-sign__leader"></span>
                <span class="official-sign__title">{title}</span>
            </div>
            <p class="official-sign__name">
                "( "
                <span class="official-value official-value--name">{name}</span>
                " )"
            </p>
            <p class="official-sign__detail">
                "ตำแหน่ง "
                <span class="official-value official-value--detail">{position}</span>
            </p>
            <p class="official-sign__detail">
                "วันที่ "
                <span class="official-value official-value--detail">{date}</span>
            </p>
        </div>
    }
}
