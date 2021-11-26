use sycamore::prelude::*;

#[component(App<G>)]
fn app() -> View<G> {
    let name = Signal::new(String::new());

    view! {
        div {
            h1 {
                "Hello "
                (if *create_selector(move || !name.get().is_empty()).get() {
                    view! {
                        span { (name.get()) }
                    }
                } else {
                    view! { span { "World" } }
                })
                "!"
            }

            input(bind:value=name)
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    sycamore::render(|| view! { App() });
}
