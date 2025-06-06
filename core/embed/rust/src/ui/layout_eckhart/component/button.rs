#[cfg(feature = "haptic")]
use crate::trezorhal::haptic::{play, HapticEffect};
use crate::{
    strutil::TString,
    time::Duration,
    ui::{
        component::{text::TextStyle, Component, Event, EventCtx, Timer},
        display::{toif::Icon, Color, Font},
        event::TouchEvent,
        geometry::{Alignment, Alignment2D, Insets, Offset, Point, Rect},
        lerp::Lerp,
        shape::{self, Renderer},
        util::split_two_lines,
    },
};

#[cfg(feature = "bootloader")]
use super::super::fonts;

use super::super::theme;

pub enum ButtonMsg {
    Pressed,
    Released,
    Clicked,
    LongPressed,
}

pub struct Button {
    area: Rect,
    touch_expand: Option<Insets>,
    content: ButtonContent,
    content_offset: Offset,
    stylesheet: ButtonStyleSheet,
    text_align: Alignment,
    radius: Option<u8>,
    state: State,
    long_press: Option<Duration>,
    long_timer: Timer,
    haptic: bool,
    gradient: bool,
}

impl Button {
    #[cfg(not(feature = "bootloader"))]
    const DEFAULT_SUBTEXT_STYLE: TextStyle = theme::label_menu_item_subtitle();
    #[cfg(feature = "bootloader")]
    const DEFAULT_SUBTEXT_STYLE: TextStyle = theme::TEXT_NORMAL;
    #[cfg(not(feature = "bootloader"))]
    pub const SUBTEXT_STYLE_GREEN: TextStyle = theme::label_menu_item_subtitle_green();
    #[cfg(feature = "bootloader")]
    pub const SUBTEXT_STYLE_GREEN: TextStyle = TextStyle::new(
        fonts::FONT_SATOSHI_REGULAR_38,
        theme::GREEN,
        theme::BG,
        theme::GREEN,
        theme::GREEN,
    );
    const MENU_ITEM_RADIUS: u8 = 12;
    const MENU_ITEM_ALIGNMENT: Alignment = Alignment::Start;
    const MENU_ITEM_CONTENT_OFFSET: Offset = Offset::x(12);

    pub const fn new(content: ButtonContent) -> Self {
        Self {
            content,
            content_offset: Offset::zero(),
            area: Rect::zero(),
            touch_expand: None,
            stylesheet: theme::button_default(),
            text_align: Alignment::Center,
            radius: None,
            state: State::Initial,
            long_press: None,
            long_timer: Timer::new(),
            haptic: true,
            gradient: false,
        }
    }

    pub fn new_menu_item(text: TString<'static>, stylesheet: ButtonStyleSheet) -> Self {
        Self::with_text(text)
            .with_text_align(Self::MENU_ITEM_ALIGNMENT)
            .with_content_offset(Self::MENU_ITEM_CONTENT_OFFSET)
            .styled(stylesheet)
            .with_radius(Self::MENU_ITEM_RADIUS)
    }

    pub fn new_menu_item_with_subtext(
        text: TString<'static>,
        stylesheet: ButtonStyleSheet,
        subtext: TString<'static>,
        subtext_style: Option<TextStyle>,
    ) -> Self {
        Self::with_text_and_subtext(text, subtext, subtext_style)
            .with_text_align(Self::MENU_ITEM_ALIGNMENT)
            .with_content_offset(Self::MENU_ITEM_CONTENT_OFFSET)
            .styled(stylesheet)
            .with_radius(Self::MENU_ITEM_RADIUS)
    }

    pub const fn with_text(text: TString<'static>) -> Self {
        Self::new(ButtonContent::Text(text))
    }

    pub fn with_text_and_subtext(
        text: TString<'static>,
        subtext: TString<'static>,
        subtext_style: Option<TextStyle>,
    ) -> Self {
        Self::new(ButtonContent::TextAndSubtext {
            text,
            subtext,
            subtext_style: subtext_style.unwrap_or(Self::DEFAULT_SUBTEXT_STYLE),
        })
    }

    pub const fn with_icon(icon: Icon) -> Self {
        Self::new(ButtonContent::Icon(icon))
    }

    pub const fn with_icon_and_text(content: IconText) -> Self {
        Self::new(ButtonContent::IconAndText(content))
    }

    #[cfg(feature = "micropython")]
    pub const fn with_homebar_content(text: Option<TString<'static>>) -> Self {
        Self::new(ButtonContent::HomeBar(text))
    }

