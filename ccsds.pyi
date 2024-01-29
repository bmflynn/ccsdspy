import enum
import typing

class RSState(enum.Enum):
    OK = 0
    Corrected = 1
    Uncorrected = 2
    NotPerformed = 3

class VCDUHeader:
    version: int
    scid: int
    vcid: int
    counter: int
    replay: bool
    cycle: bool
    counter_cycle: int

class Frame:
    header: VCDUHeader
    rsstate: RSState
    data: bytes

class PrimaryHeader:
    version: int
    type_flag: int
    has_secondary_header: bool
    apid: int
    sequence_flags: int
    sequence_id: int
    len_minus1: int

    @classmethod
    def decode(cls, dat: bytes) -> PrimaryHeader:
        """Decode `dat` into a PrimaryHeader"""

class Packet:
    header: PrimaryHeader
    data: bytes

    @classmethod
    def decode(cls, dat: bytes) -> Packet:
        """Decode `dat` into a Packet"""

def read_packets(path: str) -> typing.Iterable[Packet]:
    """Decode packets from the file at `path`.

    The file must be byte-aligned and packets must be contiguous.
    """

def read_frames(path: str, interleave: int) -> typing.Iterable[Frame]:
    """Decode frames from the file at `path`.

    Standard CCSDS decoding is performed so pseudo-noise is removed and Reed-Solomon
    FEC (223/255) is applied. Each frames `rsstate` will indicate the RS disposition.

    :param path: Path to a file containing CADUs. The data need not by synchronized.
    :param interleave:
        RS interleave; typically 4 or 5. This value sets the expected size of a CADU (ASM + RS
        codeblock), i.e., cadu_len = (255 * <interleave>) + 4, where 255 is the RS message size
        and 4 is the length of the standard ASM.
    """

def read_framed_packets(
    path: str,
    scid: int,
    interleave: int,
    izone_len: int = 0,
    trailer_len: int = 0,
) -> typing.Iterable[Packet]:
    """Decode packets from the CADU file at `path`.

    Standard CCSDS decoding is performed so pseudo-noise is removed and Reed-Solomon
    FEC (223/255) is applied. Each frames `rsstate` will indicate the RS disposition.

    :param path: Path to a file containing CADUs. The data need not by synchronized.
    :param scid: Spacecraft identifier. Any frames with an SCID other than this will be dropped.
    :param interleave:
        RS interleave; typically 4 or 5. This value sets the expected size of a CADU (ASM + RS
        codeblock), i.e., cadu_len = (255 * <interleave>) + 4, where 255 is the RS message size
        and 4 is the length of the standard ASM.
    :param izone_len: Length of the insert zone, or 0 if not used.
    :param trailer_len: Length of the trailer, or 0 if not used.
    """

def decode_cdc_timecode(dat: bytes) -> int:
    """Decode provided bytes representing a CCSDS Day Segmented timecode into a UTC
    timestamp in milliseconds.
    """

def decode_eoscuc_timecode(dat: bytes) -> int:
    """Decode provided bytes representing a CCSDS Unsegmented Timecode as used by the
    NASA EOS mission (Aqua & Terra) into a UTC timestamp in milliseconds.
    """

def missing_packets(cur: int, last: int) -> int:
    """Calculate the number of missing packets between cur and last.

    :rasies OverflowError: If cur or last doesn't fit in a u16.
    """

def missing_frames(cur: int, last: int) -> int:
    """Calculate the number of missing frames between cur and last.

    :rasies OverflowError: If cur or last doesn't fit in a u32.
    """
