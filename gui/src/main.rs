/*
 * Copyright (c) 2022, david072
 *
 * SPDX-License-Identifier: Apache-2.0
 */

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::sync::Arc;

use eframe::{CreationContext, Frame, Storage};
use eframe::egui;
use eframe::egui::text_edit::CursorRange;
use eframe::epaint::text::cursor::Cursor;
use egui::*;

use calculator::{Calculator, Color, ColorSegment, Function as CalcFn, ResultData, Verbosity};

use crate::widgets::*;

mod widgets;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const FONT_SIZE: f32 = 16.0;
const FONT_ID: FontId = FontId::monospace(FONT_SIZE);
const FOOTER_FONT_SIZE: f32 = 14.0;
const TEXT_EDIT_MARGIN: Vec2 = Vec2::new(4.0, 2.0);
const ERROR_COLOR: Color = Color::RED;

const INPUT_TEXT_EDIT_ID: &str = "input-text-edit";

#[cfg(feature = "experimental")]
fn app_key() -> String {
    eframe::APP_KEY.to_string() + "-experimental"
}

#[cfg(not(feature = "experimental"))]
fn app_key() -> String {
    eframe::APP_KEY.to_string()
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let icon = if cfg!(windows) {
        if cfg!(debug_assertions) {
            image::open("./gui/assets/app_icon_256.ico").ok().map(|i| i.to_rgba8())
        } else {
            match std::env::current_exe() {
                Ok(mut path) => {
                    path.pop();
                    path = path.join("app_icon_256.ico");

                    image::open(path).ok().map(|i| i.to_rgba8())
                }
                Err(_) => None,
            }
        }
    } else { None };

    let options = eframe::NativeOptions {
        initial_window_size: Some(Vec2::new(500.0, 400.0)),
        icon_data: {
            if let Some(icon) = icon {
                let (icon_width, icon_height) = icon.dimensions();
                Some(eframe::IconData {
                    rgba: icon.into_raw(),
                    width: icon_width,
                    height: icon_height,
                })
            } else { None }
        },
        ..Default::default()
    };
    eframe::run_native(
        "Funcially",
        options,
        Box::new(|cc| Box::new(App::new(cc))),
    );
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Make sure panics are logged using `console.error`
    console_error_panic_hook::set_once();
    // Redirect tracing to console.log, ...
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();
    eframe::start_web(
        "the_canvas_id",
        web_options,
        Box::new(|cc| Box::new(App::new(cc))),
    ).expect("Failed to start eframe");
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Function(String, usize, #[serde(skip)] CalcFn);

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Line {
    Empty,
    Line {
        #[serde(skip)]
        output_text: String,
        #[serde(skip)]
        color_segments: Vec<ColorSegment>,
        /// `name`, `argument count`, `Function`.
        ///
        /// Store the function to be able to show redefinitions as well.
        function: Option<Function>,
        show_in_plot: bool,
        #[serde(skip)]
        is_error: bool,
    },
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
struct App<'a> {
    #[serde(skip)]
    calculator: Calculator<'a>,

    source: String,
    #[serde(skip)]
    source_old: String,
    lines: Vec<Line>,
    line_numbers_text: String,

    #[serde(skip)]
    is_ui_enabled: bool,

    is_plot_open: bool,
    is_help_open: bool,
    #[cfg(target_arch = "wasm32")]
    is_download_open: bool,
    is_settings_open: bool,

    is_debug_info_open: bool,
    debug_information: Option<String>,

    use_thousands_separator: bool,

    #[serde(skip)]
    first_frame: bool,
    #[serde(skip)]
    input_should_request_focus: bool,
    #[serde(skip)]
    input_text_cursor_range: CursorRange,
    #[serde(skip)]
    bottom_text: String,
    #[serde(skip)]
    cached_help_window_color_segments: Vec<Vec<ColorSegment>>,
}

impl Default for App<'_> {
    fn default() -> Self {
        App {
            calculator: Calculator::default(),
            source_old: String::new(),
            source: String::new(),
            lines: Vec::new(),
            line_numbers_text: "1".to_string(),
            first_frame: true,
            input_should_request_focus: true,
            is_ui_enabled: true,
            is_plot_open: false,
            is_help_open: false,
            #[cfg(target_arch = "wasm32")]
            is_download_open: false,
            is_settings_open: false,
            is_debug_info_open: false,
            debug_information: None,
            use_thousands_separator: false,
            input_text_cursor_range: CursorRange::one(Cursor::default()),
            bottom_text: format!("v{}", VERSION),
            cached_help_window_color_segments: Vec::new(),
        }
    }
}

impl App<'_> {
    fn new(cc: &CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(Visuals::dark());

        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, &app_key()).unwrap_or_default();
        }

        App::default()
    }

    fn calculate(&mut self, str: &str) -> Line {
        if str.trim().is_empty() { return Line::Empty; }

        let result = self.calculator.calculate(str);

        let mut function: Option<Function> = None;
        let mut color_segments: Vec<ColorSegment> = Vec::new();
        let mut is_error: bool = false;

        let output_text = match result {
            Ok(res) => {
                color_segments = res.color_segments;
                match res.data {
                    ResultData::Number { result, unit, format } => {
                        format!("{}{}", format.format(result, self.use_thousands_separator), unit.unwrap_or_default())
                    }
                    ResultData::Boolean(b) => (if b { "True" } else { "False" }).to_string(),
                    ResultData::Function { name, arg_count, function: f } => {
                        function = Some(Function(name, arg_count, f));
                        String::new()
                    }
                    ResultData::Nothing => String::new(),
                }
            }
            Err(e) => {
                is_error = true;
                color_segments.push(ColorSegment::new(e.start..e.end, ERROR_COLOR));
                format!("{}", e.error)
            }
        };

        Line::Line {
            output_text,
            function,
            color_segments,
            is_error,
            show_in_plot: false,
        }
    }

    fn get_debug_info_for_current_line(&mut self) {
        let input_text_paragraph = self.input_text_cursor_range.primary.pcursor.paragraph;
        for (i, line) in self.source.lines().enumerate() {
            if i != input_text_paragraph { continue; }

            self.debug_information = match self.calculator.get_debug_info(line, Verbosity::Ast) {
                Ok(info) => Some(info),
                Err(e) => Some(format!("Error generating debug information: {}, {}..{}", e.error, e.start, e.end))
            };
            break;
        }
    }

    fn update_lines(&mut self, galley: Arc<Galley>) {
        if self.source == self.source_old { return; }

        self.source_old = self.source.clone();
        // Since we re-calculate everything from the beginning,
        // we need to start with a fresh environment
        self.calculator.reset();

        let mut functions = self.lines.iter()
            .filter(|l| {
                match l {
                    Line::Line { show_in_plot, .. } => *show_in_plot,
                    _ => false,
                }
            })
            .map(|l| {
                if let Line::Line { function: Some(Function(name, ..)), show_in_plot, .. } = l {
                    (name.clone(), *show_in_plot)
                } else { unreachable!() }
            })
            .collect::<Vec<_>>();
        self.lines.clear();
        self.line_numbers_text.clear();

        if galley.rows.is_empty() {
            self.line_numbers_text = "1".to_string();
            return;
        }

        let mut line = String::new();
        let mut line_index = 1usize;
        let mut did_add_line_index = false;
        for (i, row) in galley.rows.iter().enumerate() {
            line += row.glyphs.iter().map(|g| g.chr).collect::<String>().as_str();

            if !row.ends_with_newline {
                if !did_add_line_index {
                    self.line_numbers_text += &line_index.to_string();
                    self.line_numbers_text.push('\n');
                    did_add_line_index = true;
                    line_index += 1;
                }

                if i != galley.rows.len() - 1 {
                    self.lines.push(Line::Empty);
                }
                continue;
            } else {
                if !did_add_line_index {
                    self.line_numbers_text += &line_index.to_string();
                    line_index += 1;
                }
                self.line_numbers_text.push('\n');
                did_add_line_index = false;

                if !line.starts_with('#') {
                    let actual_line = if let Some(index) = line.find('#') {
                        &line[0..index]
                    } else { &line };

                    let mut res = self.calculate(actual_line);
                    if let Line::Line { function: Some(Function(name, ..)), show_in_plot, .. } = &mut res {
                        if let Some(i) = functions.iter().position(|(n, _)| n == name) {
                            *show_in_plot = functions[i].1;
                            functions.remove(i);
                        }
                    }
                    self.lines.push(res);
                } else {
                    self.lines.push(Line::Empty);
                }

                line.clear();
            }
        }

        if !line.is_empty() && !line.starts_with('#') {
            let actual_line = if let Some(index) = line.find('#') {
                &line[0..index]
            } else { &line };

            let mut res = self.calculate(actual_line);
            if let Line::Line { function: Some(Function(name, ..)), show_in_plot, .. } = &mut res {
                if let Some(i) = functions.iter().position(|(n, _)| n == name) {
                    *show_in_plot = functions[i].1;
                    functions.remove(i);
                }
            }
            self.lines.push(res);
        }

        if self.line_numbers_text.is_empty() {
            self.line_numbers_text = "1".to_string();
        }
    }

    fn plot_panel(&mut self, ctx: &Context) {
        if FullScreenPlot::is_fullscreen(ctx) { return; }

        SidePanel::right("plot_panel")
            .resizable(self.is_ui_enabled)
            .show(ctx, |ui| {
                ui.set_enabled(self.is_ui_enabled);

                let response = plot(ui, &self.lines, &self.calculator);
                ui.allocate_ui_at_rect(
                    response.response.rect.shrink(10.0),
                    |ui| {
                        ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                            if ui.small_button("Fullscreen").clicked() {
                                FullScreenPlot::set_fullscreen(ui.ctx(), true);
                            }
                        });
                    },
                );
            });
    }

    fn help_window(&mut self, ctx: &Context) {
        let is_help_open = &mut self.is_help_open;
        let color_segments = &mut self.cached_help_window_color_segments;
        Window::new("Help")
            .open(is_help_open)
            .vscroll(true)
            .hscroll(true)
            .enabled(self.is_ui_enabled)
            .show(ctx, |ui| {
                build_help(ui, FONT_ID, color_segments);
            });
    }

    #[cfg(target_arch = "wasm32")]
    fn download_window(&mut self, ctx: &Context) {
        Window::new("Download")
            .open(&mut self.is_download_open)
            .collapsible(false)
            .resizable(false)
            .enabled(self.is_ui_enabled)
            .show(ctx, |ui| {
                ui.heading("Desktop App");
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("If you're on desktop, you can download the ");
                    ui.hyperlink_to("desktop app", "https://github.com/david072/funcially/releases");
                    ui.label(".");
                });
                ui.separator();

                ui.heading("Web App");
                ui.label("You can make this website available offline through Chrome.");
                ui.add_space(4.0);
                ui.heading("Chrome:");
                ui.label("Desktop: Click the download button to the right of the address bar and follow the instructions.");
                ui.add_space(2.0);
                ui.label("Mobile: Click the three dots to the right of the address bar and click the 'Install app' button.");
                ui.add_space(4.0);
                ui.heading("Safari (iPad / iPhone):");
                ui.label("Click the share button, either at the bottom of the screen or next to the address bar, \
                    click the 'Add to Home Screen' button and follow the instructions.");
                ui.add_space(2.0);
            });
    }

    fn settings_window(&mut self, ctx: &Context) {
        Window::new("Settings")
            .open(&mut self.is_settings_open)
            .vscroll(true)
            .resizable(false)
            .enabled(self.is_ui_enabled)
            .show(ctx, |ui| {
                if ui.checkbox(&mut self.use_thousands_separator, "Use thousands separator").clicked() {
                    // Make update_lines() refresh on the next frame, since now source and source_old are not the same
                    self.source_old.clear();
                }
                CollapsingHeader::new("Debug").default_open(true).show(ui, |ui| {
                    let mut debug_on_hover = ui.ctx().debug_on_hover();
                    ui.checkbox(&mut debug_on_hover, "Debug On Hover");
                    ui.ctx().set_debug_on_hover(debug_on_hover);

                    let mut tesselation_options = ui.ctx().options().tessellation_options;
                    ui.checkbox(&mut tesselation_options.debug_paint_clip_rects, "Paint clip rectangles");
                    ui.checkbox(&mut tesselation_options.debug_paint_text_rects, "Paint text bounds");
                    *ui.ctx().tessellation_options() = tesselation_options;
                });
                ui.hyperlink_to("Source code", "https://github.com/david072/funcially");
            });
    }

    fn show_debug_information(&mut self, ctx: &Context) {
        let debug_information = &mut self.debug_information;

        Window::new("Debug Information")
            .open(&mut self.is_debug_info_open)
            .vscroll(true)
            .enabled(self.is_ui_enabled)
            .show(ctx, |ui| {
                if let Some(debug_information) = debug_information {
                    if ui.button("📋").clicked() {
                        ui.output().copied_text = debug_information.clone();
                    }

                    TextEdit::multiline(debug_information)
                        .interactive(false)
                        .show(ui);
                }
            });
    }

    fn handle_shortcuts(&mut self, ui: &Ui, cursor_range: CursorRange) {
        let mut copied_text = None;
        let mut set_line_picker_open = false;
        for event in &ui.input().events {
            if let Event::Key { key, pressed, modifiers } = event {
                if !*pressed { continue; }
                match key {
                    Key::N if modifiers.command && modifiers.alt => self.toggle_commentation(cursor_range),
                    Key::B if modifiers.command => self.surround_selection_with_brackets(cursor_range),
                    Key::C if modifiers.command && modifiers.shift => self.copy_result(cursor_range, &mut copied_text),
                    Key::L if modifiers.command && modifiers.alt => self.format_source(),
                    Key::G if modifiers.command => {
                        self.is_ui_enabled = false;
                        set_line_picker_open = true;
                    }
                    _ => {}
                }
            }
        }

        if set_line_picker_open {
            LinePickerDialog::set_open(ui.ctx(), true);
        }

        if let Some(copied) = copied_text {
            ui.output().copied_text = copied;
        }
    }

    fn toggle_commentation(&mut self, cursor_range: CursorRange) {
        let start_line = cursor_range.primary.pcursor.paragraph;
        let end_line = cursor_range.secondary.pcursor.paragraph;

        let has_uncommented_line = self.source.lines()
            .skip(start_line)
            .filter(|l| !l.is_empty())
            .take(if end_line == 0 { 1 } else { end_line })
            .any(|l| !l.trim_start().starts_with('#'));

        // If there is an uncommented line, we even the lines out by commenting
        // uncommented lines too.
        // Otherwise, we uncomment, since all lines are commented.

        let mut new_source = String::new();
        let line_count = self.source.lines().count();
        for (i, line) in self.source.lines().enumerate() {
            if i < start_line || i > end_line {
                new_source += line;
                if i != line_count - 1 { new_source.push('\n'); }
                continue;
            } else if line.is_empty() {
                if i != line_count - 1 { new_source.push('\n'); }
                continue;
            }

            let trimmed = line.trim_start();
            let offset = line.len() - trimmed.len();

            if has_uncommented_line {
                if !line.trim_start().starts_with('#') {
                    for _ in 0..offset { new_source.push(' '); }
                    new_source.push('#');
                    new_source += &line[offset..];
                    if i != line_count - 1 { new_source.push('\n'); }
                } else {
                    new_source += line;
                    if i != line_count - 1 { new_source.push('\n'); }
                }
            } else {
                for _ in 0..offset { new_source.push(' '); }
                new_source += line.chars()
                    .skip(offset + 1)
                    .collect::<String>().as_str();
                if i != line_count - 1 { new_source.push('\n'); }
            }
        }

        self.source = new_source;
    }

    fn surround_selection_with_brackets(&mut self, cursor_range: CursorRange) {
        // Check that we have a range spanning only one line
        let primary = &cursor_range.primary.pcursor;
        let secondary = &cursor_range.secondary.pcursor;

        if (*primary == *secondary) || (primary.paragraph != secondary.paragraph) {
            return;
        }

        let mut new_source = String::new();
        let line_count = self.source.lines().count();
        for (i, line) in self.source.lines().enumerate() {
            if i != primary.paragraph {
                new_source += line;
                new_source.push('\n');
                continue;
            }

            let start = std::cmp::min(primary.offset, secondary.offset);
            let end = std::cmp::max(primary.offset, secondary.offset);

            let mut line = line.to_string();
            line.insert(start, '(');
            line.insert(end + 1, ')');
            new_source += line.as_str();
            if i != line_count - 1 {
                new_source.push('\n');
            }
        }

        self.source = new_source;
    }

    fn copy_result(&mut self, cursor_range: CursorRange, copied_text: &mut Option<String>) {
        let line = cursor_range.primary.rcursor.row;
        if let Some(Line::Line { output_text, .. }) = self.lines.get(line) {
            *copied_text = Some(output_text.to_owned());
            // Taking the ui.output() lock here leads to a deadlock (if called from
            // handle_shortcuts()), so we have to write it to the variable passed in.
        }
    }

    fn format_source(&mut self) {
        let mut new_source = String::new();

        let line_count = self.source.lines().count();
        for (i, line) in self.source.lines().enumerate() {
            if !line.is_empty() {
                match self.calculator.format(line) {
                    Ok(fmt) => new_source += &fmt,
                    Err(_) => new_source += line,
                }
            }

            if i != line_count - 1 {
                new_source.push('\n');
            }
        }

        self.source = new_source;
    }

    fn line_picker_dialog(&mut self, ctx: &Context) {
        let result = LinePickerDialog::new(
            FONT_ID,
            Id::new(INPUT_TEXT_EDIT_ID),
            &self.source,
        ).show(ctx);

        if let Some(picked) = result {
            self.is_ui_enabled = true;
            if picked {
                self.input_should_request_focus = true;
            }
        }
    }
}