    pub const fn empty() -> Self {
        Self::new(ButtonContent::Empty)
    }

    pub const fn styled(mut self, stylesheet: ButtonStyleSheet) -> Self {
        self.stylesheet = stylesheet;
        self
    }

    pub const fn with_text_align(mut self, align: Alignment) -> Self {
        self.text_align = align;
        self
    }

    pub const fn with_content_offset(mut self, offset: Offset) -> Self {
        self.content_offset = offset;
        self
    }

    pub const fn with_expanded_touch_area(mut self, expand: Insets) -> Self {
        self.touch_expand = Some(expand);
        self
    }

    pub fn with_long_press(mut self, duration: Duration) -> Self {
        self.long_press = Some(duration);
        self
    }

    pub fn with_radius(mut self, radius: u8) -> Self {
        // Both radius and gradient not supported
        debug_assert!(!self.gradient);
        self.radius = Some(radius);
        self
    }

    pub fn without_haptics(mut self) -> Self {
        self.haptic = false;
        self
    }

    pub fn with_gradient(mut self) -> Self {
        // Using gradient with radius is not supported
        debug_assert!(self.radius.is_none());
        self.gradient = true;
        self
    }

    pub fn enable_if(&mut self, ctx: &mut EventCtx, enabled: bool) {
        if enabled {
            self.enable(ctx);
        } else {
            self.disable(ctx);
        }
    }

    pub fn initially_enabled(mut self, enabled: bool) -> Self {
        if !enabled {
            self.state = State::Disabled;
        }
        self
    }

    pub fn enable(&mut self, ctx: &mut EventCtx) {
        self.set(ctx, State::Initial)
    }

    pub fn disable(&mut self, ctx: &mut EventCtx) {
        self.set(ctx, State::Disabled)
    }

    pub fn is_enabled(&self) -> bool {
        matches!(
            self.state,
            State::Initial | State::Pressed | State::Released
        )
    }

    pub fn is_pressed(&self) -> bool {
        matches!(self.state, State::Pressed)
    }

    pub fn long_press(&self) -> Option<Duration> {
        self.long_press
    }

    pub fn is_disabled(&self) -> bool {
        matches!(self.state, State::Disabled)
    }

    pub fn set_content(&mut self, content: ButtonContent) {
        if self.content != content {
            self.content = content
        }
    }

    pub fn set_expanded_touch_area(&mut self, expand: Insets) {
        self.touch_expand = Some(expand);
    }

    pub fn set_content_offset(&mut self, offset: Offset) {
        self.content_offset = offset;
    }

    pub fn content(&self) -> &ButtonContent {
        &self.content
    }

    pub fn content_offset(&self) -> Offset {
        self.content_offset
    }

    fn baseline_text_height(&self) -> i16 {
        // Use static string for the content height calculation to avoid misalignment
        // among keyboard buttons.
        self.style().font.visible_text_height("1")
    }

    pub fn content_height(&self) -> i16 {
        match &self.content {
            ButtonContent::Empty => 0,
            ButtonContent::Text(_) => self.baseline_text_height(),
            ButtonContent::Icon(icon) => icon.toif.height(),
            ButtonContent::IconAndText(child) => {
                let text_height = self.baseline_text_height();
                let icon_height = child.icon.toif.height();
                text_height.max(icon_height)
            }
            ButtonContent::TextAndSubtext { subtext_style, .. } => {
                self.style().font.line_height() + subtext_style.text_font.text_height()
            }
            #[cfg(feature = "micropython")]
            ButtonContent::HomeBar(_) => theme::ACTION_BAR_HEIGHT,
        }
    }

    pub fn set_stylesheet(&mut self, stylesheet: ButtonStyleSheet) {
        if self.stylesheet != stylesheet {
            self.stylesheet = stylesheet;
        }
    }

    pub fn style(&self) -> &ButtonStyle {
        match self.state {
            State::Initial | State::Released => self.stylesheet.normal,
            State::Pressed => self.stylesheet.active,
            State::Disabled => self.stylesheet.disabled,
        }
    }

    pub fn stylesheet(&self) -> &ButtonStyleSheet {
        &self.stylesheet
    }

    pub fn area(&self) -> Rect {
        self.area
    }

    pub fn touch_area(&self) -> Rect {
        self.touch_expand
            .map_or(self.area, |expand| self.area.outset(expand))
    }

