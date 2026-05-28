use crate::FromToStringCodec;
use leptos::prelude::*;
use leptos_use::use_cookie;
use web_sys;
use crate::backend::MetricType;

#[component]
pub fn AddMetrics(set_trigger_refresh: WriteSignal<usize>) -> impl IntoView {
    let (is_open, set_open) = signal(false);

    let (selected_model, set_selected_model) = signal(MetricType::LLM);
    let (model_name, set_model_name) = signal(String::new());
    let (metric_values, set_metric_values) = signal::<Vec<String>>(vec![]);

    let cookie_name = Signal::derive(move || format!("user_models_{:?}", selected_model.get()));
    Effect::new(move |_| {
        if !is_open.get() {
            set_model_name.set(String::new());
            set_metric_values.set(vec![]);
        }
    });

    Effect::new(move |_| {
        let count = selected_model.get().metric_labels().len();
        set_metric_values.set(vec!["".to_string(); count]);
    });

    let on_metric_change = move |idx: usize, value: String| {
        set_metric_values.update(|vals| {
            if idx < vals.len() {
                vals[idx] = value;
            }
        });
    };

    let on_submit = move |_| {
        let name = model_name.get().trim().to_string();
        if name.is_empty() {
            _ = web_sys::window()
                .and_then(|w| w.alert_with_message("Model name is required").ok());
            return;
        }

        let vals = metric_values.get();

        let new_model_csv = std::iter::once(name)
            .chain(vals.into_iter())
            .map(|str| {
                if str.is_empty() {
                    return "0.0".to_string();
                } else {
                    str
                }
            })
            .collect::<Vec<_>>()
            .join(",");

        let (cookie_val, set_cookie) = use_cookie::<String, FromToStringCodec>(&cookie_name.get());
        let existing = cookie_val.get();

        let updated_csv = if let Some(existing) = existing {
            format!("{}|{}", existing, new_model_csv)
        } else {
            new_model_csv
        };

        set_cookie.set(Some(updated_csv));

        set_open.set(false);
        set_trigger_refresh.set(0);
    };

    let on_backdrop_click = move |ev: web_sys::MouseEvent| {
        if ev.target() == ev.current_target() {
            set_open.set(false);
        }
    };

    let stop_prop = |ev: web_sys::MouseEvent| ev.stop_propagation();

    view! {
        <div class="curvy-newt-container" data-open=move || is_open.get().to_string()>
            <button
                type="button"
                class="curvy-newt-btn px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-500 transition-colors"
                on:click=move |_| set_open.update(|v| *v = !*v)
            >
                <span class="truncate">Добавить модель</span>
            </button>
        </div>

        <Show when=move || is_open.get() fallback=|| ()>
            <div
                style="position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; z-index: 2147483647; background: rgba(0,0,0,0.6); backdrop-filter: blur(4px); display: flex; align-items: center; justify-content: center; padding: 1rem;"
                on:click=on_backdrop_click
            >
                <div
                    style="position: relative; z-index: 2147483647; background: #1f2937; border-radius: 12px; width: 100%; max-width: 48rem; max-height: 90vh; display: flex; flex-direction: column; overflow: hidden; border: 1px solid #374151; box-shadow: 0 25px 50px -12px rgba(0,0,0,0.5); color: white;"
                    on:click=stop_prop
                >
                    {}
                    <div style="padding: 1rem; border-bottom: 1px solid #374151; display: flex; justify-content: space-between; align-items: center;">
                        <h2 style="font-size: 1.125rem; font-weight: 600;">
                            Настройка модели
                        </h2>
                        <button
                            on:click=move |_| set_open.set(false)
                            style="color: #9ca3af; font-size: 1.25rem; background: none; border: none; cursor: pointer; line-height: 1;"
                        >
                            x
                        </button>
                    </div>

                    {}
                    <div style="flex: 1; overflow-y: auto; padding: 1rem; display: flex; flex-direction: column; gap: 1rem;">
                        {} <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                            <div>
                                <label class="block text-sm text-gray-400 mb-1">
                                    Тип модели
                                </label>
                                <select
                                    class="w-full p-2 bg-gray-700 border border-gray-600 rounded text-white"
                                    prop:value=move || selected_model.get() as usize
                                    on:change=move |ev: web_sys::Event| {
                                        if let Ok(t) = event_target_value(&ev).parse::<usize>() {
                                            if let Some(model) = MetricType::ALL.iter().nth(t) {
                                                set_selected_model.set(*model);
                                            }
                                        }
                                    }
                                >
                                    {MetricType::ALL
                                        .iter()
                                        .enumerate()
                                        .map(|(i, m)| {
                                            view! { <option value=i>{format!("{:?}", m)}</option> }
                                        })
                                        .collect::<Vec<_>>()}
                                </select>
                            </div>

                            {}
                            <div>
                                <label class="block text-sm text-gray-400 mb-1">
                                    Название
                                </label>
                                <input
                                    type="text"
                                    class="w-full p-2 bg-gray-700 border border-gray-600 rounded text-white"
                                    placeholder="my_model_v1"
                                    prop:value=model_name
                                    on:input=move |ev: web_sys::Event| {
                                        set_model_name.set(event_target_value(&ev))
                                    }
                                />
                            </div>
                        </div> {} <div>
                            <label class="block text-sm text-gray-400 mb-2">Метрики</label>
                            <div class="grid grid-cols-2 md:grid-cols-3 gap-3 max-h-64 overflow-y-auto pr-1">
                                {move || {
                                    let model = selected_model.get();
                                    let labels = model.metric_labels();
                                    labels
                                        .iter()
                                        .enumerate()
                                        .map(|(idx, &label)| {

                                            view! {
                                                <div class="flex flex-col gap-1">
                                                    <span class="text-xs text-gray-300 select-none">
                                                        {label.to_string()}
                                                    </span>
                                                    <input
                                                        type="text"
                                                        inputmode="decimal"
                                                        class="w-full p-2 bg-gray-700 border border-gray-600 rounded text-white text-sm focus:border-blue-500 focus:outline-none"
                                                        prop:value=move || {
                                                            metric_values
                                                                .get()
                                                                .get(idx)
                                                                .cloned()
                                                                .unwrap_or("".to_string())
                                                        }

                                                        on:input=move |ev: web_sys::Event| {
                                                            let raw = event_target_value(&ev);
                                                            let mut has_dot = false;
                                                            let filtered: String = raw
                                                                .chars()
                                                                .filter(|c| {
                                                                    if c.is_ascii_digit() {
                                                                        true
                                                                    } else if *c == '.' && !has_dot {
                                                                        has_dot = true;
                                                                        true
                                                                    } else {
                                                                        false
                                                                    }
                                                                })
                                                                .collect();
                                                            on_metric_change(idx, filtered);
                                                        }

                                                        on:change=move |ev: web_sys::Event| {
                                                            let raw = event_target_value(&ev);
                                                            if let Ok(val) = raw.parse::<f64>() {
                                                                let clamped = val.max(0.0).min(5.0);
                                                                on_metric_change(idx, format!("{:.2}", clamped));
                                                            } else {
                                                                on_metric_change(idx, "0.00".to_string());
                                                            }
                                                        }
                                                    />
                                                </div>
                                            }
                                        })
                                        .collect::<Vec<_>>()
                                }}
                            </div>
                        </div>
                    </div>

                    {}
                    <div style="padding: 1rem; border-top: 1px solid #374151; display: flex; justify-content: flex-end; gap: 0.5rem;">
                        <button
                            class="px-4 py-2 rounded bg-gray-600 hover:bg-gray-500 text-white transition-colors"
                            on:click=move |_| set_open.set(false)
                        >
                            Отмена
                        </button>
                        <button
                            class="px-4 py-2 rounded bg-blue-600 hover:bg-blue-500 text-white font-medium transition-colors"
                            on:click=on_submit
                        >
                            Сохранить
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}