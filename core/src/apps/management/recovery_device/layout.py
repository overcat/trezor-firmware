from typing import TYPE_CHECKING

from trezor import TR
from trezor.ui.layouts.recovery import (  # noqa: F401
    request_word_count,
    show_already_added,
    show_dry_run_result,
    show_group_share_success,
    show_group_thresholod,
    show_identifier_mismatch,
    show_recovery_warning,
)

from apps.common import backup_types

from .recover import RecoveryAborted

if TYPE_CHECKING:
    from typing import Awaitable

    from trezor.enums import BackupType

    # RemainingSharesInfo represents the data structure for remaining shares in SLIP-39 recovery:
    # - Set of tuples, each containing 2 or 3 words identifying a group
    # - List of remaining share counts for each group
    # - Group threshold (minimum number of groups required)
    RemainingSharesInfo = tuple[set[tuple[str, ...]], list[int], int]


async def request_mnemonic(
    word_count: int, backup_type: BackupType | None
) -> str | None:
    from trezor.ui.layouts.recovery import request_word

    from . import word_validity

    send_button_request = True

    # Pre-allocate the list to enable going back and overwriting words.
    words: list[str] = [""] * word_count
    i = 0

    def all_words_entered() -> bool:
        return i >= word_count

    while not all_words_entered():
        # Prefilling the previously inputted word in case of going back
        word = await request_word(
            i,
            word_count,
            is_slip39=backup_types.is_slip39_word_count(word_count),
            send_button_request=send_button_request,
            prefill_word=words[i],
        )
        send_button_request = False

        if not word:
            # User has decided to go back
            if i > 0:
                words[i] = ""
                i -= 1
            continue

        words[i] = word

        i += 1

        try:
            non_empty_words = [word for word in words if word]
            word_validity.check(backup_type, non_empty_words)
        except word_validity.AlreadyAdded:
            # show_share_already_added
            await show_already_added()
            return None
        except word_validity.IdentifierMismatch:
            # show_identifier_mismatch
            await show_identifier_mismatch()
            return None
        except word_validity.ThresholdReached:
            # show_group_threshold_reached
            await show_group_thresholod()
            return None

    return " ".join(words)


def enter_share(
    word_count: int | None = None,
    entered_remaining: tuple[int, int] | None = None,
    remaining_shares_info: RemainingSharesInfo | None = None,
) -> Awaitable[None]:
    from trezor import strings

    show_instructions = False

    if word_count is not None:
        # First-time entry. Show instructions and word count.
        text = TR.recovery__enter_any_share
        subtext = TR.recovery__word_count_template.format(word_count)
        show_instructions = True

    elif entered_remaining is not None:
        # Basic Shamir. There is only one group, we report entered/remaining count.
        entered, remaining = entered_remaining
        total = entered + remaining
        text = TR.recovery__x_of_y_entered_template.format(entered, total)
        subtext = strings.format_plural(
            TR.recovery__x_more_shares_needed_template_plural,
            remaining,
            TR.plurals__x_shares_needed,
        )

    else:
        # SuperShamir. We cannot easily show entered/remaining across groups,
        # the caller provided an info_func that has the details.
        text = TR.recovery__more_shares_needed
        subtext = None

    return homescreen_dialog(
        TR.buttons__enter_share,
        text,
        subtext,
        show_instructions,
        remaining_shares_info,
    )


async def homescreen_dialog(
    button_label: str,
    text: str,
    subtext: str | None = None,
    show_instructions: bool = False,
    remaining_shares_info: "RemainingSharesInfo | None" = None,
) -> None:
    import storage.recovery as storage_recovery
    from trezor.ui.layouts.recovery import continue_recovery

    recovery_type = storage_recovery.get_type()
    if not await continue_recovery(
        button_label,
        text,
        subtext,
        recovery_type,
        show_instructions,
        remaining_shares_info,
    ):
        raise RecoveryAborted
