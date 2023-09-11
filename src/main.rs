/*use axum::{
    extract::Form,
    response::Html,
    routing::{get, post},
    Router,
}; */

use leptos::*;
use serde::Deserialize;
//use sqlx::sqlite::SqlitePoolOptions;
//use sqlx::{Pool, Row, Sqlite};

//#[tokio::main]
fn main() {
    mount_to_body(|cx| view! {cx, <App/>});
}
#[component]
fn App(cx: Scope) -> impl IntoView {
    // (getter: ReadSignal<i32>, setter: WriteSignal<i32>)
    let (count, set_count) = create_signal(cx, 0);

    // could also do {count} instead of {move || count.get()} as count is already a function
    view! { cx,
        <button
            on:click=move |_| {
            set_count.update(|x| *x += 1);
        }>
        "Click me: "{move || count.get()}
        <Colors/>
        </button>
    }
}

//shows dynamic css classes, note the style:left, etc
#[component]
fn Colors(cx: Scope) -> impl IntoView {
    // can add raw html like this
    // let html = "<p>Injected html</p>";
    let html_str = "<p>Injected html</p>";
    let (x, set_x) = create_signal(cx, 0);
    let (y, set_y) = create_signal(cx, 0);
    view! { cx,
        <div
                style="position: absolute"
                style:left=move || format!("{}px", x() + 100)
                style:top=move || format!("{}px", y() + 100)
                style:background-color=move || format!("rgb({}, {}, 100)", x(), y())
                style=("--columns", x)
            >
        "Moves when coords change"
        </div>
        <button
            on:click=move |_| {set_x.update(|n| *n += 1);}
        >"Update me"
        </button>
        <button
            on:click=move |_| {set_y.update(|n| *n += 1);}
        >"Update y"</button>
        <div inner_html=html_str/>
        <ProgressBar progress=y/>
    }
}

#[component]
fn ProgressBar(cx: Scope, progress: ReadSignal<i32>) -> impl IntoView {
    view! { cx,
        <progress
            max="50"
            // now this works
            value=progress
        />
    }
}
