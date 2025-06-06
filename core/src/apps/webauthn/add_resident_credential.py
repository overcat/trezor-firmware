from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from trezor.messages import Success, WebAuthnAddResidentCredential


async def add_resident_credential(msg: WebAuthnAddResidentCredential) -> Success:
    import storage.device as storage_device
    from trezor import TR, wire
    from trezor.messages import Success
    from trezor.ui.layouts.fido import confirm_fido, credential_warning

    from .credential import Fido2Credential
    from .resident_credentials import store_resident_credential

    if not storage_device.is_initialized():
        raise wire.NotInitialized("Device is not initialized")
    if not msg.credential_id:
        raise wire.ProcessError("Missing credential ID parameter.")

    try:
        cred = Fido2Credential.from_cred_id(bytes(msg.credential_id), None)
    except Exception:
        await credential_warning(
            "warning_credential",
            TR.fido__does_not_belong,
        )

    await confirm_fido(
        TR.fido__title_import_credential,
        cred.app_name(),
        cred.icon_name(),
        [cred.account_name()],
    )

    if store_resident_credential(cred):
        return Success(message="Credential added")
    else:
        raise wire.ProcessError("Internal credential storage is full.")