impl eframe::App for App<'_> {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        if !self.cached_help_window_color_segments.is_empty() && !self.is_help_open {
            self.cached_help_window_color_segments.clear();
        }
        if !self.is_debug_info_open { self.debug_information = None; }

        FullScreenPlot::new(
            ctx.available_rect().size(),
            &self.lines,
            &self.calculator,
        ).maybe_show(ctx);

        TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.set_enabled(self.is_ui_enabled);

            menu::bar(ui, |ui| {
                let cmd_string = if matches!(std::env::consts::OS, "macos" | "ios") { "⌘" } else { "Ctrl" };
                let shortcut = |keys: &str| { format!("{cmd_string}+{keys}") };

                ui.menu_button("Edit", |ui| {
                    if shortcut_button(ui, "Surround selection with brackets", &shortcut("B")).clicked() {
                        self.surround_selection_with_brackets(self.input_text_cursor_range);
                        ui.close_menu();
                    }
                    if shortcut_button(ui, "(Un)Comment selected lines", &shortcut("Alt+N")).clicked() {
                        self.toggle_commentation(self.input_text_cursor_range);
                        ui.close_menu();
                    }
                    if shortcut_button(ui, "Copy result", &shortcut("Shift+C")).clicked() {
                        let mut copied_text = None;
                        self.copy_result(self.input_text_cursor_range, &mut copied_text);
                        if let Some(copied) = copied_text {
                            ui.output().copied_text = copied;
                        }
                        ui.close_menu();
                    }
                    if shortcut_button(ui, "Format input", &shortcut("Shift+L")).clicked() {
                        self.format_source();
                        ui.close_menu();
                    }
                });

                ui.menu_button("Navigate", |ui| {
                    if shortcut_button(ui, "Go to Line", &shortcut("G")).clicked() {
                        LinePickerDialog::set_open(ctx, true);
                        self.is_ui_enabled = false;
                        ui.close_menu();
                    }
                });

                if ui.button(if self.is_plot_open { "Close Plot" } else { "Open Plot" }).clicked() {
                    self.is_plot_open = !self.is_plot_open;
                }
                if ui.button("Help").clicked() {
                    self.is_help_open = !self.is_help_open;
                }
                #[cfg(target_arch = "wasm32")]
                if ui.button("Download").clicked() {
                    self.is_download_open = !self.is_download_open;
                }
                if ui.button("Settings").clicked() {
                    self.is_settings_open = !self.is_settings_open;
                }

                ui.menu_button("Debug", |ui| {
                    if ui.button("Print Debug Information for current line").clicked() {
                        self.get_debug_info_for_current_line();
                        self.is_debug_info_open = true;
                    }
                });
            })
        });

        TopBottomPanel::bottom("bottom_bar").show(ctx, |ui| {
            ui.set_enabled(self.is_ui_enabled);

            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                ui.label(RichText::new(&self.bottom_text).font(FontId::proportional(FOOTER_FONT_SIZE)));
            });
        });

        // We wait for the second frame to have the lines updated if they've been loaded on startup
        if !self.first_frame && self.is_plot_open { self.plot_panel(ctx); }

        if self.is_help_open { self.help_window(ctx); }
        #[cfg(target_arch = "wasm32")]
        if self.is_download_open { self.download_window(ctx); }
        if self.is_settings_open { self.settings_window(ctx); }
        if self.is_debug_info_open { self.show_debug_information(ctx); }

        CentralPanel::default().show(ctx, |ui| {
            ui.set_enabled(self.is_ui_enabled);

            let rows = ((ui.available_height() - TEXT_EDIT_MARGIN.y - FOOTER_FONT_SIZE) / FONT_SIZE) as usize;

            ScrollArea::vertical().show(ui, |ui| {
                ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
                    let char_width = ui.fonts().glyph_width(&FONT_ID, '0') + 2.0;

                    let longest_row_chars = self.line_numbers_text.lines()
                        .last()
                        .map(str::len)
                        .unwrap_or_default() as f32;

                    TextEdit::multiline(&mut self.line_numbers_text)
                        .frame(false)
                        .font(FontSelection::from(FONT_ID))
                        .interactive(false)
                        .desired_width(longest_row_chars * char_width)
                        .desired_rows(rows)
                        .margin(vec2(0.0, 2.0))
                        .show(ui);

                    let input_width = ui.available_width() * (2.0 / 3.0);

                    let lines = &mut self.lines;
                    let output = TextEdit::multiline(&mut self.source)
                        .id(Id::new(INPUT_TEXT_EDIT_ID))
                        .lock_focus(true)
                        .hint_text("Calculate something")
                        .frame(false)
                        .desired_width(input_width)
                        .font(FontSelection::from(FONT_ID))
                        .desired_rows(rows)
                        .layouter(&mut input_layouter(lines))
                        .show(ui);
                    if let Some(range) = output.cursor_range {
                        self.input_text_cursor_range = range;
                    }

                    if self.input_should_request_focus {
                        self.input_should_request_focus = false;
                        ui.ctx().memory().request_focus(output.response.id);
                    }

                    self.update_lines(output.galley);

                    if let Some(range) = output.cursor_range {
                        self.handle_shortcuts(ui, range);
                    }

                    vertical_spacer(ui);

                    ui.vertical(|ui| {
                        ui.add_space(2.0);
                        ui.spacing_mut().item_spacing.y = 0.0;

                        // Spacer to put scroll wheel at the right side of the window
                        ui.allocate_exact_size(
                            vec2(ui.available_width(), 0.0), Sense::hover());

                        for (i, line) in self.lines.iter_mut().enumerate() {
                            if let Line::Line {
                                output_text: text,
                                function,
                                is_error,
                                show_in_plot,
                                ..
                            } = line {
                                if !*is_error {
                                    if let Some(Function(_, arg_count, _)) = function {
                                        if *arg_count == 1 {
                                            ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                                                ui.checkbox(show_in_plot, "Plot");
                                                ui.add_space(-2.0);
                                            });
                                            continue;
                                        }
                                    }
                                }

                                output_text(ui, text, FONT_ID, i + 1);
                            } else {
                                ui.add_space(FONT_SIZE);
                            }
                        }
                    });
                });
            });
        });

        self.line_picker_dialog(ctx);
        self.first_frame = false;
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        eframe::set_value(storage, &app_key(), self);
    }
}

