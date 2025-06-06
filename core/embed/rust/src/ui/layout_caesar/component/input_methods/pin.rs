use crate::{
    strutil::{ShortString, TString},
    time::Duration,
    translations::TR,
    trezorhal::random,
    ui::{
        component::{
            text::common::TextBox, Child, Component, ComponentExt, Event, EventCtx, Timer,
        },
        display::Icon,
        geometry::Rect,
        shape::Renderer,
    },
};

use super::super::{
    super::fonts, theme, title::Title, ButtonDetails, ButtonLayout, CancelConfirmMsg,
    ChangingTextLine, ChoiceControls, ChoiceFactory, ChoiceItem, ChoiceMsg, ChoicePage,
};

#[derive(Clone, Copy)]
enum PinAction {
    Delete,
    Show,
    Enter,
    Digit(char),
}

struct PinChoice {
    text: TString<'static>,
    action: PinAction,
    icon: Option<Icon>,
    without_release: bool,
}

impl PinChoice {
    pub const fn new(
        text: TString<'static>,
        action: PinAction,
        icon: Option<Icon>,
        without_release: bool,
    ) -> Self {
        Self {
            text,
            action,
            icon,
            without_release,
        }
    }
}

const MAX_PIN_LENGTH: usize = 50;
const EMPTY_PIN_STR: &str = "_";

const CHOICE_LENGTH: usize = 13;
const NUMBER_START_INDEX: usize = 3;

const LAST_DIGIT_TIMEOUT_S: u32 = 1;

const CHOICES: [PinChoice; CHOICE_LENGTH] = [
    // DELETE should be triggerable without release (after long-press)
    PinChoice::new(
        TR::inputs__delete.as_tstring(),
        PinAction::Delete,
        Some(theme::ICON_DELETE),
        true, // without_release
    ),
    PinChoice::new(
        TR::inputs__show.as_tstring(),
        PinAction::Show,
        Some(theme::ICON_EYE),
        false,
    ),
    PinChoice::new(
        TR::inputs__enter.as_tstring(),
        PinAction::Enter,
        Some(theme::ICON_TICK),
        false,
    ),
    PinChoice::new(TString::from_str("0"), PinAction::Digit('0'), None, false),
    PinChoice::new(TString::from_str("1"), PinAction::Digit('1'), None, false),
    PinChoice::new(TString::from_str("2"), PinAction::Digit('2'), None, false),
    PinChoice::new(TString::from_str("3"), PinAction::Digit('3'), None, false),
    PinChoice::new(TString::from_str("4"), PinAction::Digit('4'), None, false),
    PinChoice::new(TString::from_str("5"), PinAction::Digit('5'), None, false),
    PinChoice::new(TString::from_str("6"), PinAction::Digit('6'), None, false),
    PinChoice::new(TString::from_str("7"), PinAction::Digit('7'), None, false),
    PinChoice::new(TString::from_str("8"), PinAction::Digit('8'), None, false),
    PinChoice::new(TString::from_str("9"), PinAction::Digit('9'), None, false),
];

fn get_random_digit_position() -> usize {
    random::uniform_between(NUMBER_START_INDEX as u32, (CHOICE_LENGTH - 1) as u32) as usize
}

struct ChoiceFactoryPIN;

impl ChoiceFactory for ChoiceFactoryPIN {
    type Action = PinAction;
    type Item = ChoiceItem;

    fn get(&self, choice_index: usize) -> (Self::Item, Self::Action) {
        let choice = &CHOICES[choice_index];

        let mut choice_item = choice.text.map(|t| {
            ChoiceItem::new(
                t,
                ButtonLayout::arrow_armed_arrow(TR::buttons__select.into()),
            )
        });

        // Action buttons have different middle button text
        if !matches!(choice.action, PinAction::Digit(_)) {
            let confirm_btn = ButtonDetails::armed_text(TR::buttons__confirm.into());
            choice_item.set_middle_btn(Some(confirm_btn));
        }

        // Making middle button create LongPress events
        if choice.without_release {
            choice_item = choice_item.with_middle_action_without_release();
        }

        // Adding icons for appropriate items
        if let Some(icon) = choice.icon {
            choice_item = choice_item.with_icon(icon);
        }

        (choice_item, choice.action)
    }

