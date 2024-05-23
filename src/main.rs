use slint::SharedPixelBuffer;
use std::path::PathBuf;
use std::rc::Rc;
use std::{cell::RefCell, fs};

slint::slint! {
    import { Button } from "std-widgets.slint";

    export component MainWindow inherits Window {
        min-width: 200px;
        min-height: 200px;
        max-width: 10000px;
        max-height: 10000px;

        in-out property <image> image;
        in-out property <float> zoom: 1.0;

        callback load_image(string);

        VerticalLayout {
            Image {
                source: image;
                width: parent.width * zoom;
                height: (parent.height - 50px) * zoom;
                image-fit: contain;
            }

            HorizontalLayout {
                width: parent.width;
                height: 50px;
                alignment: center;

                Button {
                    text: "Previous";
                    clicked => { root.load_image("previous"); }
                }
                Button {
                    text: "Zoom In";
                    clicked => {
                        zoom *= 1.2;
                    }
                }
                Button {
                    text: "Zoom Out";
                    clicked => {
                        zoom /= 1.2;
                    }
                }
                Button {
                    text: "Next";
                    clicked => { root.load_image("next"); }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
struct ImageViewer {
    images: Vec<PathBuf>,
    current_index: usize,
}

impl ImageViewer {
    fn new(images: Vec<PathBuf>) -> Self {
        Self {
            images,
            current_index: 0,
        }
    }

    fn load_image(&mut self, direction: &str) -> Option<slint::Image> {
        if self.images.is_empty() {
            return None;
        }

        match direction {
            "previous" => {
                if self.current_index == 0 {
                    self.current_index = self.images.len() - 1;
                } else {
                    self.current_index -= 1;
                }
            }
            "next" => {
                self.current_index = (self.current_index + 1) % self.images.len();
            }
            _ => {}
        }

        let image_path = &self.images[self.current_index];
        let img = image::open(image_path).ok()?;
        let img = img.to_rgba8();
        let (width, height) = img.dimensions();
        let buffer = SharedPixelBuffer::clone_from_slice(&img, width, height);
        let img = slint::Image::from_rgba8(buffer);

        Some(img)
    }
}

fn main() {
    let path: PathBuf = std::env::args()
        .skip(1)
        .next()
        .unwrap_or(".".to_string())
        .into();

    let images: Vec<PathBuf> = if path.is_dir() {
        fs::read_dir(path)
            .unwrap()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension()?.to_str()?.eq_ignore_ascii_case("png")
                    || path.extension()?.to_str()?.eq_ignore_ascii_case("jpg")
                    || path.extension()?.to_str()?.eq_ignore_ascii_case("jpeg")
                {
                    Some(path)
                } else {
                    None
                }
            })
            .collect()
    } else {
        vec![path]
    };

    let viewer = Rc::new(RefCell::new(ImageViewer::new(images)));
    let main_window = MainWindow::new().unwrap();

    main_window.on_load_image({
        let main_window = main_window.as_weak();
        let viewer = Rc::clone(&viewer);
        move |direction| {
            if let Some(image) = viewer.borrow_mut().load_image(&direction) {
                main_window.unwrap().set_image(image);
            }
        }
    });

    if let Some(image) = viewer.borrow_mut().load_image("") {
        main_window.set_image(image);
    }

    main_window.run().unwrap();
}
