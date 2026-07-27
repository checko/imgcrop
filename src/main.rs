#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use std::path::{Path, PathBuf};

use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};
use image::{DynamicImage, GenericImageView, ImageReader};

const HANDLE_RADIUS: f32 = 7.0;
const MIN_CROP: f32 = 0.01;

fn main() -> eframe::Result<()> {
    // Registers HEIC/HEIF and AVIF decoders supplied by the bundled libheif runtime.
    libheif_rs::integration::image::register_all_decoding_hooks();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Image Cropper")
            .with_inner_size([1120.0, 760.0])
            .with_min_inner_size([820.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Image Cropper",
        options,
        Box::new(|cc| Ok(Box::new(CropApp::new(cc)))),
    )
}

#[derive(Clone, Copy, Debug)]
struct Crop {
    min: Pos2,
    max: Pos2,
}

impl Crop {
    fn full() -> Self {
        Self {
            min: Pos2::new(0.1, 0.1),
            max: Pos2::new(0.9, 0.9),
        }
    }

    fn from_points(first: Pos2, second: Pos2) -> Self {
        Self {
            min: Pos2::new(first.x.min(second.x), first.y.min(second.y)),
            max: Pos2::new(first.x.max(second.x), first.y.max(second.y)),
        }
        .clamped()
    }

    fn clamped(self) -> Self {
        let min_x = self.min.x.clamp(0.0, 1.0 - MIN_CROP);
        let min_y = self.min.y.clamp(0.0, 1.0 - MIN_CROP);
        let max_x = self.max.x.clamp(min_x + MIN_CROP, 1.0);
        let max_y = self.max.y.clamp(min_y + MIN_CROP, 1.0);
        Self {
            min: Pos2::new(min_x, min_y),
            max: Pos2::new(max_x, max_y),
        }
    }

    fn width(self) -> f32 {
        self.max.x - self.min.x
    }

    fn height(self) -> f32 {
        self.max.y - self.min.y
    }

    fn translate(self, delta: Vec2) -> Self {
        let mut min = self.min + delta;
        let mut max = self.max + delta;
        if min.x < 0.0 {
            max.x -= min.x;
            min.x = 0.0;
        }
        if min.y < 0.0 {
            max.y -= min.y;
            min.y = 0.0;
        }
        if max.x > 1.0 {
            min.x -= max.x - 1.0;
            max.x = 1.0;
        }
        if max.y > 1.0 {
            min.y -= max.y - 1.0;
            max.y = 1.0;
        }
        Self { min, max }
    }
}

#[derive(Clone, Copy)]
enum Corner {
    NorthWest,
    NorthEast,
    SouthWest,
    SouthEast,
}

enum DragMode {
    Draw { start: Pos2 },
    Move { start: Pos2, crop: Crop },
    Resize { corner: Corner, anchor: Pos2 },
}

struct LoadedImage {
    path: PathBuf,
    image: DynamicImage,
    texture: egui::TextureHandle,
    crop: Crop,
}

struct CropApp {
    loaded: Option<LoadedImage>,
    drag_mode: Option<DragMode>,
    message: String,
    is_error: bool,
}

impl CropApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self {
            loaded: None,
            drag_mode: None,
            message: "Open or drop an image to begin.".into(),
            is_error: false,
        };
        if let Some(path) = std::env::args_os().nth(1).map(PathBuf::from) {
            app.open_path(&cc.egui_ctx, path);
        }
        app
    }

    fn select_file(&mut self, ctx: &egui::Context) {
        // Do not apply an extension filter here. Some Linux native-dialog backends
        // treat patterns as case-sensitive, which hides uppercase camera files such
        // as IMG_7690.HEIC. Decoding below is the authoritative format check.
        let file = rfd::FileDialog::new().pick_file();
        if let Some(path) = file {
            self.open_path(ctx, path);
        }
    }

    fn open_path(&mut self, ctx: &egui::Context, path: PathBuf) {
        match load_image(&path) {
            Ok(image) => {
                let rgba = image.to_rgba8();
                let size = [rgba.width() as usize, rgba.height() as usize];
                let texture = ctx.load_texture(
                    path.display().to_string(),
                    egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw()),
                    egui::TextureOptions::LINEAR,
                );
                self.loaded = Some(LoadedImage {
                    path: path.clone(),
                    image,
                    texture,
                    crop: Crop::full(),
                });
                self.drag_mode = None;
                self.message = format!(
                    "Opened {}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                );
                self.is_error = false;
            }
            Err(error) => {
                self.message = format!("Could not open {}: {error}", path.display());
                self.is_error = true;
            }
        }
    }

    fn save_crop(&mut self) {
        let Some(loaded) = &self.loaded else {
            return;
        };
        match save_crop(loaded) {
            Ok(path) => {
                self.message = format!(
                    "Saved {}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                );
                self.is_error = false;
            }
            Err(error) => {
                self.message = format!("Could not save crop: {error}");
                self.is_error = true;
            }
        }
    }

    fn dropped_files(&mut self, ctx: &egui::Context) {
        let files = ctx.input(|input| input.raw.dropped_files.clone());
        if let Some(path) = files.into_iter().find_map(|file| file.path) {
            self.open_path(ctx, path);
        }
    }

    fn draw_empty_state(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() * 0.22);
            ui.heading("Crop any image");
            ui.add_space(8.0);
            ui.label("Drop an image here, or choose a file from your computer.");
            ui.add_space(16.0);
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new("Open File")
                            .size(18.0)
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(96, 78, 220))
                    .min_size(Vec2::new(180.0, 48.0)),
                )
                .clicked()
            {
                self.select_file(&ctx);
            }
            ui.add_space(14.0);
            ui.small("HEIC, PNG, JPEG, WebP, TIFF, GIF, BMP, AVIF, and ICO");
        });
    }

    fn draw_editor(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let (file_name, dimensions, crop_size) = {
            let loaded = self.loaded.as_ref().expect("editor requires loaded image");
            let (width, height) = loaded.image.dimensions();
            (
                loaded
                    .path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                format!("{width} × {height} px"),
                format!(
                    "{} × {} px",
                    (loaded.crop.width() * width as f32).round() as u32,
                    (loaded.crop.height() * height as f32).round() as u32
                ),
            )
        };

        egui::Panel::left("details")
            .resizable(false)
            .default_size(230.0)
            .show(ui, |ui| {
                ui.add_space(12.0);
                ui.strong("CURRENT IMAGE");
                ui.add_space(8.0);
                ui.label(file_name);
                ui.small(dimensions);
                ui.separator();
                ui.strong("CROP SELECTION");
                ui.add_space(8.0);
                ui.label(crop_size);
                ui.separator();
                ui.strong("HOW TO CROP");
                ui.add_space(8.0);
                ui.small("Drag on the image to make a selection. Drag inside it to move it, or drag any corner handle to resize it.");
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    if ui.button("Open another file").clicked() {
                        self.select_file(&ctx);
                    }
                });
            });

        egui::Panel::bottom("status")
            .resizable(false)
            .exact_size(64.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let color = if self.is_error {
                        Color32::from_rgb(186, 43, 72)
                    } else {
                        Color32::from_gray(92)
                    };
                    ui.colored_label(color, &self.message);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let save_button = egui::Button::new(
                            egui::RichText::new("Save Crop")
                                .size(18.0)
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(Color32::from_rgb(19, 133, 112))
                        .min_size(Vec2::new(170.0, 46.0));
                        if ui.add_enabled(self.loaded.is_some(), save_button).clicked() {
                            self.save_crop();
                        }
                    });
                });
            });

        egui::CentralPanel::default().show(ui, |ui| self.draw_canvas(ui));
    }

    fn draw_canvas(&mut self, ui: &mut egui::Ui) {
        let Some(loaded) = self.loaded.as_ref() else {
            return;
        };
        let texture_id = loaded.texture.id();
        let image_size = loaded.image.dimensions();
        let mut crop = loaded.crop;
        let available = ui.available_rect_before_wrap();
        let scale = (available.width() / image_size.0 as f32)
            .min(available.height() / image_size.1 as f32)
            .min(1.0);
        let display = Vec2::new(image_size.0 as f32 * scale, image_size.1 as f32 * scale);
        let image_rect = Rect::from_center_size(available.center(), display);
        let response = ui.allocate_rect(image_rect, Sense::click_and_drag());
        let painter = ui.painter_at(image_rect);
        painter.image(
            texture_id,
            image_rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );

        self.handle_crop_input(ui.ctx(), &response, image_rect, &mut crop);
        paint_crop(&painter, image_rect, crop);
        if let Some(loaded) = self.loaded.as_mut() {
            loaded.crop = crop;
        }
    }

    fn handle_crop_input(
        &mut self,
        ctx: &egui::Context,
        response: &egui::Response,
        rect: Rect,
        crop: &mut Crop,
    ) {
        let pointer = ctx.input(|input| input.pointer.interact_pos());
        if response.drag_started() {
            if let Some(position) = pointer.filter(|point| rect.contains(*point)) {
                let normalized = to_normalized(rect, position);
                self.drag_mode = Some(match corner_at(rect, *crop, position) {
                    Some((corner, anchor)) => DragMode::Resize { corner, anchor },
                    None if crop_rect(rect, *crop).contains(position) => DragMode::Move {
                        start: normalized,
                        crop: *crop,
                    },
                    None => DragMode::Draw { start: normalized },
                });
            }
        }

        if response.dragged() {
            if let (Some(mode), Some(position)) = (&self.drag_mode, pointer) {
                let point = to_normalized(rect, position);
                match *mode {
                    DragMode::Draw { start } => *crop = Crop::from_points(start, point),
                    DragMode::Move {
                        start,
                        crop: original,
                    } => *crop = original.translate(point - start),
                    DragMode::Resize { corner, anchor } => {
                        let adjusted = match corner {
                            Corner::NorthWest
                            | Corner::NorthEast
                            | Corner::SouthWest
                            | Corner::SouthEast => point,
                        };
                        *crop = Crop::from_points(anchor, adjusted);
                    }
                }
            }
        }

        if !ctx.input(|input| input.pointer.primary_down()) {
            self.drag_mode = None;
        }
    }
}

