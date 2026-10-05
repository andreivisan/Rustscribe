use super::*;
use gpui::{
    Context, Div, ElementId, ExternalPaths, FontWeight, ObjectFit, SharedString, Stateful, Window,
    WindowControlArea, div, img, prelude::*, px, relative, rgb, rgba, svg,
};

const BG: u32 = 0x303237;
const SHELF: u32 = 0x202124;
const PANEL: u32 = 0x1a1b1e;
const FIELD: u32 = 0x292b30;
const LINE: u32 = 0x393b41;
const TEXT: u32 = 0xf4f5f7;
const MUTED: u32 = 0x9da1ac;
const BLUE: u32 = 0x1683ff;
const GREEN: u32 = 0x78d9b0;

fn icon(name: &'static str, size: f32, color: u32) -> gpui::Svg {
    svg()
        .path(name)
        .size(px(size))
        .flex_shrink_0()
        .text_color(rgb(color))
}
fn row() -> Div {
    div().flex().items_center()
}
fn column() -> Div {
    div().flex().flex_col()
}
fn label(text: impl Into<SharedString>) -> Div {
    div()
        .text_size(px(11.))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(rgb(MUTED))
        .child(text.into())
}
fn button(
    id: impl Into<ElementId>,
    title: impl Into<SharedString>,
    symbol: &'static str,
    primary: bool,
    enabled: bool,
) -> Stateful<Div> {
    row()
        .id(id)
        .h(px(38.))
        .px_4()
        .gap_2()
        .justify_center()
        .rounded(px(11.))
        .bg(rgb(if primary { BLUE } else { FIELD }))
        .text_color(rgb(TEXT))
        .text_size(px(12.))
        .font_weight(FontWeight::MEDIUM)
        .when(!enabled, |this| this.opacity(0.35))
        .when(enabled, |this| {
            this.cursor_pointer()
                .hover(|this| this.bg(rgb(if primary { 0x3495ff } else { 0x363940 })))
        })
        .child(icon(symbol, 16., TEXT))
        .child(title.into())
}
fn icon_button(id: impl Into<ElementId>, symbol: &'static str) -> Stateful<Div> {
    row()
        .id(id)
        .size(px(32.))
        .justify_center()
        .rounded(px(9.))
        .cursor_pointer()
        .hover(|this| this.bg(rgb(LINE)))
        .child(icon(symbol, 17., MUTED))
}
fn duration(seconds: f64) -> String {
    let seconds = seconds.round() as u64;
    if seconds >= 3600 {
        format!(
            "{}:{:02}:{:02}",
            seconds / 3600,
            seconds / 60 % 60,
            seconds % 60
        )
    } else {
        format!("{}:{:02}", seconds / 60, seconds % 60)
    }
}
fn bytes(bytes: u64) -> String {
    if bytes >= 1_000_000_000 {
        format!("{:.1} GB", bytes as f64 / 1_000_000_000.)
    } else {
        format!("{:.1} MB", bytes as f64 / 1_000_000.)
    }
}

