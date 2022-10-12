use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    html! {
        <>
        <div>
            <h1>{"Yew image app"}</h1>
        </div>
        </>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<App>();
}