    fn set(&mut self, ctx: &mut EventCtx, state: State) {
        if self.state != state {
            self.state = state;
            ctx.request_paint();
        }
    }

    fn render_gradient_bar<'s>(&self, target: &mut impl Renderer<'s>, style: &ButtonStyle) {
        let height = self.area.height();
        let half_width = (self.area.width() / 2) as f32;
        let x_mid = self.area.center().x;

        // Layer 1: Horizontal Gradient (Overall intensity: 100%)
        // Stops:    21%, 100%
        // Opacity: 100%,  20%
        for y in self.area.y0..self.area.y1 {
            let factor = (y - self.area.y0) as f32 / height as f32;
            let slice = Rect::new(Point::new(self.area.x0, y), Point::new(self.area.x1, y + 1));
            let factor_grad = ((factor - 0.21) / (1.00 - 0.21)).clamp(0.0, 1.0);
            let alpha = u8::lerp(u8::MAX, 51, factor_grad);
            shape::Bar::new(slice)
                .with_bg(style.button_color)
                .with_alpha(alpha)
                .render(target);
        }

        // Layer 2: Vertical Gradient (Overall intensity: 100%)
        // distance from mid
        for x in self.area.x0..self.area.x1 {
            let slice = Rect::new(Point::new(x, self.area.y0), Point::new(x + 1, self.area.y1));
            let dist_from_mid = (x - x_mid).abs() as f32 / half_width;
            let alpha = u8::lerp(u8::MIN, u8::MAX, dist_from_mid);
            shape::Bar::new(slice)
                .with_bg(theme::BG)
                .with_alpha(alpha)
                .render(target);
        }

        // Layer 3: Black overlay (Overall intensity: 20%)
        shape::Bar::new(self.area)
            .with_bg(theme::BG)
            .with_alpha(51)
            .render(target);
    }

    pub fn render_background<'s>(
        &self,
        target: &mut impl Renderer<'s>,
        style: &ButtonStyle,
        alpha: u8,
    ) {
        match (self.radius, self.gradient) {
            (Some(radius), _) => {
                shape::Bar::new(self.area)
                    .with_bg(style.background_color)
                    .with_radius(radius as i16)
                    .with_thickness(2)
                    .with_fg(style.button_color)
                    .with_alpha(alpha)
                    .render(target);
            }
            // Gradient bar is rendered only in `normal` state, not `active` or `disabled`
            (None, true) if self.state.is_normal() => {
                self.render_gradient_bar(target, style);
            }
            _ => {
                shape::Bar::new(self.area)
                    .with_bg(style.button_color)
                    .with_fg(style.button_color)
                    .with_alpha(alpha)
                    .render(target);
            }
        }
    }

    fn render_content<'s>(
        &self,
        target: &mut impl Renderer<'s>,
        stylesheet: &ButtonStyle,
        alpha: u8,
    ) {
        match &self.content {
            ButtonContent::Empty => {}
            ButtonContent::Text(text) => {
                let render_origin = match self.text_align {
                    Alignment::Start => self.area.left_center().ofs(self.content_offset),
                    Alignment::Center => self.area.center().ofs(self.content_offset),
                    Alignment::End => self.area.right_center().ofs(self.content_offset.neg()),
                }
                .ofs(Offset::y(self.content_height() / 2));
                text.map(|text| {
                    shape::Text::new(render_origin, text, stylesheet.font)
                        .with_fg(stylesheet.text_color)
                        .with_align(self.text_align)
                        .with_alpha(alpha)
                        .render(target);
                });
            }
            ButtonContent::TextAndSubtext {
                text,
                subtext,
                subtext_style,
            } => {
                let base = match self.text_align {
                    Alignment::Start => self.area.left_center().ofs(self.content_offset),
                    Alignment::Center => self.area.center().ofs(self.content_offset),
                    Alignment::End => self.area.right_center().ofs(self.content_offset.neg()),
                };

                let text_render_origin = base
                    .ofs(Offset::y(self.content_height() / 2 - self.baseline_text_height()).neg());
                let subtext_render_origin = base.ofs(Offset::y(self.content_height() / 2));

                text.map(|t| {
                    shape::Text::new(text_render_origin, t, stylesheet.font)
                        .with_fg(stylesheet.text_color)
                        .with_align(self.text_align)
                        .with_alpha(alpha)
                        .render(target);
                });

                subtext.map(|subtext| {
                    shape::Text::new(subtext_render_origin, subtext, subtext_style.text_font)
                        .with_fg(subtext_style.text_color)
                        .with_align(self.text_align)
                        .with_alpha(alpha)
                        .render(target);
                });
            }
            ButtonContent::Icon(icon) => {
                shape::ToifImage::new(self.area.center() + self.content_offset, icon.toif)
                    .with_align(Alignment2D::CENTER)
                    .with_fg(stylesheet.icon_color)
                    .with_alpha(alpha)
                    .render(target);
            }
            ButtonContent::IconAndText(child) => {
                child.render(target, self.area, self.style(), self.content_offset, alpha);
            }
            #[cfg(feature = "micropython")]
            ButtonContent::HomeBar(text) => {
                let baseline = self.area.center();
                if let Some(text) = text {
                    const OFFSET_Y: Offset = Offset::y(25);
                    text.map(|text| {
                        shape::Text::new(baseline, text, stylesheet.font)
                            .with_fg(stylesheet.text_color)
                            .with_align(Alignment::Center)
                            .with_alpha(alpha)
                            .render(target);
                    });
                    shape::ToifImage::new(
                        self.area.center() + OFFSET_Y,
                        theme::ICON_DASH_HORIZONTAL.toif,
                    )
                    .with_fg(stylesheet.icon_color)
                    .with_align(Alignment2D::CENTER)
                    .render(target);
                } else {
                    // double dash icon in the middle
                    const OFFSET_Y: Offset = Offset::y(5);
                    shape::ToifImage::new(baseline - OFFSET_Y, theme::ICON_DASH_HORIZONTAL.toif)
                        .with_fg(theme::GREY_LIGHT)
                        .with_align(Alignment2D::CENTER)
                        .render(target);

                    shape::ToifImage::new(baseline + OFFSET_Y, theme::ICON_DASH_HORIZONTAL.toif)
                        .with_fg(theme::GREY_LIGHT)
                        .with_align(Alignment2D::CENTER)
                        .render(target);
                }
            }
        }
    }

    pub fn render_with_alpha<'s>(&self, target: &mut impl Renderer<'s>, alpha: u8) {
        let style = self.style();
        self.render_background(target, style, alpha);
        self.render_content(target, style, alpha);
    }
}

