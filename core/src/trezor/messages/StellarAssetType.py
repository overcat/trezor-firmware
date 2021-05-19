# Automatically generated by pb2py
# fmt: off
# isort:skip_file
import protobuf as p

if __debug__:
    try:
        from typing import Dict, List, Optional  # noqa: F401
        from typing_extensions import Literal  # noqa: F401
    except ImportError:
        pass


class StellarAssetType(p.MessageType):

    def __init__(
        self,
        *,
        type: int,
        code: Optional[str] = None,
        issuer: Optional[str] = None,
    ) -> None:
        self.type = type
        self.code = code
        self.issuer = issuer

    @classmethod
    def get_fields(cls) -> Dict:
        return {
            1: ('type', p.UVarintType, p.FLAG_REQUIRED),
            2: ('code', p.UnicodeType, None),
            3: ('issuer', p.UnicodeType, None),
        }