impl Render for Desktop {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        column()
            .id("rustscribe")
            .track_focus(&self.focus)
            .size_full()
            .relative()
            .bg(rgb(BG))
            .font_family(".SystemUIFont")
            .text_size(px(13.))
            .text_color(rgb(TEXT))
            .on_action(cx.listener(|this, _: &AddFiles, _, cx| this.pick_files(cx)))
            .on_action(cx.listener(|this, _: &OpenSettings, _, cx| {
                this.settings_open = true;
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &StartTranscription, _, cx| {
                if !this.settings_open {
                    this.transcribe(false, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &ExportAll, _, cx| {
                if !this.settings_open {
                    this.export(false, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &Dismiss, _, cx| {
                this.settings_open = false;
                this.reading = false;
                this.notice = None;
                cx.notify();
            }))
            .on_drop(cx.listener(|this, paths: &ExternalPaths, _, cx| {
                this.add_files(paths.paths().to_vec(), cx)
            }))
            .drag_over::<ExternalPaths>(|this, _, _, _| this.bg(rgb(0x303c50)))
            .child(self.titlebar(cx))
            .child(
                column()
                    .flex_1()
                    .min_h_0()
                    .px(px(28.))
                    .pt(px(12.))
                    .pb(px(22.))
                    .gap(px(22.))
                    .child(
                        row()
                            .justify_between()
                            .items_end()
                            .child(
                                column()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_size(px(25.))
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .child("Less watching. More knowing."),
                                    )
                                    .child(
                                        div()
                                            .text_color(rgb(MUTED))
                                            .text_size(px(13.))
                                            .child("Turn your videos into words worth keeping."),
                                    ),
                            )
                            .child(
                                row()
                                    .gap_2()
                                    .pb_1()
                                    .text_color(rgb(MUTED))
                                    .text_size(px(11.))
                                    .child(icon("shield", 15., GREEN))
                                    .child("On your Mac. Always."),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_1()
                            .min_h_0()
                            .gap(px(20.))
                            .child(self.shelf(cx))
                            .child(self.inspector(cx)),
                    ),
            )
            .child(
                row()
                    .h(px(32.))
                    .flex_shrink_0()
                    .px(px(30.))
                    .justify_between()
                    .border_t_1()
                    .border_color(rgb(0x3b3d42))
                    .text_size(px(10.))
                    .text_color(rgb(MUTED))
                    .child(
                        row()
                            .gap_2()
                            .child(div().size(px(5.)).rounded_full().bg(rgb(GREEN)))
                            .child("LOCAL TRANSCRIPTION"),
                    )
                    .child("⌘ O  Add files     ·     ⌘ ↵  Transcribe"),
            )
            .when(self.reading, |this| this.child(self.reader(cx)))
            .when(self.settings_open, |this| {
                this.child(self.settings_modal(cx))
            })
    }
}

impl Desktop {
    fn titlebar(&self, cx: &mut Context<Self>) -> Div {
        row()
            .h(px(62.))
            .flex_shrink_0()
            .pl(px(104.))
            .pr(px(27.))
            .justify_between()
            .window_control_area(WindowControlArea::Drag)
            .child(
                row().gap_2().child(icon("wave", 21., TEXT)).child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_size(px(14.))
                        .child("Rustscribe"),
                ),
            )
            .child(
                icon_button("settings", "settings").on_click(cx.listener(|this, _, _, cx| {
                    this.settings_open = true;
                    cx.notify();
                })),
            )
    }

    fn shelf(&self, cx: &mut Context<Self>) -> Div {
        let done = self
            .files
            .iter()
            .filter(|file| file.transcript.is_some())
            .count();
        let ready = self.files.len() - done;
        let visible: Vec<_> = self
            .files
            .iter()
            .filter(|file| match self.filter {
                Filter::All => true,
                Filter::Ready => file.transcript.is_none(),
                Filter::Done => file.transcript.is_some(),
            })
            .collect();
        column()
            .flex_1()
            .min_w_0()
            .min_h_0()
            .child(
                row()
                    .h(px(27.))
                    .w(px(174.))
                    .px_4()
                    .rounded_t(px(12.))
                    .bg(rgb(SHELF))
                    .gap_2()
                    .child(icon("folder", 13., MUTED))
                    .child(label("YOUR WORKSPACE")),
            )
            .child(
                column()
                    .flex_1()
                    .min_h_0()
                    .rounded_b(px(22.))
                    .rounded_tr(px(22.))
                    .bg(rgb(SHELF))
                    .shadow_lg()
                    .overflow_hidden()
                    .child(
                        row()
                            .px(px(24.))
                            .pt(px(24.))
                            .pb(px(18.))
                            .justify_between()
                            .child(
                                column()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_size(px(24.))
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .child("Media shelf"),
                                    )
                                    .child(div().text_color(rgb(MUTED)).text_size(px(12.)).child(
                                        if self.files.is_empty() {
                                            "A little space for your next big idea.".into()
                                        } else {
                                            format!(
                                                "{} file{} · {} transcribed",
                                                self.files.len(),
                                                if self.files.len() == 1 { "" } else { "s" },
                                                done
                                            )
                                        },
                                    )),
                            )
                            .child(
                                icon_button("add-top", "plus")
                                    .on_click(cx.listener(|this, _, _, cx| this.pick_files(cx))),
                            ),
                    )
                    .child(
                        row()
                            .mx(px(24.))
                            .pb(px(16.))
                            .gap_2()
                            .border_b_1()
                            .border_color(rgb(0x303237))
                            .children(
                                [
                                    (Filter::All, "All files", self.files.len()),
                                    (Filter::Ready, "Ready", ready),
                                    (Filter::Done, "Transcribed", done),
                                ]
                                .into_iter()
                                .enumerate()
                                .map(
                                    |(index, (filter, title, count))| {
                                        row()
                                            .id(("filter", index))
                                            .h(px(29.))
                                            .px_3()
                                            .gap_2()
                                            .rounded(px(8.))
                                            .text_size(px(11.))
                                            .cursor_pointer()
                                            .bg(rgb(if self.filter == filter {
                                                FIELD
                                            } else {
                                                SHELF
                                            }))
                                            .text_color(rgb(if self.filter == filter {
                                                TEXT
                                            } else {
                                                MUTED
                                            }))
                                            .hover(|this| this.bg(rgb(FIELD)))
                                            .child(title)
                                            .child(
                                                div()
                                                    .text_size(px(10.))
                                                    .text_color(rgb(MUTED))
                                                    .child(count.to_string()),
                                            )
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.filter = filter;
                                                cx.notify();
                                            }))
                                    },
                                ),
                            ),
                    )
                    .child(if self.files.is_empty() {
                        self.empty_shelf(cx).into_any_element()
                    } else if visible.is_empty() {
                        column()
                            .flex_1()
                            .items_center()
                            .justify_center()
                            .gap_3()
                            .child(icon("check", 28., MUTED))
                            .child(div().text_color(rgb(MUTED)).child("Nothing here just yet."))
                            .into_any_element()
                    } else {
                        div()
                            .id("media-scroll")
                            .flex_1()
                            .min_h_0()
                            .overflow_y_scroll()
                            .p(px(20.))
                            .child(div().grid().grid_cols(3).gap(px(14.)).children(
                                visible.into_iter().map(|file| self.media_card(file, cx)),
                            ))
                            .into_any_element()
                    })
                    .when(self.busy, |this| this.child(self.progress(cx)))
                    .when_some(self.notice.as_ref(), |this, (message, error)| {
                        this.child(
                            row()
                                .mx(px(20.))
                                .mb_3()
                                .p_3()
                                .gap_2()
                                .rounded(px(10.))
                                .bg(rgb(if *error { 0x39282c } else { 0x25332f }))
                                .child(icon(
                                    if *error { "document" } else { "check" },
                                    15.,
                                    if *error { 0xf1a5a5 } else { GREEN },
                                ))
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w_0()
                                        .text_size(px(11.))
                                        .line_clamp(3)
                                        .text_color(rgb(if *error { 0xf1b8b8 } else { 0xbee3d2 }))
                                        .child(message.clone()),
                                )
                                .child(icon_button("dismiss-notice", "close").on_click(
                                    cx.listener(|this, _, _, cx| {
                                        this.notice = None;
                                        cx.notify();
                                    }),
                                )),
                        )
                    })
                    .child(
                        row()
                            .h(px(74.))
                            .px(px(22.))
                            .flex_shrink_0()
                            .border_t_1()
                            .border_color(rgb(0x303237))
                            .justify_between()
                            .child(
                                row()
                                    .gap_2()
                                    .text_size(px(11.))
                                    .text_color(rgb(MUTED))
                                    .child(icon("document", 17., MUTED))
                                    .child(if done > 0 {
                                        format!("{done} ready to export")
                                    } else {
                                        "Video in. Markdown out.".into()
                                    }),
                            )
                            .child(
                                row()
                                    .gap_2()
                                    .child(
                                        button(
                                            "export-all",
                                            "Export all",
                                            "export",
                                            false,
                                            done > 0 && !self.busy,
                                        )
                                        .on_click(
                                            cx.listener(|this, _, _, cx| {
                                                if !this.busy {
                                                    this.export(false, cx);
                                                }
                                            }),
                                        ),
                                    )
                                    .child(
                                        button(
                                            "transcribe-all",
                                            if self.busy {
                                                "Working…"
                                            } else {
                                                "Transcribe"
                                            },
                                            "arrow",
                                            true,
                                            ready > 0 && !self.busy,
                                        )
                                        .on_click(
                                            cx.listener(|this, _, _, cx| {
                                                this.transcribe(false, cx)
                                            }),
                                        ),
                                    ),
                            ),
                    ),
            )
    }