    fn count(&self) -> usize {
        CHOICE_LENGTH
    }
}

/// Component for entering a PIN.
pub struct PinEntry<'a> {
    choice_page: ChoicePage<ChoiceFactoryPIN, PinAction>,
    header_line: Child<Title>,
    pin_line: Child<ChangingTextLine>,
    prompt: TString<'static>,
    subprompt: TString<'a>,
    /// Whether we already show the "real" prompt (not the warning).
    showing_real_prompt: bool,
    show_real_pin: bool,
    show_last_digit: bool,
    textbox: TextBox,
    timeout_timer: Timer,
}

impl<'a> PinEntry<'a> {
    pub fn new(prompt: TString<'static>, subprompt: TString<'a>) -> Self {
        // When subprompt is not empty, it means that the user has entered bad PIN
        // before. In this case we show the warning together with the subprompt
        // at the beginning. (WRONG PIN will be replaced by real prompt after
        // any button click.)
        let show_subprompt = !subprompt.is_empty();
        let (showing_real_prompt, header_line_content, pin_line_content) = if show_subprompt {
            (false, TR::pin__title_wrong_pin.into(), subprompt)
        } else {
            (true, prompt, EMPTY_PIN_STR.into())
        };

        let mut pin_line = pin_line_content
            .map(|s| ChangingTextLine::center_bold(s, MAX_PIN_LENGTH).without_ellipsis());
        if show_subprompt {
            pin_line.update_font(fonts::FONT_NORMAL);
        }

        Self {
            // Starting at a random digit.
            choice_page: ChoicePage::new(ChoiceFactoryPIN)
                .with_initial_page_counter(get_random_digit_position())
                .with_controls(ChoiceControls::Carousel),
            header_line: Child::new(Title::new(header_line_content).with_centered()),
            pin_line: Child::new(pin_line),
            subprompt,
            prompt,
            showing_real_prompt,
            show_real_pin: false,
            show_last_digit: false,
            textbox: TextBox::empty(MAX_PIN_LENGTH),
            timeout_timer: Timer::new(),
        }
    }

    /// Performs overall update of the screen.
    fn update(&mut self, ctx: &mut EventCtx) {
        self.update_pin_line(ctx);
        ctx.request_paint();
    }

    /// Show updated content in the changing line.
    /// Many possibilities, according to the PIN state.
    fn update_pin_line(&mut self, ctx: &mut EventCtx) {
        debug_assert!({
            let s = ShortString::new();
            s.capacity() >= MAX_PIN_LENGTH
        });
        let mut used_font = fonts::FONT_BOLD;
        let pin_line_text = if self.is_empty() && !self.subprompt.is_empty() {
            // Showing the subprompt in NORMAL font
            used_font = fonts::FONT_NORMAL;
            self.subprompt.map(|s| unwrap!(ShortString::try_from(s)))
        } else if self.is_empty() {
            unwrap!(ShortString::try_from(EMPTY_PIN_STR))
        } else if self.show_real_pin {
            unwrap!(ShortString::try_from(self.pin()))
        } else {
            // Showing asterisks and possibly the last digit.
            let mut dots = ShortString::new();
            for _ in 0..self.textbox.len() - 1 {
                unwrap!(dots.push('*'));
            }
            let last_char = if self.show_last_digit {
                unwrap!(self.textbox.content().chars().last())
            } else {
                '*'
            };
            unwrap!(dots.push(last_char));
            dots
        };

        self.pin_line.mutate(ctx, |ctx, pin_line| {
            pin_line.update_font(used_font);
            pin_line.update_text(&pin_line_text);
            pin_line.request_complete_repaint(ctx);
        });
    }

