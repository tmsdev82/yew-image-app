use gloo_file::{callbacks::FileReader, File};
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};
use web_sys::{Event, HtmlInputElement};
use yew::prelude::*;

pub enum Msg {
    LoadedBytes(String, Vec<u8>),
    Files(Vec<File>),
}

pub struct FileDataComponent {
    files: Vec<String>,
    readers: HashMap<String, FileReader>,
}

impl Component for FileDataComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            files: Vec::new(),
            readers: HashMap::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Files(files) => {
                log::info!("Files selected: {}", files.len());
                for file in files.into_iter() {
                    let file_name = file.name();
                    let task = {
                        let file_name = file_name.clone();
                        let link = ctx.link().clone();

                        gloo_file::callbacks::read_as_bytes(&file, move |res| {
                            link.send_message(Msg::LoadedBytes(
                                file_name,
                                res.expect("failed to read file"),
                            ))
                        })
                    };
                    self.readers.insert(file_name, task);
                }
                true
            }
            Msg::LoadedBytes(file_name, data) => {
                log::info!("Processing: {}", file_name);

                let image_data = base64::encode(data);
                self.files.push(image_data);
                self.readers.remove(&file_name);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_change = ctx.link().callback(move |e: Event| {
            let mut selected_files = Vec::new();
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Some(files) = input.files() {
                let files = js_sys::try_iter(&files)
                    .unwrap()
                    .unwrap()
                    .map(|v| web_sys::File::from(v.unwrap()))
                    .map(File::from);
                selected_files.extend(files);
            }
            Msg::Files(selected_files)
        });
        html! {
            <div>
                <div>
                    {"Choose a image file:"}
                </div>
                <div>
                    <input type="file" accept="image/png" onchange={on_change} multiple=false/>
                </div>
                <div>
                { for self.files.iter().map(|f| Self::view_file(f))}
                </div>
            </div>
        }
    }
}

impl FileDataComponent {
    fn view_file(data: &str) -> Html {
        let img = format!("data:image/png;base64,{}", data.to_string());
        html! {
            <div>
                <img src={img}/>
            </div>
        }
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <>
        <div>
            <h1>{"Yew image app"}</h1>
            <FileDataComponent/>
        </div>
        </>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<App>();
}
