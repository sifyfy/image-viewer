/*
    The simple image viewer.
    Copyright (C) 2024 Sifi Takashina <sifyfy@sifyfy.dev>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use slint::SharedPixelBuffer;
use std::path::PathBuf;
use std::rc::Rc;
use std::{cell::RefCell, fs};

slint::slint! {
    import { Button, ScrollView } from "std-widgets.slint";

    export component MainWindow inherits Window {
        min-width: 200px;
        min-height: 200px;
        max-width: 10000px;
        max-height: 10000px;

        in-out property <image> image;
        in-out property <float> zoom: 1.0;
        in-out property <string> filename: "";

        callback load_image(string);

        vl := VerticalLayout {
            label := Text {
                text: filename;
                x: 25px;
                height: 50px;
                vertical-alignment: center;
            }

            sv := ScrollView {
                width: parent.width;
                height: parent.height - (label.height + control.height);
                viewport-width: parent.width * zoom;
                viewport-height: (parent.height - (label.height + control.height)) * zoom;

                img := Image {
                    source: image;
                    width: parent.viewport-width;
                    height: parent.viewport-height;
                    image-fit: contain;
                }
            }

            control := HorizontalLayout {
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
                    text: "Zoom Reset";
                    clicked => {
                        zoom = 1.0;
                        sv.viewport-x = 0;
                        sv.viewport-y = 0;
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
    fn new(images: Vec<PathBuf>, current_index: usize) -> Self {
        Self {
            images,
            current_index,
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

    fn filename(&self) -> String {
        self.images
            .get(self.current_index)
            .and_then(|p| p.file_name())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "".to_string())
    }
}

fn main() {
    let path: PathBuf = std::env::args()
        .skip(1)
        .next()
        .unwrap_or(".".to_string())
        .into();

    let (image_path, image_dir) = if path.is_dir() {
        (None, path)
    } else {
        let dir = path
            .parent()
            .map(|p| p.to_owned())
            .unwrap_or_else(|| ".".into());
        (Some(path), dir)
    };

    let images = fs::read_dir(image_dir)
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
        .collect::<Vec<_>>();

    let image_index = if let Some(image_path) = image_path {
        images.iter().position(|p| p == &image_path).unwrap_or(0)
    } else {
        0
    };

    let viewer = Rc::new(RefCell::new(ImageViewer::new(images, image_index)));
    let main_window = MainWindow::new().unwrap();

    main_window.on_load_image({
        let main_window = main_window.as_weak();
        let viewer = Rc::clone(&viewer);
        move |direction| {
            let main_window = main_window.unwrap();
            if let Some((image, filename)) = load_image_and_filename(&viewer, &direction) {
                main_window.set_image(image);
                main_window.set_filename(filename.into());
            } else {
                main_window.set_filename("empty".into());
            }
        }
    });

    if let Some((image, filename)) = load_image_and_filename(&viewer, "") {
        main_window.set_image(image);
        main_window.set_filename(filename.into());
    } else {
        main_window.set_filename("empty".into());
    }

    main_window.run().unwrap();
}

fn load_image_and_filename(
    viewer: &Rc<RefCell<ImageViewer>>,
    direction: &str,
) -> Option<(slint::Image, String)> {
    viewer
        .try_borrow_mut()
        .ok()
        .and_then(|mut v| v.load_image(direction))
        .and_then(|image| {
            let filename = viewer.try_borrow().ok()?.filename();
            Some((image, filename))
        })
}