impl Component for Button {
    type Msg = ButtonMsg;

    fn place(&mut self, bounds: Rect) -> Rect {
        self.area = bounds;
        self.area
    }

    fn event(&mut self, ctx: &mut EventCtx, event: Event) -> Option<Self::Msg> {
        let touch_area = self.touch_area();
        match event {
            Event::Touch(TouchEvent::TouchStart(pos)) => {
                match self.state {
                    State::Disabled => {
                        // Do nothing.
                    }
                    _ => {
                        // Touch started in our area, transform to `Pressed` state.
                        if touch_area.contains(pos) {
                            #[cfg(feature = "haptic")]
                            if self.haptic {
                                play(HapticEffect::ButtonPress);
                            }
                            self.set(ctx, State::Pressed);
                            if let Some(duration) = self.long_press {
                                self.long_timer.start(ctx, duration);
                            }
                            return Some(ButtonMsg::Pressed);
                        }
                    }
                }
            }
            Event::Touch(TouchEvent::TouchMove(pos)) => {
                match self.state {
                    State::Pressed if !touch_area.contains(pos) => {
                        // Touch is leaving our area, transform to `Released` state.
                        self.set(ctx, State::Released);
                        return Some(ButtonMsg::Released);
                    }
                    _ => {
                        // Do nothing.
                    }
                }
            }
            Event::Touch(TouchEvent::TouchEnd(pos)) => {
                match self.state {
                    State::Initial | State::Disabled => {
                        // Do nothing.
                    }
                    State::Pressed if touch_area.contains(pos) => {
                        // Touch finished in our area, we got clicked.
                        self.set(ctx, State::Initial);
                        return Some(ButtonMsg::Clicked);
                    }
                    State::Pressed => {
                        // Touch finished outside our area.
                        self.set(ctx, State::Initial);
                        self.long_timer.stop();
                        return Some(ButtonMsg::Released);
                    }
                    _ => {
                        // Touch finished outside our area.
                        self.set(ctx, State::Initial);
                        self.long_timer.stop();
                    }
                }
            }
            Event::Swipe(_) => {
                // When a swipe is detected, abort any ongoing touch.
                match self.state {
                    State::Initial | State::Disabled => {
                        // Do nothing.
                    }
                    State::Pressed => {
                        // Touch aborted
                        self.set(ctx, State::Initial);
                        self.long_timer.stop();
                        return Some(ButtonMsg::Released);
                    }
                    _ => {
                        // Irrelevant touch abort
                        self.set(ctx, State::Initial);
                        self.long_timer.stop();
                    }
                }
            }

            Event::Timer(_) if self.long_timer.expire(event) => {
                if matches!(self.state, State::Pressed) {
                    #[cfg(feature = "haptic")]
                    if self.haptic {
                        play(HapticEffect::ButtonPress);
                    }
                    self.set(ctx, State::Initial);
                    return Some(ButtonMsg::LongPressed);
                }
            }
            _ => {}
        };
        None
    }