    /// Showing the real prompt instead of WRONG PIN
    fn show_prompt(&mut self, ctx: &mut EventCtx) {
        self.header_line.mutate(ctx, |ctx, header_line| {
            header_line.set_text(ctx, self.prompt);
            header_line.request_complete_repaint(ctx);
        });
    }

    pub fn pin(&self) -> &str {
        self.textbox.content()
    }

    fn is_full(&self) -> bool {
        self.textbox.len() >= MAX_PIN_LENGTH
    }

    fn is_empty(&self) -> bool {
        self.textbox.is_empty()
    }
}

impl Component for PinEntry<'_> {
    type Msg = CancelConfirmMsg;

    fn place(&mut self, bounds: Rect) -> Rect {
        // same adjustment of height as in ChangingTextLine::needed_height()
        let header_height = Title::height() + 2;
        let (header_area, rest) = bounds.split_top(header_height);
        let pin_height = self.pin_line.inner().needed_height();
        let (pin_area, choice_area) = rest.split_top(pin_height);
        self.header_line.place(header_area);
        self.pin_line.place(pin_area);
        self.choice_page.place(choice_area);
        bounds
    }

    fn event(&mut self, ctx: &mut EventCtx, event: Event) -> Option<Self::Msg> {
        self.header_line.mutate(ctx, |ctx, title| {
            title.event(ctx, event);
        });
        match event {
            // Timeout for showing the last digit.
            Event::Timer(_) if self.timeout_timer.expire(event) => {
                if self.show_last_digit {
                    self.show_last_digit = false;
                    self.update(ctx)
                }
            }
            // Other timers are ignored.
            Event::Timer(_) => {}
            // Any non-timer event when showing real PIN should hide it
            _ if self.show_real_pin => {
                self.show_real_pin = false;
                self.update(ctx);
            }
            _ => {}
        }

        // Any button event will show the "real" prompt
        if !self.showing_real_prompt {
            if let Event::Button(_) = event {
                self.show_prompt(ctx);
                self.showing_real_prompt = true;
            }
        }

        match self.choice_page.event(ctx, event) {
            Some(ChoiceMsg::Choice {
                item: PinAction::Delete,
                long_press,
            }) => {
                // Deleting all when long-pressed
                if long_press {
                    self.textbox.clear(ctx);
                } else {
                    self.textbox.delete_last(ctx);
                }
                self.update(ctx);
            }
            Some(ChoiceMsg::Choice {
                item: PinAction::Show,
                ..
            }) => {
                self.show_real_pin = true;
                self.update(ctx);
            }
            Some(ChoiceMsg::Choice {
                item: PinAction::Enter,
                ..
            }) if !self.is_empty() => {
                // ENTER is not valid when the PIN is empty
                return Some(CancelConfirmMsg::Confirmed);
            }
            Some(ChoiceMsg::Choice {
                item: PinAction::Digit(ch),
                ..
            }) if !self.is_full() => {
                self.textbox.append(ctx, ch);
                // Choosing random digit to be shown next
                self.choice_page
                    .set_page_counter(ctx, get_random_digit_position(), true);
                self.show_last_digit = true;
                self.timeout_timer
                    .start(ctx, Duration::from_secs(LAST_DIGIT_TIMEOUT_S));
                self.update(ctx);
            }
            _ => {}
        };

        None
    }

    fn render<'s>(&'s self, target: &mut impl Renderer<'s>) {
        self.header_line.render(target);
        self.pin_line.render(target);
        self.choice_page.render(target);
    }
}

// DEBUG-ONLY SECTION BELOW

#[cfg(feature = "ui_debug")]
impl crate::trace::Trace for PinEntry<'_> {
    fn trace(&self, t: &mut dyn crate::trace::Tracer) {
        t.component("PinKeyboard");
        t.string("subprompt", self.subprompt);
        t.string("pin", self.textbox.content().into());
        t.child("choice_page", &self.choice_page);
    }
}
