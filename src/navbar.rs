use leptos::prelude::*;
use leptos::server_fn::redirect::call_redirect_hook;

#[component]
pub fn Navbar() -> impl IntoView {
    view! {
        <nav style="padding:16px; display:flex; flex-direction:column; gap:12px;">
            <a
                on:click=move |ev| {
                    ev.prevent_default();
                    call_redirect_hook("")
                }
                href="#"
                style="display:flex; align-items:center; padding:8px 12px; font-size:13px; font-weight:600; background:#1e2433; color:#4a9eff; border-radius:6px; border:1px solid #1e3a6e; text-decoration:none;"
            >
                <svg class="w-5 h-5 mr-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"
                    ></path>
                </svg>
                Рейтинг моделей
            </a>

            <a
                on:click=move |ev| {
                    ev.prevent_default();
                    call_redirect_hook("/ml")
                }
                href="#"
                style="display:flex; align-items:center; padding:8px 12px; font-size:13px; font-weight:600; background:#1e2433; color:#4a9eff; border-radius:6px; border:1px solid #1e3a6e; text-decoration:none;"
            >
                <svg class="w-5 h-5 mr-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M12 6v6m0 0v6m0-6h6m-6 0H6"
                    ></path>
                </svg>
                ML метрики
            </a>
        </nav>
    }
}