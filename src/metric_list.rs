use leptos::prelude::*;
use crate::backend::MetricType;

#[component]
pub fn MetricList(
    metric_type: ReadSignal<MetricType>,
    set_metric_type: WriteSignal<MetricType>,
) -> impl IntoView {
    let (is_open, set_open) = signal(false);
    let mock_metrics: Vec<String> = MetricType::ALL.iter().map(|t| t.to_string()).collect();

    view! {
        <div class="curvy-newt-container" data-open=move || is_open.get().to_string()>

            <button
                type="button"
                class="curvy-newt-btn"
                on:click=move |_| set_open.update(|prev| *prev = !*prev)
            >
                <span class="truncate">{move || metric_type.get().to_string()}</span>

                <svg
                    class="curvy-newt-arrow"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                    xmlns="http://www.w3.org/2000/svg"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2.5"
                        d="M19 9l-7 7-7-7"
                    ></path>
                </svg>
            </button>

            <Show when=move || is_open.get()>
                <div class="curvy-newt-menu">
                    <div class="curvy-newt-menu-inner">
                        {mock_metrics
                            .iter()
                            .cloned()
                            .map(|typename| {
                                let current_type = typename.clone();
                                view! {
                                    <button
                                        type="button"
                                        class="curvy-newt-item"
                                        data-selected=move || {
                                            (metric_type.get().to_string() == current_type).to_string()
                                        }
                                        on:click=move |_| {
                                            set_metric_type.set(MetricType::from_str(&typename));
                                            set_open.set(false);
                                        }
                                    >
                                        {typename.clone()}
                                    </button>
                                }
                            })
                            .collect_view()}
                    </div>
                </div>
            </Show>
        </div>
    }
}