mod cli;

use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use clap::Parser;
use cli::{Cli, Start};
use eframe::egui::{self, load::SizedTexture, ColorImage, TextBuffer};
use glob::glob;
use image::{self, ImageBuffer, Rgba, RgbaImage};
use notify::{event::CreateKind, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use rayon::prelude::*;

const ALPHA_CHANNEL: usize = 3;
const FADE_ITERATION_DURATION: u64 = 50;
const FADE_ITERATION_STEPS: u8 = 5;

fn main() {
    let cli = Cli::parse();
    match cli {
        Cli::Start(cmd) => {
            let options = eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
                ..Default::default()
            };

            eframe::run_native(
                "Diashow",
                options,
                Box::new(move |cc| {
                    // This gives us image support:
                    egui_extras::install_image_loaders(&cc.egui_ctx);
                    Ok(Box::new(Diashow::new(cc.egui_ctx.clone(), cmd)))
                }),
            )
            .unwrap();
        }
    };
}

struct Diashow {
    _watcher: RecommendedWatcher,
    change_flag: Arc<AtomicBool>,
    change_sender: Sender<bool>,
    current_alpha: u8,
    current_image: RgbaImage,
    current_index: usize,
    fade_flag: Arc<AtomicBool>,
    fade_iteration_step: u8,
    fade_sender: Sender<bool>,
    image_queue: Arc<Mutex<VecDeque<PathBuf>>>,
    images: Vec<PathBuf>,
    next_image: RgbaImage,
    previous_image: RgbaImage,
    texture: egui::TextureHandle,
}

impl Diashow {
    pub fn new(context: egui::Context, start_parameter: Start) -> Self {
        // Load images and create texture
        let mut images = Self::get_images_paths_from(start_parameter.images.clone());
        let image_queue: Arc<Mutex<VecDeque<PathBuf>>> = Default::default();
        images.sort();

        let texture =
            context.load_texture("Current image", ColorImage::default(), Default::default());

        // Create change timer
        let change_flag = Arc::new(AtomicBool::new(false));
        let (change_sender, change_receiver): (Sender<bool>, Receiver<bool>) = channel();
        Self::start_timer(
            context.clone(),
            change_receiver,
            Duration::from_secs(start_parameter.duration),
            change_flag.clone(),
        );
        // Activate change timer
        change_sender.send(true).unwrap();

        // Create fade timer but do not activate it yet
        let fade_flag = Arc::new(AtomicBool::new(false));
        let (fade_sender, fade_receiver): (Sender<bool>, Receiver<bool>) = channel();
        Self::start_timer(
            context,
            fade_receiver,
            Duration::from_millis(
                start_parameter
                    .fade_iteration_duration
                    .unwrap_or(FADE_ITERATION_DURATION),
            ),
            fade_flag.clone(),
        );

        // Get Starting index and correspoding image
        let start_index = Self::get_start_index(start_parameter.start_index, images.len());
        let current_image_path = &images.clone()[start_index];
        let start_image = Self::load_rgba8_image(&current_image_path);

        let mut watcher = notify::recommended_watcher({
            let image_queue = image_queue.clone();
            move |res: notify::Result<Event>| match res {
                Ok(event) => {
                    let path = event.paths[0].clone();
                    if event.kind == EventKind::Create(CreateKind::File) {
                        println!(
                            "New image created: {}",
                            path.file_name().unwrap().to_string_lossy().as_str()
                        );
                        image_queue.lock().unwrap().push_back(path);
                    }
                }
                Err(e) => println!("watch error: {:?}", e),
            }
        })
        .unwrap();

        // Add a path to be watched. All files and directories at that path and
        // below will be monitored for changes.
        watcher
            .watch(Path::new(&start_parameter.images), RecursiveMode::Recursive)
            .unwrap();

        println!("\n--- Starting diashow ---\n");

        Self {
            change_flag,
            change_sender,
            current_alpha: u8::MAX,
            current_image: start_image.clone(),
            current_index: start_index,
            fade_flag,
            fade_iteration_step: start_parameter
                .fade_iteration_step
                .unwrap_or(FADE_ITERATION_STEPS),
            fade_sender,
            images,
            next_image: Default::default(),
            previous_image: start_image,
            texture,
            _watcher: watcher,
            image_queue,
        }
    }

