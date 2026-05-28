use std::future::Future;
use std::pin::Pin;
use leptos::{component, view, IntoView};
use leptos::prelude::*;
use leptos::prelude::codee::string::FromToStringCodec;
use leptos_use::use_cookie;
use crate::agent::AiChat;
use crate::backend::{ai_agent_turn, MetricType};
use crate::TableLoader;

#[component]
pub fn MetricRoute(metric_type: ReadSignal<MetricType>, trigger_refresh: ReadSignal<usize>) -> impl IntoView {
    let (session_id, set_session_id) = signal::<Option<String>>(None);
    let cookie_name = Signal::derive(move || format!("user_models_{:?}", metric_type.get()));

    view! {
        <div style="background:#0f1219; border:3px solid #1e3a6e; border-radius:6px">
            <TableLoader
                metric_type=metric_type
                trigger_refresh=trigger_refresh
                cookie_name=cookie_name
            />
        </div>

        <div style="background:#0f1219; border:3px solid #1e3a6e; border-radius:6px">
            <AiChat on_send=Callback::new(move |(user_message, embed)| {
                let (cookie, _) = use_cookie::<
                    String,
                    FromToStringCodec,
                >(&cookie_name.get_untracked());
                let mt = metric_type.get_untracked();
                let session_id = session_id.get_untracked();
                let user_metrics = cookie.get_untracked().unwrap_or_default();
                Box::pin(async move {
                    let (response, id) = ai_agent_turn(
                            user_message,
                            embed,
                            mt,
                            session_id,
                            user_metrics,
                        )
                        .await;
                    set_session_id.set(Some(id));
                    response
                }) as Pin<Box<dyn Future<Output = String>>>
            }) />
        </div>
    }
}