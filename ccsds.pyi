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

class DecodedPacket:
    scid: int
    vcid: int
    packet: Packet

def decode_packets(source: str) -> typing.Iterable[Packet]:
    """Decode space packet data from the provided source.

    Parameters
    ----------
    source : str
        Source providing stream of space packets to decode. Currently only local
        file paths are supported.

    Returns
    -------
        Iterator of Packets
    """

def decode_frames(
    source: str, frame_len: int, interleave: int
) -> typing.Iterable[Frame]:
    """Decode frames from the byte stream provided by source.

    The stream is assumed to be a standard CCSDS CADU stream, i.e., pseudo-noise encoded,
    utilizing the standard attached sync marker, and optionally using Reed-Solomon forward
    error correction.

    Parameters
    ----------
    source: str
        Source of stream containing CADUs using the standard CCSDS ASM that are pseudo
        randomized. Currently, only local file paths are supported.

    frame_len : int
        Length of each frame. This will be the overall CADU length minus the ASM bytes.
        If using Reed-Solomon this must be the interleave * RS message size (255). If
        this value is < 0 a ValueError will be raised.

    interleave : int
        The Reed-Solomon interleave. Typical values include 4 o4 5. If this is not set
        no Reed-Solomon FEC is used and it is assumed the frames will not include any
        Reed-Solomon parity bytes.

    Returns
    -------
    FrameIterator
        An interable providing all decoded Frames.
    """

def decode_framed_packets(
    source: str,
    scid: int,
    frame_len: int,
    izone_len: int = 0,
    trailer_len: int = 0,
    interleave: int | None = None,
) -> typing.Iterable[DecodedPacket]:
    """
    Decode space packets from the byte stream provided by source.

    The stream is assumed to be a standard CCSDS CADU stream, i.e., pseudo-noise encoded,
    utilizing the standard attached sync marker, and optionally using Reed-Solomon forward
    error correction.

    Parameters
    ----------
    source: str
        Source of stream containing CADUs using the standard CCSDS ASM that are pseudo
        randomized. Currently, only local file paths are supported.

    scid : int
        Spacecraft identifier for the spacecraft that is the source of the data

    frame_len : int
        Length of each frame. This will be the overall CADU length minus the ASM bytes.
        If using Reed-Solomon this must be the interleave * RS message size (255). If
        this value is < 0 a ValueError will be raised.

    izone_len : int
        Frame insert-zone number of bytes used by the spacecraft, if any.

    trailer_len : int
        Frame trailer number of bytes used by the spacecraft, if any.

    interleave : int
        The Reed-Solomon interleave. Typical values include 4 o4 5. If this is not set
        no Reed-Solomon FEC is used and it is assumed the frames will not include any
        Reed-Solomon parity bytes.

    Returns
    -------
    DecodedPacketIterator
        An interable providing all DecodedPackets
    """

def decode_cdc_timecode(dat: bytes) -> int:
    """Decode the provided CCSDS Day-Segmented timecode bytes into UTC milliseconds.

    Parameters
    ----------
    dat : bytearray
        Byte array of at least 8 bytes for a CSD timecode. Only the first 8 are used
        if there are more. Raises a ValueError if there are not enough bytes to decode.
    """

def decode_eoscuc_timecode(dat: bytes) -> int:
    """Decode provided bytes representing a CCSDS Unsegmented Timecode as used by the
    NASA EOS mission (Aqua & Terra) into a UTC timestamp in milliseconds.
    """

def missing_packets(cur: int, last: int) -> int:
    """Calculate the number of missing packets between cur and last.
    """

def missing_frames(cur: int, last: int) -> int:
    """Calculate the number of missing frames between cur and last.
    """