    fn get_start_index(start_index: Option<i64>, images_len: usize) -> usize {
        match start_index {
            Some(start_index) => {
                assert!(
                    (start_index.abs() as usize) < images_len,
                    "Start index is to low/high!"
                );
                if start_index < 0 {
                    images_len - start_index.abs() as usize
                } else {
                    start_index.abs() as usize
                }
            }
            None => 0,
        }
    }

    fn get_images_paths_from(directory: String) -> Vec<PathBuf> {
        let mut directory_pattern = directory.clone();
        directory_pattern.push_str("/*.jpg");

        println!("Reading \"{}\"", directory_pattern);

        let images: Vec<PathBuf> = glob(&directory_pattern)
            .expect("Failed to construct glob pattern")
            .map(|entry| entry.unwrap().as_path().to_path_buf())
            .collect();
        println!("Found {} files", images.len());
        images
    }

    fn load_rgba8_image(path: &Path) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let image = image::open(path).unwrap();
        image.into_rgba8()
    }

    fn start_timer(
        context: egui::Context,
        rx: Receiver<bool>,
        duration: Duration,
        flag: Arc<AtomicBool>,
    ) {
        thread::spawn(move || loop {
            let _ = rx.recv().unwrap();
            thread::sleep(duration);
            flag.store(true, Ordering::Relaxed);
            context.request_repaint();
        });
    }

    fn iterate_index(&mut self) {
        if self.current_index == self.images.len() - 1 {
            self.current_index = 0;
        } else {
            self.current_index += 1;
        }
    }

    fn set_alpha_channel_to(image: &mut RgbaImage, alpha: u8) {
        image
            .par_pixels_mut()
            .for_each(|pixel| pixel[ALPHA_CHANNEL] = alpha);
    }
}

impl eframe::App for Diashow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.change_flag.load(Ordering::Relaxed) {
                self.change_flag.store(false, Ordering::Relaxed);
                self.iterate_index();

                let mut image_queue_locked = self.image_queue.lock().unwrap();
                let image_path = if image_queue_locked.is_empty() {
                    self.images
                        .get(self.current_index)
                        .expect("Failed to get image frome queue")
                        .clone()
                } else {
                    let image_path = image_queue_locked.pop_front().unwrap();
                    self.images.push(image_path.clone());
                    image_path
                };

                println!(
                    "Next image: {}",
                    image_path.file_name().unwrap().to_string_lossy().as_str()
                );

                self.next_image = Self::load_rgba8_image(&image_path);
                self.previous_image = self.current_image.clone();
                self.current_alpha = 0;
                self.fade_sender.send(true).unwrap();
            }

            if self.fade_flag.load(Ordering::Relaxed) {
                self.fade_flag.store(false, Ordering::Relaxed);

                self.current_alpha += self.fade_iteration_step;

                if self.current_alpha < u8::MAX - self.fade_iteration_step {
                    Self::set_alpha_channel_to(
                        &mut self.previous_image,
                        u8::MAX - self.current_alpha,
                    );
                    Self::set_alpha_channel_to(&mut self.next_image, 0 + self.current_alpha);
                    self.current_image = self.previous_image.clone();
                    image::imageops::overlay(&mut self.current_image, &self.next_image, 0, 0);
                    self.fade_sender.send(true).unwrap();
                } else {
                    self.current_image = self.next_image.clone();
                    Self::set_alpha_channel_to(&mut self.current_image, u8::MAX);
                    self.change_sender.send(true).unwrap();
                }
            }

            let image = egui::ColorImage::from_rgba_unmultiplied(
                [
                    self.current_image.width() as _,
                    self.current_image.height() as _,
                ],
                &self.current_image.as_flat_samples().as_slice(),
            );

            self.texture.set(image, Default::default());
            ui.vertical_centered(|ui| {
                ui.add(
                    egui::Image::from_texture(SizedTexture::from_handle(&self.texture))
                        .shrink_to_fit(),
                );
            });
        });
    }
}
