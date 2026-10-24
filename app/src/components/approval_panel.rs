use leptos::prelude::*;
use snap_herb_models::ApprovalSettings;

use crate::api;
use crate::components::settings_modal::{StatusLine, StatusMessage};
use crate::state::AppState;

#[component]
pub fn ApprovalPanel(state: AppState) -> impl IntoView {
    let initial = state
        .settings
        .get_untracked()
        .map(|settings| settings.approvals)
        .unwrap_or_default();
    let work_unit = RwSignal::new(initial.work_unit);
    let requester_position = RwSignal::new(initial.requester_position);
    let receiver_name = RwSignal::new(initial.receiver_name);
    let receiver_position = RwSignal::new(initial.receiver_position);
    let orderer_name = RwSignal::new(initial.orderer_name);
    let orderer_position = RwSignal::new(initial.orderer_position);
    let payer_name = RwSignal::new(initial.payer_name);
    let payer_position = RwSignal::new(initial.payer_position);
    let supervisor_name = RwSignal::new(initial.supervisor_name);
    let status = RwSignal::new(None::<StatusMessage>);
    let busy = RwSignal::new(false);

    let on_save = {
        let state = state.clone();
        move |_| {
            let approvals = ApprovalSettings {
                work_unit: work_unit.get(),
                requester_position: requester_position.get(),
                receiver_name: receiver_name.get(),
                receiver_position: receiver_position.get(),
                orderer_name: orderer_name.get(),
                orderer_position: orderer_position.get(),
                payer_name: payer_name.get(),
                payer_position: payer_position.get(),
                supervisor_name: supervisor_name.get(),
            };
            busy.set(true);
            status.set(None);
            let state = state.clone();
            leptos::task::spawn_local(async move {
                match api::save_approvals(&approvals).await {
                    Ok(()) => {
                        state.settings.update(|settings| {
                            if let Some(settings) = settings {
                                settings.approvals = approvals;
                            }
                        });
                        state.invalidate_confirmation();
                        status.set(Some(StatusMessage::ok("บันทึกผู้ลงนามแล้ว")));
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
            let approvals = state
                .settings
                .get_untracked()
                .map(|settings| settings.approvals)
                .unwrap_or_default();
            work_unit.set(approvals.work_unit);
            requester_position.set(approvals.requester_position);
            receiver_name.set(approvals.receiver_name);
            receiver_position.set(approvals.receiver_position);
            orderer_name.set(approvals.orderer_name);
            orderer_position.set(approvals.orderer_position);
            payer_name.set(approvals.payer_name);
            payer_position.set(approvals.payer_position);
            supervisor_name.set(approvals.supervisor_name);
            status.set(None);
        }
    };

    view! {
        <div class="panel">
            <h3 class="panel__subtitle">"ผู้ลงนามในใบเบิก"</h3>
            <p class="panel__note">
                "ชื่อผู้เบิกใช้จากบัญชีที่เข้าสู่ระบบ ส่วนชื่อและตำแหน่งด้านล่างใช้เติมในใบเบิกพัสดุ (ทางการ) และแบบฟอร์มระบบ"
            </p>

            <label class="field">
                <span class="field__label">"งาน / หน่วยงานที่เบิก"</span>
                <input
                    class="text-input"
                    type="text"
                    placeholder="เช่น กลุ่มงานเภสัชกรรม"
                    prop:value=move || work_unit.get()
                    on:input=move |ev| work_unit.set(event_target_value(&ev))
                />
            </label>

            <div class="form-grid form-grid--even">
                <label class="field">
                    <span class="field__label">"ตำแหน่งผู้เบิก"</span>
                    <input
                        class="text-input"
                        type="text"
                        placeholder="เช่น เจ้าหน้าที่"
                        prop:value=move || requester_position.get()
                        on:input=move |ev| requester_position.set(event_target_value(&ev))
                    />
                </label>
                <label class="field">
                    <span class="field__label">"ชื่อหัวหน้างาน (แบบฟอร์มระบบ)"</span>
                    <input
                        class="text-input"
                        type="text"
                        placeholder="ชื่อผู้ลงนามหัวหน้างาน"
                        prop:value=move || supervisor_name.get()
                        on:input=move |ev| supervisor_name.set(event_target_value(&ev))
                    />
                </label>
            </div>

            <div class="form-card">
                <span class="form-card__title">"ผู้รับ"</span>
                <div class="form-grid form-grid--even">
                    <label class="field">
                        <span class="field__label">"ชื่อผู้รับ"</span>
                        <input
                            class="text-input"
                            type="text"
                            prop:value=move || receiver_name.get()
                            on:input=move |ev| receiver_name.set(event_target_value(&ev))
                        />
                    </label>
                    <label class="field">
                        <span class="field__label">"ตำแหน่งผู้รับ"</span>
                        <input
                            class="text-input"
                            type="text"
                            prop:value=move || receiver_position.get()
                            on:input=move |ev| receiver_position.set(event_target_value(&ev))
                        />
                    </label>
                </div>
            </div>

            <div class="form-card">
                <span class="form-card__title">"ผู้สั่งจ่าย"</span>
                <div class="form-grid form-grid--even">
                    <label class="field">
                        <span class="field__label">"ชื่อผู้สั่งจ่าย"</span>
                        <input
                            class="text-input"
                            type="text"
                            prop:value=move || orderer_name.get()
                            on:input=move |ev| orderer_name.set(event_target_value(&ev))
                        />
                    </label>
                    <label class="field">
                        <span class="field__label">"ตำแหน่งผู้สั่งจ่าย"</span>
                        <input
                            class="text-input"
                            type="text"
                            prop:value=move || orderer_position.get()
                            on:input=move |ev| orderer_position.set(event_target_value(&ev))
                        />
                    </label>
                </div>
            </div>

            <div class="form-card">
                <span class="form-card__title">"ผู้จ่าย"</span>
                <div class="form-grid form-grid--even">
                    <label class="field">
                        <span class="field__label">"ชื่อผู้จ่าย"</span>
                        <input
                            class="text-input"
                            type="text"
                            prop:value=move || payer_name.get()
                            on:input=move |ev| payer_name.set(event_target_value(&ev))
                        />
                    </label>
                    <label class="field">
                        <span class="field__label">"ตำแหน่งผู้จ่าย"</span>
                        <input
                            class="text-input"
                            type="text"
                            prop:value=move || payer_position.get()
                            on:input=move |ev| payer_position.set(event_target_value(&ev))
                        />
                    </label>
                </div>
            </div>

            <StatusLine message=status />
            <div class="panel__actions">
                <button class="button-secondary" disabled=move || busy.get() on:click=on_reset>
                    "คืนค่าเดิม"
                </button>
                <button class="button-primary" disabled=move || busy.get() on:click=on_save>
                    "บันทึกผู้ลงนาม"
                </button>
            </div>
        </div>
    }
}