    fn empty_shelf(&self, cx: &mut Context<Self>) -> Div {
        column()
            .flex_1()
            .min_h_0()
            .items_center()
            .justify_center()
            .gap_3()
            .py_6()
            .child(
                div()
                    .relative()
                    .w(px(168.))
                    .h(px(126.))
                    .mb_3()
                    .child(
                        div()
                            .absolute()
                            .left(px(18.))
                            .top(px(6.))
                            .w(px(132.))
                            .h(px(91.))
                            .rounded(px(15.))
                            .bg(rgb(0x363941))
                            .border_1()
                            .border_color(rgb(0x4a4e58)),
                    )
                    .child(
                        div()
                            .absolute()
                            .left(px(8.))
                            .top(px(19.))
                            .w(px(152.))
                            .h(px(92.))
                            .rounded(px(16.))
                            .bg(rgb(0x2d3e59))
                            .border_1()
                            .border_color(rgb(0x435d84)),
                    )
                    .child(
                        row()
                            .absolute()
                            .left_0()
                            .top(px(32.))
                            .w(px(168.))
                            .h(px(94.))
                            .justify_center()
                            .rounded(px(17.))
                            .bg(rgb(0x223754))
                            .border_1()
                            .border_color(rgb(0x3669a5))
                            .shadow_lg()
                            .child(icon("wave", 45., 0x78b4ff)),
                    ),
            )
            .child(
                div()
                    .text_size(px(18.))
                    .font_weight(FontWeight::MEDIUM)
                    .child("Drop something worth keeping"),
            )
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(rgb(MUTED))
                    .child("Videos, podcasts, lectures. It all starts here."),
            )
            .child(
                button("browse-files", "Choose files", "plus", true, true)
                    .mt_2()
                    .on_click(cx.listener(|this, _, _, cx| this.pick_files(cx))),
            )
            .child(
                div()
                    .mt_1()
                    .text_size(px(10.))
                    .text_color(rgb(0x737986))
                    .child("MP4, WEBM, MOV, MP3, WAV + more"),
            )
    }

    fn media_card(&self, file: &Media, cx: &mut Context<Self>) -> Stateful<Div> {
        let id = file.id;
        let selected = self.selected == Some(id);
        let color = match file.status {
            Status::Done => GREEN,
            Status::Failed => 0xf2a4a4,
            Status::Loading | Status::Transcribing => BLUE,
            _ => MUTED,
        };
        column()
            .id(("media", id))
            .min_w_0()
            .gap_2()
            .p(px(6.))
            .rounded(px(13.))
            .cursor_pointer()
            .bg(rgb(if selected { 0x30343d } else { SHELF }))
            .hover(|this| this.bg(rgb(0x2c2f35)))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.selected = Some(id);
                cx.notify();
            }))
            .child(
                div()
                    .relative()
                    .w_full()
                    .h(px(108.))
                    .overflow_hidden()
                    .rounded(px(9.))
                    .border_1()
                    .border_color(rgb(if selected { 0x6b8cb7 } else { 0x3d424b }))
                    .bg(rgb(0x292f3d))
                    .child(match file.thumbnail.as_ref() {
                        Some(path) => img(path.clone())
                            .size_full()
                            .object_fit(ObjectFit::Cover)
                            .into_any_element(),
                        None => row()
                            .size_full()
                            .justify_center()
                            .child(icon("wave", 36., 0x8297b5))
                            .into_any_element(),
                    })
                    .child(
                        div()
                            .absolute()
                            .top_2()
                            .left_2()
                            .px(px(5.))
                            .py(px(2.))
                            .rounded(px(4.))
                            .bg(rgba(0x141820c9))
                            .text_size(px(9.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(file.extension()),
                    )
                    .when_some(file.duration, |this, seconds| {
                        this.child(
                            div()
                                .absolute()
                                .bottom_2()
                                .right_2()
                                .px(px(5.))
                                .py(px(2.))
                                .rounded(px(4.))
                                .bg(rgba(0x101318cc))
                                .text_size(px(10.))
                                .child(duration(seconds)),
                        )
                    })
                    .when(file.transcript.is_some(), |this| {
                        this.child(
                            row()
                                .absolute()
                                .top_2()
                                .right_2()
                                .size(px(20.))
                                .rounded_full()
                                .justify_center()
                                .bg(rgb(0x235140))
                                .child(icon("check", 13., GREEN)),
                        )
                    }),
            )
            .child(
                div()
                    .px_1()
                    .text_size(px(11.))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(rgb(if selected { 0x88bbff } else { TEXT }))
                    .truncate()
                    .child(file.name()),
            )
            .child(
                row()
                    .px_1()
                    .pb_1()
                    .gap(px(5.))
                    .text_size(px(10.))
                    .text_color(rgb(color))
                    .child(div().size(px(4.)).rounded_full().bg(rgb(color)))
                    .child(file.status.label()),
            )
    }

    fn progress(&self, cx: &mut Context<Self>) -> Div {
        let elapsed = self
            .started
            .map(|start| start.elapsed().as_secs())
            .unwrap_or(0);
        column()
            .mx(px(22.))
            .mb_3()
            .gap_2()
            .child(
                row()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(rgb(0x9fcaff))
                            .child(format!(
                                "{} of {} complete · {}",
                                self.completed_count,
                                self.batch_count,
                                duration(elapsed as f64)
                            )),
                    )
                    .child(
                        div()
                            .id("stop-queue")
                            .cursor_pointer()
                            .text_size(px(10.))
                            .text_color(rgb(MUTED))
                            .hover(|this| this.text_color(rgb(TEXT)))
                            .child(if self.cancel.load(Ordering::Relaxed) {
                                "Stopping after this file…"
                            } else {
                                "Stop after this file"
                            })
                            .on_click(cx.listener(|this, _, _, cx| this.stop(cx))),
                    ),
            )
            .child(
                div().h(px(3.)).w_full().rounded_full().bg(rgb(LINE)).child(
                    div()
                        .h_full()
                        .w(relative(
                            self.completed_count as f32 / self.batch_count.max(1) as f32,
                        ))
                        .rounded_full()
                        .bg(rgb(BLUE)),
                ),
            )
    }

    fn inspector(&self, cx: &mut Context<Self>) -> Stateful<Div> {
        let model = self.settings.model;
        let configured = self
            .settings
            .path(model)
            .is_some_and(|path| model.valid_path(path));
        column()
            .id("inspector-scroll")
            .w(px(300.))
            .flex_shrink_0()
            .min_h_0()
            .overflow_y_scroll()
            .pt(px(27.))
            .gap_3()
            .child(
                column()
                    .p(px(16.))
                    .gap_3()
                    .flex_shrink_0()
                    .rounded(px(20.))
                    .bg(rgb(PANEL))
                    .shadow_md()
                    .child(
                        row()
                            .justify_between()
                            .child(
                                div()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_size(px(14.))
                                    .child("Transcription"),
                            )
                            .child(
                                row()
                                    .gap_1()
                                    .text_size(px(10.))
                                    .text_color(rgb(GREEN))
                                    .child(div().size(px(4.)).rounded_full().bg(rgb(GREEN)))
                                    .child("LOCAL"),
                            ),
                    )
                    .child(
                        row()
                            .p_1()
                            .gap_1()
                            .rounded(px(11.))
                            .bg(rgb(0x101113))
                            .children([Model::Whisper, Model::Cohere].into_iter().enumerate().map(
                                |(index, option)| {
                                    row()
                                        .id(("engine", index))
                                        .flex_1()
                                        .h(px(32.))
                                        .justify_center()
                                        .rounded(px(8.))
                                        .text_size(px(12.))
                                        .font_weight(FontWeight::MEDIUM)
                                        .bg(rgb(if model == option { 0x3a3d44 } else { 0x101113 }))
                                        .text_color(rgb(if model == option { TEXT } else { MUTED }))
                                        .when(!self.busy, |this| {
                                            this.cursor_pointer().hover(|this| this.bg(rgb(FIELD)))
                                        })
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            if !this.busy {
                                                this.settings.model = option;
                                                this.save_settings(cx);
                                            }
                                        }))
                                        .child(option.name())
                                },
                            )),
                    )
                    .child(
                        column()
                            .gap_2()
                            .child(
                                row().justify_between().child(label("MODEL")).child(
                                    div()
                                        .text_size(px(10.))
                                        .text_color(rgb(if configured { GREEN } else { 0xeab877 }))
                                        .child(if configured {
                                            "Ready on this Mac"
                                        } else {
                                            "Setup needed"
                                        }),
                                ),
                            )
                            .child(
                                row()
                                    .id("model-settings")
                                    .gap_2()
                                    .cursor_pointer()
                                    .text_size(px(11.))
                                    .text_color(rgb(MUTED))
                                    .child(icon("spark", 14., MUTED))
                                    .child(div().flex_1().child(match model {
                                        Model::Whisper => "Whisper · Metal acceleration",
                                        Model::Cohere => "Cohere · CPU inference",
                                    }))
                                    .child(icon("chevron", 12., MUTED))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.settings_open = true;
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(
                        column()
                            .gap_2()
                            .pt_3()
                            .border_t_1()
                            .border_color(rgb(0x303237))
                            .child(
                                row().justify_between().child(label("EXPORT TO")).child(
                                    div()
                                        .text_size(px(10.))
                                        .text_color(rgb(MUTED))
                                        .child("Markdown .md"),
                                ),
                            )
                            .child(
                                row()
                                    .id("output-folder")
                                    .gap_2()
                                    .cursor_pointer()
                                    .text_color(rgb(TEXT))
                                    .text_size(px(11.))
                                    .child(icon("folder", 16., MUTED))
                                    .child(
                                        div().flex_1().truncate().child(
                                            self.settings
                                                .output_directory
                                                .as_ref()
                                                .map(|path| {
                                                    path.file_name()
                                                        .unwrap_or_default()
                                                        .to_string_lossy()
                                                        .into_owned()
                                                })
                                                .unwrap_or_else(|| "Same folder as source".into()),
                                        ),
                                    )
                                    .child(icon("chevron", 12., MUTED))
                                    .on_click(cx.listener(|this, _, _, cx| this.choose_output(cx))),
                            ),
                    )
                    .child(
                        row()
                            .gap_2()
                            .text_size(px(10.))
                            .text_color(rgb(MUTED))
                            .child(icon("wave", 13., MUTED))
                            .child("English · Adaptive audio chunks"),
                    ),
            )
            .child(self.preview(cx))
    }

    fn preview(&self, cx: &mut Context<Self>) -> Div {
        let panel = column()
            .flex_1()
            .min_h(px(260.))
            .p(px(16.))
            .gap_2()
            .rounded(px(20.))
            .bg(rgb(PANEL))
            .shadow_md()
            .overflow_hidden();
        let Some(file) = self.selected_file() else {
            return panel.child(label("FROM VIDEO TO NOTES"))
                .child(div().mt_1().text_size(px(19.)).font_weight(FontWeight::MEDIUM).child("Keep the good parts."))
                .child(div().text_size(px(12.)).line_height(relative(1.5)).text_color(rgb(MUTED)).child("A clean transcript from the things you watch and listen to. Ready for your notes."))
                .child(column().mt_2().gap_3().children([("01", "Drop in a video or audio file"), ("02", "Pick your local AI model"), ("03", "Export a readable Markdown file")].into_iter().map(|(number, text)| {
                    row().gap_3().text_size(px(11.)).child(div().text_color(rgb(0x6da8ed)).child(number)).child(div().text_color(rgb(MUTED)).child(text))
                })));
        };
        let transcript = file.transcript.as_ref();
        let can_transcribe = !self.busy && !file.exporting;
        panel
            .child(
                row()
                    .gap_2()
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .font_weight(FontWeight::MEDIUM)
                            .text_size(px(12.))
                            .truncate()
                            .child(file.name()),
                    )
                    .child(
                        icon_button("remove-file", "remove")
                            .size(px(24.))
                            .when(!can_transcribe, |this| this.opacity(0.3))
                            .on_click(cx.listener(|this, _, _, cx| this.remove_selected(cx))),
                    ),
            )
            .child(
                div()
                    .text_size(px(10.))
                    .text_color(rgb(MUTED))
                    .child(format!(
                        "{} · {}{}",
                        file.extension(),
                        bytes(file.bytes),
                        file.duration
                            .map(|seconds| format!(" · {}", duration(seconds)))
                            .unwrap_or_default()
                    )),
            )
            .child(
                row()
                    .justify_between()
                    .pt_2()
                    .border_t_1()
                    .border_color(rgb(0x303237))
                    .child(label(if transcript.is_some() {
                        "TRANSCRIPT"
                    } else {
                        "PREVIEW"
                    }))
                    .when(transcript.is_some(), |this| {
                        this.child(
                            row()
                                .gap_1()
                                .child(
                                    div()
                                        .id("read-transcript")
                                        .text_size(px(10.))
                                        .text_color(rgb(0x80b9ff))
                                        .cursor_pointer()
                                        .child("Read ↗")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.reading = true;
                                            cx.notify();
                                        })),
                                )
                                .child(icon_button("copy", "copy").size(px(24.)).on_click(
                                    cx.listener(|this, _, _, cx| this.copy_transcript(cx)),
                                )),
                        )
                    }),
            )
            .child(
                div()
                    .id(("transcript-scroll", file.id))
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .pr_1()
                    .child(match transcript {
                        Some(transcript) => div()
                            .text_size(px(12.))
                            .line_height(relative(1.65))
                            .text_color(rgb(0xc8cbd1))
                            .child(transcript.text.clone())
                            .into_any_element(),
                        None => column()
                            .gap_2()
                            .py_3()
                            .child(icon("wave", 25., 0x626e81))
                            .child(div().text_size(px(11.)).text_color(rgb(MUTED)).child(
                                file.error.clone().unwrap_or_else(|| match file.status {
                                    Status::Queued => "Waiting in the queue.".into(),
                                    Status::Loading => "Loading your local model…".into(),
                                    Status::Transcribing => {
                                        "Listening, one chunk at a time…".into()
                                    }
                                    _ => "The words will appear here. Ready when you are.".into(),
                                }),
                            ))
                            .into_any_element(),
                    }),
            )
            .when_some(transcript, |this, transcript| {
                this.child(
                    row()
                        .justify_between()
                        .child(
                            div()
                                .text_size(px(10.))
                                .text_color(rgb(MUTED))
                                .child(format!(
                                    "{} words · {} · {:.1}s",
                                    transcript.text.split_whitespace().count(),
                                    transcript.engine,
                                    transcript.transcription_duration.as_secs_f64()
                                )),
                        )
                        .child(
                            div()
                                .id("retry")
                                .text_size(px(10.))
                                .text_color(rgb(0x80b9ff))
                                .when(can_transcribe, |this| this.cursor_pointer())
                                .when(!can_transcribe, |this| this.opacity(0.3))
                                .child("Retry")
                                .on_click(cx.listener(|this, _, _, cx| this.transcribe(true, cx))),
                        ),
                )
            })
            .child(if transcript.is_some() {
                button(
                    "export-selected",
                    if file.exporting {
                        "Saving…"
                    } else {
                        "Export Markdown"
                    },
                    "export",
                    true,
                    !file.exporting && !self.busy,
                )
                .on_click(cx.listener(|this, _, _, cx| {
                    if !this.busy {
                        this.export(true, cx);
                    }
                }))
                .into_any_element()
            } else {
                button(
                    "transcribe-selected",
                    "Transcribe file",
                    "wave",
                    true,
                    can_transcribe,
                )
                .on_click(cx.listener(|this, _, _, cx| this.transcribe(true, cx)))
                .into_any_element()
            })
            .when(transcript.is_some(), |this| {
                this.child(
                    row()
                        .justify_between()
                        .text_size(px(10.))
                        .text_color(rgb(MUTED))
                        .child(
                            div()
                                .id("save-as")
                                .cursor_pointer()
                                .hover(|this| this.text_color(rgb(TEXT)))
                                .child("Save as…")
                                .on_click(cx.listener(|this, _, _, cx| this.save_as(cx))),
                        )
                        .when_some(file.saved.clone(), |this, path| {
                            this.child(
                                div()
                                    .id("reveal-export")
                                    .cursor_pointer()
                                    .text_color(rgb(0x80b9ff))
                                    .child("Show in Finder ↗")
                                    .on_click(move |_, _, cx| cx.reveal_path(&path)),
                            )
                        }),
                )
            })
    }

    fn reader(&self, cx: &mut Context<Self>) -> Div {
        let Some(file) = self.selected_file() else {
            return div();
        };
        let Some(transcript) = file.transcript.as_ref() else {
            return div();
        };
        let mut paragraphs = Vec::new();
        let mut paragraph = String::new();
        for word in transcript.text.split_whitespace() {
            if !paragraph.is_empty() {
                paragraph.push(' ');
            }
            paragraph.push_str(word);
            if paragraph.len() >= 650
                && word
                    .trim_end_matches(['"', '\'', '”', '’', ')', ']', '}'])
                    .ends_with(['.', '?', '!'])
            {
                paragraphs.push(std::mem::take(&mut paragraph));
            }
        }
        if !paragraph.trim().is_empty() {
            paragraphs.push(paragraph);
        }
        row()
            .absolute()
            .inset_0()
            .occlude()
            .justify_center()
            .bg(rgba(0x090b10cc))
            .child(
                column()
                    .w(px(740.))
                    .h(relative(0.84))
                    .p(px(28.))
                    .gap_4()
                    .rounded(px(22.))
                    .border_1()
                    .border_color(rgb(LINE))
                    .bg(rgb(SHELF))
                    .shadow_2xl()
                    .child(
                        row()
                            .justify_between()
                            .child(
                                column().gap_1().child(label("TRANSCRIPT")).child(
                                    div()
                                        .text_size(px(20.))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .child(file.name()),
                                ),
                            )
                            .child(icon_button("close-reader", "close").on_click(cx.listener(
                                |this, _, _, cx| {
                                    this.reading = false;
                                    cx.notify();
                                },
                            ))),
                    )
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(rgb(MUTED))
                            .child(format!(
                                "{} · {} words · English",
                                transcript.engine,
                                transcript.text.split_whitespace().count()
                            )),
                    )
                    .child(
                        div()
                            .id(("reader-scroll", file.id))
                            .flex_1()
                            .min_h_0()
                            .overflow_y_scroll()
                            .pr_4()
                            .child(
                                column()
                                    .gap_4()
                                    .text_size(px(15.))
                                    .line_height(relative(1.7))
                                    .text_color(rgb(0xd5d8de))
                                    .children(
                                        paragraphs.into_iter().map(|paragraph| {
                                            div().child(paragraph.trim().to_owned())
                                        }),
                                    ),
                            ),
                    )
                    .child(
                        row()
                            .justify_between()
                            .child(div().text_size(px(11.)).text_color(rgb(MUTED)).child(
                                if file.exporting {
                                    "Saving Markdown…".to_owned()
                                } else if file.saved.is_some() {
                                    "Markdown saved. Your notes are ready.".to_owned()
                                } else {
                                    "Captured locally. Ready for your notes.".to_owned()
                                },
                            ))
                            .child(
                                row()
                                    .gap_2()
                                    .child(
                                        button("reader-copy", "Copy", "copy", false, true)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.copy_transcript(cx)
                                            })),
                                    )
                                    .child(
                                        button(
                                            "reader-export",
                                            "Export Markdown",
                                            "export",
                                            true,
                                            !self.busy && !file.exporting,
                                        )
                                        .on_click(
                                            cx.listener(|this, _, _, cx| {
                                                if !this.busy {
                                                    this.export(true, cx);
                                                }
                                            }),
                                        ),
                                    ),
                            ),
                    ),
            )
    }

    fn settings_modal(&self, cx: &mut Context<Self>) -> Div {
        row()
            .absolute()
            .inset_0()
            .occlude()
            .justify_center()
            .bg(rgba(0x090b10bb))
            .child(
                column()
                    .w(px(500.))
                    .p(px(28.))
                    .gap_5()
                    .rounded(px(22.))
                    .border_1()
                    .border_color(rgb(LINE))
                    .bg(rgb(SHELF))
                    .shadow_2xl()
                    .child(
                        row()
                            .justify_between()
                            .child(
                                column()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_size(px(22.))
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .child("Make yourself at home"),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(12.))
                                            .text_color(rgb(MUTED))
                                            .child("Your models. Your Mac. Your preferences."),
                                    ),
                            )
                            .child(icon_button("close-settings", "close").on_click(cx.listener(
                                |this, _, _, cx| {
                                    this.settings_open = false;
                                    cx.notify();
                                },
                            ))),
                    )
                    .children([Model::Whisper, Model::Cohere].into_iter().enumerate().map(
                        |(index, model)| {
                            let path = self.settings.path(model);
                            column()
                                .p_4()
                                .gap_3()
                                .rounded(px(13.))
                                .bg(rgb(FIELD))
                                .child(
                                    row()
                                        .justify_between()
                                        .child(
                                            div()
                                                .font_weight(FontWeight::MEDIUM)
                                                .child(model.name()),
                                        )
                                        .child(
                                            div()
                                                .text_size(px(10.))
                                                .text_color(rgb(
                                                    if path
                                                        .is_some_and(|path| model.valid_path(path))
                                                    {
                                                        GREEN
                                                    } else {
                                                        MUTED
                                                    },
                                                ))
                                                .child(
                                                    if path
                                                        .is_some_and(|path| model.valid_path(path))
                                                    {
                                                        "Available"
                                                    } else {
                                                        "Not configured"
                                                    },
                                                ),
                                        ),
                                )
                                .child(div().text_size(px(11.)).text_color(rgb(MUTED)).child(
                                    match model {
                                        Model::Whisper => {
                                            "Select a whisper.cpp .bin file. Uses your Apple GPU."
                                        }
                                        Model::Cohere => {
                                            "Select the Cohere int8 ONNX folder. Runs on the CPU."
                                        }
                                    },
                                ))
                                .child(
                                    div()
                                        .text_size(px(10.))
                                        .line_clamp(2)
                                        .text_color(rgb(0xb0b9c8))
                                        .child(
                                            path.map(|path| path.display().to_string())
                                                .unwrap_or_else(|| "No model selected".into()),
                                        ),
                                )
                                .child(
                                    button(
                                        ("choose-model", index),
                                        if model == Model::Whisper {
                                            "Choose model file…"
                                        } else {
                                            "Choose model folder…"
                                        },
                                        "folder",
                                        false,
                                        !self.busy,
                                    )
                                    .on_click(cx.listener(
                                        move |this, _, _, cx| this.choose_model(model, cx),
                                    )),
                                )
                        },
                    ))
                    .child(
                        row()
                            .justify_between()
                            .child(
                                column().gap_1().child(label("MARKDOWN EXPORTS")).child(
                                    div().text_size(px(11.)).text_color(rgb(MUTED)).child(
                                        "Existing files are kept. New exports get a new name.",
                                    ),
                                ),
                            )
                            .child(
                                div()
                                    .id("reset-output")
                                    .cursor_pointer()
                                    .text_size(px(11.))
                                    .text_color(rgb(0x80b9ff))
                                    .child("Reset folder")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.settings.output_directory = None;
                                        this.save_settings(cx);
                                    })),
                            ),
                    )
                    .when_some(
                        self.notice.as_ref().filter(|(_, error)| *error),
                        |this, (message, _)| {
                            this.child(
                                div()
                                    .text_size(px(11.))
                                    .text_color(rgb(0xf1a5a5))
                                    .line_clamp(3)
                                    .child(message.clone()),
                            )
                        },
                    )
                    .child(
                        button("done-settings", "All set", "check", true, true).on_click(
                            cx.listener(|this, _, _, cx| {
                                this.settings_open = false;
                                cx.notify();
                            }),
                        ),
                    ),
            )
    }
}
