use gloo_file::{callbacks::FileReader, File};
use image::{imageops, DynamicImage, GenericImageView};
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
    asciis: Vec<String>,
    readers: HashMap<String, FileReader>,
}

impl Component for FileDataComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            files: Vec::new(),
            asciis: Vec::new(),
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
                let org_image = image::load_from_memory(&data).unwrap();
                let ascii_art = image_to_ascii(org_image.clone(), 4);
                let mut inverted_image = org_image.clone();
                inverted_image.invert();

                let mut out = img_to_bytes(inverted_image);

                let image_data = base64::encode(data);
                let inverted_data = base64::encode(&mut out);
                self.files.push(image_data);
                self.files.push(inverted_data);
                self.asciis.push(ascii_art);
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
                <div >
                    {for self.asciis.iter().map(|f| {
                        html!{
                            <div class="ascii-art">
                                {f}
                            </div>
                        }
                    })}
                </div>
            </div>
        }
    }
}

fn img_to_bytes(img: DynamicImage) -> Vec<u8> {
    let mut cursor = std::io::Cursor::new(Vec::new());
    match img.write_to(&mut cursor, image::ImageFormat::Png) {
        Ok(_c) => {
            log::debug!("write to cursor success!");
        }
        Err(error) => {
            panic!("There was an problem: {:?}", error)
        }
    };

    cursor.seek(SeekFrom::Start(0)).unwrap();
    let mut out = Vec::new();
    cursor.read_to_end(&mut out).unwrap();

    out
}

fn image_to_ascii(image: DynamicImage, resolution: u32) -> String {
    let pallete: [char; 12] = [' ', '.', ',', ':', ';', 'o', 'x', '9', '$', '%', '#', '@'];
    let mut y = 0;
    let mut ascii_art = String::new();
    let small_img = image.resize(
        image.width() / resolution,
        image.height() / resolution,
        imageops::FilterType::Nearest,
    );
    println!("Transforming image");
    for p in small_img.pixels() {
        if y != p.1 {
            ascii_art.push_str("\n");
            y = p.1;
        }

        let r = p.2 .0[0] as f32;
        let g = p.2 .0[1] as f32;
        let b = p.2 .0[2] as f32;
        let k = r * 0.2126 + g * 0.7152 + b * 0.0722;
        let character = ((k / 255.0) * (pallete.len() - 1) as f32).round() as usize;

        ascii_art.push(pallete[character]);
    }

    ascii_art.push_str("\n");
    ascii_art
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