fn input_layouter(lines: &[Line]) -> impl FnMut(&Ui, &str, f32) -> Arc<Galley> + '_ {
    move |ui, string, wrap_width| {
        let mut job = text::LayoutJob {
            text: string.into(),
            ..Default::default()
        };

        if !lines.is_empty() {
            let mut end = 0usize;
            let mut offset = 0usize;
            let mut i = 0usize;

            for line in string.lines() {
                if i >= lines.len() { break; }

                let trimmed_line = line.trim();
                // Skip empty lines
                if !trimmed_line.is_empty() && !trimmed_line.starts_with('#') {
                    // NOTE: We use `Line::Empty`s to add spacing if the line spans multiple rows.
                    //  We have to skip these lines here to get to the actual color segments.
                    while matches!(lines.get(i), Some(Line::Empty)) { i += 1; }

                    if let Some(Line::Line { color_segments, .. }) = &lines.get(i) {
                        if !layout_segments(FONT_ID, color_segments, &mut job, string, &mut end, offset) {
                            break;
                        }
                    }
                }

                offset += line.len() + 1;
                i += 1;
            }

            if end != string.len() {
                job.sections.push(section(end..string.len(), FONT_ID, Color32::GRAY));
            }
        } else {
            job.sections.push(section(0..string.len(), FONT_ID, Color32::GRAY));
        }

        job.wrap.max_width = wrap_width;
        ui.fonts().layout_job(job)
    }
}