impl eframe::App for CropApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.dropped_files(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        egui::Panel::top("toolbar")
            .resizable(false)
            .exact_size(72.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Image Cropper");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("Open File")
                                        .size(18.0)
                                        .strong()
                                        .color(Color32::WHITE),
                                )
                                .fill(Color32::from_rgb(96, 78, 220))
                                .min_size(Vec2::new(170.0, 46.0)),
                            )
                            .clicked()
                        {
                            self.select_file(&ctx);
                        }
                    });
                });
            });

        if self.loaded.is_some() {
            self.draw_editor(ui);
        } else {
            self.draw_empty_state(ui);
        }
    }
}

fn crop_rect(image_rect: Rect, crop: Crop) -> Rect {
    Rect::from_min_max(
        Pos2::new(
            image_rect.left() + image_rect.width() * crop.min.x,
            image_rect.top() + image_rect.height() * crop.min.y,
        ),
        Pos2::new(
            image_rect.left() + image_rect.width() * crop.max.x,
            image_rect.top() + image_rect.height() * crop.max.y,
        ),
    )
}

fn to_normalized(rect: Rect, point: Pos2) -> Pos2 {
    Pos2::new(
        ((point.x - rect.left()) / rect.width()).clamp(0.0, 1.0),
        ((point.y - rect.top()) / rect.height()).clamp(0.0, 1.0),
    )
}

