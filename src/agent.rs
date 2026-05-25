use std::future::Future;
use std::pin::Pin;
use leptos::ev::KeyboardEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn AiChat(
    on_send: Callback<(String, bool), Pin<Box<dyn Future<Output = String>>>>,
) -> impl IntoView {
    let (input, set_input) = signal(String::new());
    let (messages, set_messages) = signal::<Vec<(bool, String)>>(vec![]);
    let (is_loading, set_is_loading) = signal(false);
    let (table_access, set_table_access) = signal(false);

    let handle_send = move || {
        let text = input.get_untracked();
        if text.trim().is_empty() || is_loading.get_untracked() {
            return;
        }
        let access = table_access.get_untracked();
        set_input.set(String::new());
        set_messages.update(|m| m.push((true, text.clone())));
        set_is_loading.set(true);
        spawn_local(async move {
            let response = on_send.run((text, access)).await;
            set_messages.update(|m| m.push((false, response)));
            set_is_loading.set(false);
        });
    };

    let handle_keydown = move |ev: KeyboardEvent| {
        if ev.key() == "Enter" && !ev.shift_key() {
            ev.prevent_default();
            handle_send();
        }
    };

    let border_col = "#1e3a6e";
    let bg_dark    = "#0d1226";
    let bg_cell    = "#0f1a3a";
    let fg_main    = "#c8d8f0";
    let fg_dim     = "#4a6080";

    view! {
        <div style=format!(
            "display:flex; flex-direction:column; height:100%; \
             background:{bg_dark}; border:3px solid {border_col}; border-radius:6px; \
             font-family:'JetBrains Mono',monospace; overflow:hidden;",
        )>
            <div style=format!(
                "padding:14px 20px; border-bottom:3px solid {border_col}; \
                 background:{bg_cell}; display:flex; align-items:center; \
                 justify-content:space-between; flex-shrink:0;",
            )>
                <span style="color:#e2e8f0; font-weight:700; font-size:14px;">
                    "✨ ИИ Ассистент"
                </span>

                <label style="display:flex; align-items:center; gap:8px; cursor:pointer; user-select:none;">
                    <span style=format!(
                        "color:{fg_dim}; font-size:11px;",
                    )>"Доступ к таблице"</span>
                    <div
                        style=move || {
                            format!(
                                "width:36px; height:20px; border-radius:10px; position:relative; \
                             transition:background 0.2s; cursor:pointer; background:{};",
                                if table_access.get() { "#2563eb" } else { "#1e3a6e" },
                            )
                        }
                        on:click=move |_| set_table_access.update(|v| *v = !*v)
                    >
                        <div style=move || {
                            format!(
                                "width:14px; height:14px; border-radius:50%; background:white; \
                             position:absolute; top:3px; transition:left 0.2s; left:{};",
                                if table_access.get() { "19px" } else { "3px" },
                            )
                        }></div>
                    </div>
                </label>
            </div>

            <div
                class="overflow-y-auto"
                style="flex:1; padding:16px; display:flex; flex-direction:column; \
                     gap:10px; min-height:0;"
                    .to_string()
            >
                {move || {
                    if messages.get().is_empty() {
                        view! {
                            <div style=format!(
                                "display:flex; flex-direction:column; align-items:center; \
                                 justify-content:center; height:100%; gap:8px; color:{fg_dim};",
                            )>
                                <span style="font-size:32px;">"🤖"</span>
                                <span style="font-size:13px;">
                                    "Задайте вопрос про модели"
                                </span>
                            </div>
                        }
                            .into_any()
                    } else {
                        messages
                            .get()
                            .into_iter()
                            .map(|(is_user, text)| {
                                let (bg, fg, align, radius) = if is_user {
                                    ("#1a3a7a", "#e2e8f0", "flex-end", "12px 12px 2px 12px")
                                } else {
                                    (bg_cell, fg_main, "flex-start", "12px 12px 12px 2px")
                                };
                                view! {
                                    <div style=format!("display:flex; justify-content:{align};")>
                                        <div
                                            class=if !is_user { "ai-message" } else { "" }
                                            style=format!(
                                                "background:{bg}; color:{fg}; \
                                             padding:10px 14px; border-radius:{radius}; \
                                             max-width:85%; font-size:13px; line-height:1.6; \
                                             word-break:break-word; \
                                             border:1px solid {border_col};",
                                            )
                                            prop:innerHTML=text.clone()
                                        ></div>
                                    </div>
                                }
                            })
                            .collect_view()
                            .into_any()
                    }
                }}

                {move || {
                    is_loading
                        .get()
                        .then(|| {
                            view! {
                                <div style="display:flex; justify-content:flex-start;">
                                    <div style=format!(
                                        "background:{bg_cell}; color:{fg_dim}; padding:10px 14px; \
                             border-radius:12px 12px 12px 2px; font-size:13px; \
                             border:1px solid {border_col};",
                                    )>"· · ·"</div>
                                </div>
                            }
                        })
                }}
            </div>

            <div style=format!(
                "padding:12px 16px; border-top:3px solid {border_col}; \
                 background:{bg_cell}; display:flex; \
                 position:relative; \
                 flex-shrink:0;",
            )>
                <textarea
                    prop:value=move || input.get()
                    on:input=move |ev| set_input.set(event_target_value(&ev))
                    on:keydown=handle_keydown
                    placeholder="Введите сообщение... (Enter — отправить, Shift+Enter — перенос)"
                    rows="2"
                    style=format!(
                        "flex:1; resize:none; border-radius:6px; font-family:'JetBrains Mono',monospace; \
                         font-size:13px; background:#080d18; color:{fg_main}; \
                         padding:16px 46px 16px 12px; border:1.5px solid {border_col}; \
                         max-height:80px; overflow-y:auto; line-height:1.5; outline:none; \
                         transition:border-color 0.2s;",
                    )
                />
                <button
                    on:click=move |_| handle_send()
                    disabled=move || is_loading.get()
                    style="background:#2563eb; color:white; border:none; border-radius:6px; \
                         position:absolute; right:24px; top:50%; transform:translateY(-50%); \
                         width:30px; height:30px; display:flex; align-items:center; justify-content:center; \
                         font-size:15px; font-family:'JetBrains Mono',monospace; \
                         cursor:pointer; transition:all 0.2s;"
                        .to_string()
                    style:opacity=move || if is_loading.get() { "0.4" } else { "1" }
                    style:cursor=move || if is_loading.get() { "not-allowed" } else { "pointer" }
                >
                    "→"
                </button>
            </div>
        </div>
    }
}