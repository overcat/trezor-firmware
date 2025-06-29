use crate::{
    error,
    micropython::{buffer::StrBuffer, iter::IterBuf, obj::Obj, util},
    strutil::TString,
    translations::TR,
    ui::{
        button_request::ButtonRequest,
        component::{
            swipe_detect::SwipeSettings,
            text::paragraphs::{Paragraph, ParagraphSource, Paragraphs},
            ButtonRequestExt, ComponentExt, Qr,
        },
        flow::{
            base::{Decision, DecisionBuilder as _},
            FlowController, FlowMsg, SwipeFlow, SwipePage,
        },
        geometry::Direction,
        layout::util::ConfirmValueParams,
    },
};

use super::super::{
    component::{AddressDetails, Frame, PromptScreen, SwipeContent, VerticalMenu},
    theme,
};

const QR_BORDER: i16 = 4;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum GetAddress {
    Address,
    Tap,
    Menu,
    QrCode,
    AccountInfo,
    Cancel,
    CancelTap,
}

impl FlowController for GetAddress {
    #[inline]
    fn index(&'static self) -> usize {
        *self as usize
    }

    fn handle_swipe(&'static self, direction: Direction) -> Decision {
        match (self, direction) {
            (Self::Address, Direction::Up) => Self::Tap.swipe(direction),
            (Self::Tap, Direction::Down) => Self::Address.swipe(direction),
            (Self::Cancel, Direction::Up) => Self::CancelTap.swipe(direction),
            (Self::CancelTap, Direction::Down) => Self::Cancel.swipe(direction),
            _ => self.do_nothing(),
        }
    }

    fn handle_event(&'static self, msg: FlowMsg) -> Decision {
        match (self, msg) {
            (Self::Address, FlowMsg::Info) => Self::Menu.goto(),
            (Self::Tap, FlowMsg::Confirmed) => self.return_msg(FlowMsg::Confirmed),
            (Self::Tap, FlowMsg::Info) => Self::Menu.swipe_left(),
            (Self::Menu, FlowMsg::Choice(0)) => Self::QrCode.swipe_left(),
            (Self::Menu, FlowMsg::Choice(1)) => Self::AccountInfo.swipe_left(),
            (Self::Menu, FlowMsg::Choice(2)) => Self::Cancel.swipe_left(),
            (Self::Menu, FlowMsg::Cancelled) => Self::Address.swipe_right(),
            (Self::QrCode, FlowMsg::Cancelled) => Self::Menu.goto(),
            (Self::AccountInfo, FlowMsg::Cancelled) => Self::Menu.goto(),
            (Self::Cancel, FlowMsg::Cancelled) => Self::Menu.goto(),
            (Self::CancelTap, FlowMsg::Confirmed) => self.return_msg(FlowMsg::Cancelled),
            (Self::CancelTap, FlowMsg::Cancelled) => Self::Menu.goto(),
            _ => self.do_nothing(),
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn new_get_address(
    title: TString<'static>,
    description: Option<TString<'static>>,
    extra: Option<TString<'static>>,
    address: Obj, // TODO: get rid of Obj
    chunkify: bool,
    address_qr: TString<'static>,
    case_sensitive: bool,
    account: Option<TString<'static>>,
    path: Option<TString<'static>>,
    xpubs: Obj, // TODO: get rid of Obj
    br_code: u16,
    br_name: TString<'static>,
) -> Result<SwipeFlow, error::Error> {
    // Address
    let paragraphs = ConfirmValueParams {
        description: description.unwrap_or_else(|| "".into()),
        extra: extra.unwrap_or_else(|| "".into()),
        value: address.try_into()?,
        font: if chunkify {
            let address: TString = address.try_into()?;
            theme::get_chunkified_text_style(address.len())
        } else {
            &theme::TEXT_MONO_DATA
        },
        description_font: &theme::TEXT_NORMAL,
        extra_font: &theme::TEXT_DEMIBOLD,
    }
    .into_paragraphs();
    let content_address =
        Frame::left_aligned(title, SwipeContent::new(SwipePage::vertical(paragraphs)))
            .with_menu_button()
            .with_swipeup_footer(None)
            .with_vertical_pages()
            .map_to_button_msg()
            .one_button_request(ButtonRequest::from_num(br_code, br_name))
            // Count tap-to-confirm screen towards page count
            .with_pages(|address_pages| address_pages + 1);

    // Tap
    let content_tap =
        Frame::left_aligned(title, SwipeContent::new(PromptScreen::new_tap_to_confirm()))
            .with_footer(TR::instructions__tap_to_confirm.into(), None)
            .with_swipe(Direction::Down, SwipeSettings::default())
            .map(super::util::map_to_confirm);

    // Menu
    let content_menu = Frame::left_aligned(
        "".into(),
        VerticalMenu::empty()
            .item(theme::ICON_QR_CODE, TR::address__qr_code.into())
            .item(
                theme::ICON_CHEVRON_RIGHT,
                TR::address_details__account_info.into(),
            )
            .danger(theme::ICON_CANCEL, TR::address__cancel_receive.into()),
    )
    .with_cancel_button()
    .map(super::util::map_to_choice);

    // QrCode
    let content_qr = Frame::left_aligned(
        title,
        address_qr
            .map(|s| Qr::new(s, case_sensitive))?
            .with_border(QR_BORDER),
    )
    .with_cancel_button()
    .map_to_button_msg();

    // AccountInfo
    let mut ad = AddressDetails::new(TR::address_details__account_info.into(), account, path)?;
    for i in IterBuf::new().try_iterate(xpubs)? {
        let [xtitle, text]: [StrBuffer; 2] = util::iter_into_array(i)?;
        ad.add_xpub(xtitle, text)?;
    }
    let content_account = ad.map(|_| Some(FlowMsg::Cancelled));

    // Cancel
    let content_cancel_info = Frame::left_aligned(
        TR::address__cancel_receive.into(),
        SwipeContent::new(Paragraphs::new(Paragraph::new(
            &theme::TEXT_MAIN_GREY_LIGHT,
            TR::address__cancel_contact_support,
        ))),
    )
    .with_cancel_button()
    .with_swipeup_footer(None)
    .map_to_button_msg();

    // CancelTap
    let content_cancel_tap = Frame::left_aligned(
        TR::address__cancel_receive.into(),
        PromptScreen::new_tap_to_cancel(),
    )
    .with_cancel_button()
    .with_footer(TR::instructions__tap_to_confirm.into(), None)
    .with_swipe(Direction::Down, SwipeSettings::default())
    .map(super::util::map_to_confirm);

    let mut res = SwipeFlow::new(&GetAddress::Address)?;
    res.add_page(&GetAddress::Address, content_address)?
        .add_page(&GetAddress::Tap, content_tap)?
        .add_page(&GetAddress::Menu, content_menu)?
        .add_page(&GetAddress::QrCode, content_qr)?
        .add_page(&GetAddress::AccountInfo, content_account)?
        .add_page(&GetAddress::Cancel, content_cancel_info)?
        .add_page(&GetAddress::CancelTap, content_cancel_tap)?;
    Ok(res)
}