fn corner_at(image_rect: Rect, crop: Crop, point: Pos2) -> Option<(Corner, Pos2)> {
    let rect = crop_rect(image_rect, crop);
    let corners = [
        (Corner::NorthWest, rect.left_top(), crop.max),
        (
            Corner::NorthEast,
            rect.right_top(),
            Pos2::new(crop.min.x, crop.max.y),
        ),
        (
            Corner::SouthWest,
            rect.left_bottom(),
            Pos2::new(crop.max.x, crop.min.y),
        ),
        (Corner::SouthEast, rect.right_bottom(), crop.min),
    ];
    corners
        .into_iter()
        .find(|(_, handle, _)| handle.distance(point) <= HANDLE_RADIUS * 1.8)
        .map(|(corner, _, anchor)| (corner, anchor))
}

fn paint_crop(painter: &egui::Painter, image_rect: Rect, crop: Crop) {
    let selection = crop_rect(image_rect, crop);
    let shade = Color32::from_black_alpha(118);
    painter.rect_filled(
        Rect::from_min_max(image_rect.min, Pos2::new(image_rect.max.x, selection.top())),
        0.0,
        shade,
    );
    painter.rect_filled(
        Rect::from_min_max(
            Pos2::new(image_rect.left(), selection.bottom()),
            image_rect.max,
        ),
        0.0,
        shade,
    );
    painter.rect_filled(
        Rect::from_min_max(
            Pos2::new(image_rect.left(), selection.top()),
            Pos2::new(selection.left(), selection.bottom()),
        ),
        0.0,
        shade,
    );
    painter.rect_filled(
        Rect::from_min_max(
            Pos2::new(selection.right(), selection.top()),
            Pos2::new(image_rect.right(), selection.bottom()),
        ),
        0.0,
        shade,
    );
    painter.rect_stroke(
        selection,
        0.0,
        Stroke::new(2.0, Color32::WHITE),
        StrokeKind::Outside,
    );

    for point in [
        selection.left_top(),
        selection.right_top(),
        selection.left_bottom(),
        selection.right_bottom(),
    ] {
        painter.circle_filled(point, HANDLE_RADIUS, Color32::WHITE);
        painter.circle_stroke(
            point,
            HANDLE_RADIUS,
            Stroke::new(2.0, Color32::from_rgb(101, 88, 232)),
        );
    }
}

