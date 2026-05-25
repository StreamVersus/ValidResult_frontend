use leptos::prelude::*;

#[derive(Debug, Copy, Clone)]
#[derive(PartialEq)]
pub enum Sort {
    DESC, // from highest to lowest
    ASC, // from lowest to highest
}

impl Sort {
    pub fn cycle(self) -> (Sort, bool) {
        match self {
            Sort::DESC => (Sort::ASC, false),
            Sort::ASC  => (Sort::DESC, true),
        }
    }
}

impl Default for Sort {
    fn default() -> Self {
        Sort::DESC
    }
}

#[component]
pub fn SortButton(col_idx: usize, set_sort_col: WriteSignal<Option<(usize, Sort)>>, sort_col: ReadSignal<Option<(usize, Sort)>>) -> impl IntoView {
    view! {
        <button
            type="button"
            style="background:#0f1219; border:1px solid #4a5568; border-radius:4px; color:#c8d8f0; width:22px; height:22px; display:flex; align-items:center; justify-content:center; cursor:pointer;"
            class="hover:border-[#4a9eff] focus:outline-none transition-colors text-[11px]"
            on:click=move |_| {
                set_sort_col
                    .update(|current| {
                        let ret_val;
                        if let Some((idx, sort)) = current {
                            if *idx != col_idx {
                                return;
                            }
                            let (new_sort, should_end) = sort.cycle();
                            if should_end {
                                ret_val = None;
                            } else {
                                ret_val = Some((*idx, new_sort));
                            }
                        } else {
                            ret_val = Some((col_idx, Sort::default()));
                        }
                        *current = ret_val;
                    });
            }
        >
            {move || match sort_col.get() {
                Some((idx, Sort::ASC)) if idx == col_idx => "▲",
                Some((idx, Sort::DESC)) if idx == col_idx => "▼",
                _ => "↕",
            }}
        </button>
    }
}