    fn render<'s>(&'s self, target: &mut impl Renderer<'s>) {
        let style = self.style();
        self.render_background(target, style, 0xFF);
        self.render_content(target, style, 0xFF);
    }
}

#[cfg(feature = "ui_debug")]
impl crate::trace::Trace for Button {
    fn trace(&self, t: &mut dyn crate::trace::Tracer) {
        t.component("Button");
        match &self.content {
            ButtonContent::Empty => {}
            ButtonContent::Text(text) => t.string("text", *text),
            ButtonContent::Icon(_) => t.bool("icon", true),
            ButtonContent::IconAndText(content) => {
                t.string("text", content.text);
                t.bool("icon", true);
            }
            ButtonContent::TextAndSubtext { text, .. } => {
                t.string("text", *text);
            }
            #[cfg(feature = "micropython")]
            ButtonContent::HomeBar(text) => t.string("text", text.unwrap_or(TString::empty())),
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
enum State {
    Initial,
    Pressed,
    Released,
    Disabled,
}

impl State {
    /// Returns true if the button is in a normal state (not pressed or
    /// disabled).
    fn is_normal(&self) -> bool {
        matches!(self, State::Initial | State::Released)
    }
}

#[derive(PartialEq, Eq, Clone)]
pub enum ButtonContent {
    Empty,
    Text(TString<'static>),
    TextAndSubtext {
        text: TString<'static>,
        subtext: TString<'static>,
        subtext_style: TextStyle,
    },
    Icon(Icon),
    IconAndText(IconText),
    #[cfg(feature = "micropython")]
    HomeBar(Option<TString<'static>>),
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct ButtonStyleSheet {
    pub normal: &'static ButtonStyle,
    pub active: &'static ButtonStyle,
    pub disabled: &'static ButtonStyle,
}

#[derive(PartialEq, Eq, Clone)]
pub struct ButtonStyle {
    pub font: Font,
    pub text_color: Color,
    pub button_color: Color,
    pub icon_color: Color,
    pub background_color: Color,
}

#[derive(PartialEq, Eq, Clone)]
pub struct IconText {
    text: TString<'static>,
    icon: Icon,
}

impl IconText {
    const ICON_SPACE: i16 = 46;
    const ICON_MARGIN: i16 = 4;
    const TEXT_MARGIN: i16 = 6;

    pub fn new(text: impl Into<TString<'static>>, icon: Icon) -> Self {
        Self {
            text: text.into(),
            icon,
        }
    }

    pub fn render<'s>(
        &self,
        target: &mut impl Renderer<'s>,
        area: Rect,
        style: &ButtonStyle,
        baseline_offset: Offset,
        alpha: u8,
    ) {
        let mut show_text = |text: &str, rect: Rect| {
            let text_pos = rect.left_center() + baseline_offset;
            let text_pos = Point::new(rect.top_left().x + Self::ICON_SPACE, text_pos.y);
            shape::Text::new(text_pos, text, style.font)
                .with_fg(style.text_color)
                .with_alpha(alpha)
                .render(target)
        };

        self.text.map(|t| {
            let (t1, t2) = split_two_lines(
                t,
                style.font,
                area.width() - Self::ICON_SPACE - Self::TEXT_MARGIN,
            );

            if t1.is_empty() || t2.is_empty() {
                show_text(t, area);
            } else {
                show_text(t1, Rect::new(area.top_left(), area.right_center()));
                show_text(t2, Rect::new(area.left_center(), area.bottom_right()));
            }
        });

        let icon_pos = Point::new(
            area.top_left().x + ((Self::ICON_SPACE + Self::ICON_MARGIN) / 2),
            area.center().y,
        );
        shape::ToifImage::new(icon_pos, self.icon.toif)
            .with_align(Alignment2D::CENTER)
            .with_fg(style.icon_color)
            .with_alpha(alpha)
            .render(target);
    }
}