fn load_image(path: &Path) -> Result<DynamicImage, String> {
    ImageReader::open(path)
        .map_err(|error| error.to_string())?
        .with_guessed_format()
        .map_err(|error| error.to_string())?
        .decode()
        .map_err(|error| error.to_string())
}

fn save_crop(loaded: &LoadedImage) -> Result<PathBuf, String> {
    let (image_width, image_height) = loaded.image.dimensions();
    let x = (loaded.crop.min.x * image_width as f32).round() as u32;
    let y = (loaded.crop.min.y * image_height as f32).round() as u32;
    let width = ((loaded.crop.width() * image_width as f32).round() as u32)
        .max(1)
        .min(image_width - x);
    let height = ((loaded.crop.height() * image_height as f32).round() as u32)
        .max(1)
        .min(image_height - y);
    let cropped = loaded.image.crop_imm(x, y, width, height);

    let original_extension = loaded
        .path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("png")
        .to_ascii_lowercase();
    let extension = match original_extension.as_str() {
        "heic" | "heif" | "avif" | "svg" | "jp2" => "jpg",
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "tif" | "tiff" | "ico" => {
            &original_extension
        }
        _ => "png",
    };
    let stem = loaded
        .path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("image");
    let parent = loaded.path.parent().unwrap_or_else(|| Path::new("."));
    let mut number = 1;
    let output = loop {
        let candidate = parent.join(format!("{stem}crop{number}.{extension}"));
        if !candidate.exists() {
            break candidate;
        }
        number += 1;
    };

    cropped.save(&output).map_err(|error| error.to_string())?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crop_normalizes_and_clamps_points() {
        let crop = Crop::from_points(Pos2::new(0.9, 0.8), Pos2::new(-0.2, 1.3));
        assert_eq!(crop.min, Pos2::new(0.0, 0.8));
        assert_eq!(crop.max, Pos2::new(0.9, 1.0));
    }

    #[test]
    fn crop_translation_stays_inside_image() {
        let crop = Crop::from_points(Pos2::new(0.2, 0.2), Pos2::new(0.8, 0.8));
        let moved = crop.translate(Vec2::new(0.5, -0.5));
        assert!((moved.min.x - 0.4).abs() < 0.000_01);
        assert!((moved.min.y - 0.0).abs() < 0.000_01);
        assert!((moved.max.x - 1.0).abs() < 0.000_01);
        assert!((moved.max.y - 0.6).abs() < 0.000_01);
    }
}
