from micropython import const

import storage.device as storage_device
from trezor.wire import DataError

_MAX_PASSPHRASE_LEN = const(50)


def is_enabled() -> bool:
    return storage_device.is_passphrase_enabled()


async def get() -> str:
    from trezor import workflow

    if not is_enabled():
        return ""
    else:
        workflow.close_others()  # request exclusive UI access
        if storage_device.get_passphrase_always_on_device():
            from trezor.ui.layouts import request_passphrase_on_device

            passphrase = await request_passphrase_on_device(_MAX_PASSPHRASE_LEN)
        else:
            passphrase = await _request_on_host()
        if len(passphrase.encode()) > _MAX_PASSPHRASE_LEN:
            raise DataError(f"Maximum passphrase length is {_MAX_PASSPHRASE_LEN} bytes")

        return passphrase


async def _request_on_host() -> str:
    from trezor import loop, workflow
    from trezor.messages import PassphraseAck, PassphraseRequest
    from trezor.ui.layouts import (
        confirm_hidden_passphrase_from_host,
        request_passphrase_on_host,
        show_passphrase_from_host,
    )
    from trezor.wire.context import call

    async def _delay_request_passphrase_on_host() -> None:
        await loop.sleep(100)
        return request_passphrase_on_host()

    on_host = workflow.spawn(_delay_request_passphrase_on_host())
    try:
        request = PassphraseRequest()
        ack = await call(request, PassphraseAck)
        passphrase = ack.passphrase  # local_cache_attribute
    finally:
        # make sure on-host passphrase prompt closed after receiving an ack
        on_host.close()

    if ack.on_device:
        from trezor.ui.layouts import request_passphrase_on_device

        if passphrase is not None:
            raise DataError("Passphrase provided when it should not be")
        return await request_passphrase_on_device(_MAX_PASSPHRASE_LEN)

    if passphrase is None:
        raise DataError(
            "Passphrase not provided and on_device is False. Use empty string to set an empty passphrase."
        )

    # non-empty passphrase
    if passphrase:
        # We want to hide the passphrase, or show it, according to settings.
        if storage_device.get_hide_passphrase_from_host():
            await confirm_hidden_passphrase_from_host()
        else:
            await show_passphrase_from_host(passphrase)

    return passphrase
