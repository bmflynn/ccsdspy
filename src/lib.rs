use pyo3::{
    exceptions::{PyFileNotFoundError, PyValueError},
    prelude::*,
    types::PyType,
};
use std::{
    fs::File,
    io::{BufReader, Read},
};

#[pyclass]
#[derive(Clone, Debug)]
struct PrimaryHeader {
    #[pyo3(get)]
    version: u8,
    #[pyo3(get)]
    type_flag: u8,
    #[pyo3(get)]
    has_secondary_header: bool,
    #[pyo3(get)]
    apid: u16,
    #[pyo3(get)]
    sequence_flags: u8,
    #[pyo3(get)]
    sequence_id: u16,
    #[pyo3(get)]
    len_minus1: u16,
}

#[pymethods]
impl PrimaryHeader {
    fn __repr__(&self) -> String {
        self.__str__()
    }
    fn __str__(&self) -> String {
        format!(
            "PrimaryHeader(version={}, type_flag={}, has_secondary_header={}, apid={}, sequence_flags={}, sequence_id={}, len_minus1={})",
            self.version, self.type_flag, self.has_secondary_header, self.apid, self.sequence_flags, self.sequence_id, self.len_minus1,
        ).to_owned()
    }

    #[classmethod]
    fn decode(_cls: &PyType, dat: &[u8]) -> Option<Self> {
        ccsds::PrimaryHeader::decode(dat).map(|hdr| Self {
            version: hdr.version,
            type_flag: hdr.type_flag,
            has_secondary_header: hdr.has_secondary_header,
            apid: hdr.apid,
            sequence_flags: hdr.sequence_flags,
            sequence_id: hdr.sequence_id,
            len_minus1: hdr.len_minus1,
        })
    }
}

#[pyclass]
struct Packet {
    #[pyo3(get)]
    header: PrimaryHeader,
    #[pyo3(get)]
    data: Vec<u8>,
}

#[pymethods]
impl Packet {
    fn __repr__(&self) -> String {
        self.__str__()
    }
    fn __str__(&self) -> String {
        format!(
            "Packet(header={}, data_len={})",
            self.header.__str__(),
            self.data.len()
        )
        .to_owned()
    }
    #[classmethod]
    fn decode(_cls: &PyType, dat: &[u8]) -> Option<Self> {
        ccsds::Packet::decode(dat).map(Packet::new)
    }
}

impl Packet {
    fn new(packet: ccsds::Packet) -> Self {
        Packet {
            header: PrimaryHeader {
                version: packet.header.version,
                type_flag: packet.header.type_flag,
                has_secondary_header: packet.header.has_secondary_header,
                apid: packet.header.apid,
                sequence_flags: packet.header.sequence_flags,
                sequence_id: packet.header.sequence_id,
                len_minus1: packet.header.len_minus1,
            },
            data: packet.data.clone(),
        }
    }
}

#[pyclass]
struct DecodedPacket {
    #[pyo3(get)]
    scid: u16,
    vcid: u16,
    packet: Packet,
}

#[pymethods]
impl DecodedPacket {
    fn __repr__(&self) -> String {
        self.__str__()
    }
    fn __str__(&self) -> String {
        format!(
            "DecodedPacket(scid={}, vcid={}, packet={})",
            self.scid,
            self.vcid,
            self.packet.__str__(),
        )
        .to_owned()
    }
}

impl DecodedPacket {
    fn new(packet: ccsds::DecodedPacket) -> Self {
        DecodedPacket {
            scid: packet.scid,
            vcid: packet.vcid,
            packet: Packet::new(packet.packet),
        }
    }
}

#[pyclass]
struct PacketIterator {
    packets: Box<dyn Iterator<Item = ccsds::Packet> + Send>,
}

#[pymethods]
impl PacketIterator {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<Self>) -> Option<Py<Packet>> {
        match slf.packets.next() {
            Some(packet) => Py::new(slf.py(), Packet::new(packet)).ok(),
            None => None,
        }
    }
}

/// Decode space packet data from the provided source.
///
/// Parameters
/// ----------
/// source : str
///     Source providing stream of space packets to decode. Currently only local
///     file paths are supported.
///
/// Returns
/// -------
///     Iterator of Packets
#[pyfunction]
fn decode_packets(source: PyObject) -> PyResult<PacketIterator> {
    let path = match Python::with_gil(|py| -> PyResult<String> { source.extract(py) }) {
        Ok(s) => s,
        Err(e) => return Err(e),
    };

    let file: Box<dyn Read + Send> = Box::new(File::open(path)?);
    let packets: Box<dyn Iterator<Item = ccsds::Packet> + Send + 'static> =
        Box::new(ccsds::read_packets(file).filter_map(Result::ok));

    Ok(PacketIterator { packets })
}

#[pyclass]
struct DecodedPacketIterator {
    packets: Box<dyn Iterator<Item = ccsds::DecodedPacket> + Send>,
}

#[pymethods]
impl DecodedPacketIterator {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<Self>) -> Option<Py<DecodedPacket>> {
        match slf.packets.next() {
            Some(packet) => Py::new(slf.py(), DecodedPacket::new(packet)).ok(),
            None => None,
        }
    }
}

#[pyclass]
#[derive(Clone, Debug)]
enum RSState {
    Ok,
    Corrected,
    Uncorrectable,
    NotPerformed,
}

#[pymethods]
impl RSState {
    fn __repr__(&self) -> String {
        self.__str__()
    }
    fn __str__(&self) -> String {
        match self {
            Self::Ok => "ok",
            Self::Corrected => "corrected",
            Self::Uncorrectable => "uncorrectable",
            Self::NotPerformed => "notperformed",
        }
        .to_owned()
    }
}

#[pyclass]
#[derive(Clone, Debug)]
struct VCDUHeader {
    #[pyo3(get)]
    version: u8,
    #[pyo3(get)]
    scid: u16,
    #[pyo3(get)]
    vcid: u16,
    #[pyo3(get)]
    counter: u32,
    #[pyo3(get)]
    replay: bool,
    #[pyo3(get)]
    cycle: bool,
    #[pyo3(get)]
    counter_cycle: u8,
}

#[pymethods]
impl VCDUHeader {
    fn __repr__(&self) -> String {
        self.__str__()
    }
    fn __str__(&self) -> String {
        format!(
            "VCDUHeader(version={}, scid={}, vcid={}, counter={}, replay={}, cycle={}, counter_cycle={})",
            self.version, self.scid, self.vcid, self.counter, self.replay, self.cycle, self.counter_cycle,
        ).to_owned()
    }
}

#[pyclass]
#[derive(Clone, Debug)]
struct Frame {
    #[pyo3(get)]
    header: VCDUHeader,
    #[pyo3(get)]
    rsstate: RSState,
    #[pyo3(get)]
    data: Vec<u8>,
}

#[pymethods]
impl Frame {
    fn __repr__(&self) -> String {
        self.__str__()
    }
    fn __str__(&self) -> String {
        format!(
            "Frame(header={}, rsstate={}, data_len={})",
            self.header.__str__(),
            self.rsstate.__str__(),
            self.data.len(),
        )
        .to_owned()
    }
}

impl Frame {
    fn new(decoded_frame: ccsds::DecodedFrame) -> Self {
        use ccsds::RSState::{Corrected, NotPerformed, Ok, Uncorrectable};
        let frame = decoded_frame.frame;
        let h = frame.header;
        Frame {
            header: VCDUHeader {
                version: h.version,
                scid: h.scid,
                vcid: h.vcid,
                counter: h.counter,
                replay: h.replay,
                cycle: h.cycle,
                counter_cycle: h.counter_cycle,
            },
            rsstate: match decoded_frame.rsstate {
                Ok => RSState::Ok,
                Corrected(_) => RSState::Corrected,
                Uncorrectable(_) => RSState::Uncorrectable,
                NotPerformed => RSState::NotPerformed,
            },
            data: frame.data,
        }
    }
}

#[pyclass]
struct FrameIterator {
    frames: Box<dyn Iterator<Item = ccsds::DecodedFrame> + Send>,
}

#[pymethods]
impl FrameIterator {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<Self>) -> Option<Py<Frame>> {
        match slf.frames.next() {
            Some(decoded_frame) => Py::new(slf.py(), Frame::new(decoded_frame)).ok(),
            None => None,
        }
    }
}

/// Decode frames from the byte stream provided by source.
///
/// The decode synchronization process starts immediately in the background and progresses
/// until the first sync marker is found.
///
/// The stream is assumed to be a standard CCSDS CADU stream, i.e., pseudo-noise encoded,
/// utilizing the standard attached sync marker, and optionally using Reed-Solomon forward
/// error correction.
///
/// Parameters
/// ----------
/// source: str
///     Source of stream containing CADUs using the standard CCSDS ASM that are pseudo
///     randomized. Currently, only local file paths are supported.
///
/// frame_len : int
///     Length of each frame. This will be the overall CADU length minus the ASM bytes.
///     If using Reed-Solomon this must be the interleave * RS message size (255). If
///     this value is < 0 a ValueError will be raised.
///
/// interleave : int
///     The Reed-Solomon interleave. Typical values include 4 o4 5. If this is not set
///     no Reed-Solomon FEC is used and it is assumed the frames will not include any
///     Reed-Solomon parity bytes.
///
/// Returns
/// -------
/// FrameIterator
///     An interable providing all decoded Frames.
#[pyfunction(signature=(source, frame_len, interleave=None))]
fn decode_frames(source: &str, frame_len: i32, interleave: Option<i32>) -> PyResult<FrameIterator> {
    if frame_len < 0 {
        return Err(PyValueError::new_err("frame_size cannot be > 0"));
    }
    let file: Box<dyn Read + Send> = Box::new(File::open(source)?);
    let blocks =
        ccsds::Synchronizer::new(file, &ccsds::ASM.to_vec(), frame_len.try_into().unwrap())
            .into_iter()
            .filter_map(Result::ok);

    let mut builder = ccsds::FrameDecoderBuilder::default();

    if let Some(interleave) = interleave {
        if !(2..=10).contains(&interleave) {
            return Err(PyValueError::new_err(format!(
                "improbable interleave value; expected 2..10: got {interleave}"
            )));
        }
        let interleave: u8 = interleave.try_into().unwrap(); // checked above
        builder = builder.reed_solomon_interleave(interleave);
    }

    let frames = builder.build().start(blocks).filter_map(Result::ok);

    Ok(FrameIterator {
        frames: Box::new(frames),
    })
}

/// Decode space packets from the byte stream provided by source.
///
/// The decode synchronization process starts immediately in the background and progresses
/// until the first sync marker is found.
///
/// The stream is assumed to be a standard CCSDS CADU stream, i.e., pseudo-noise encoded,
/// utilizing the standard attached sync marker, and optionally using Reed-Solomon forward
/// error correction.
///
/// Parameters
/// ----------
/// source: str
///     Source of stream containing CADUs using the standard CCSDS ASM that are pseudo
///     randomized. Currently, only local file paths are supported.
///
/// scid : int
///     Spacecraft identifier for the spacecraft that is the source of the data
///
/// cadu_len: int
///     The length of the CADU, i.e., the ASM length plus the length of the frame plus the
///     length of any integrity or parity bytes.
///
///     When using Reed-Solomon, this will typically be 1024 for interleave=4 and 1279 when
///     using interleave=5.
///
/// izone_len : int
///     Frame insert-zone number of bytes used by the spacecraft, if any.
///
/// trailer_len : int
///     Frame trailer number of bytes used by the spacecraft, if any.
///
/// interleave : int
///     The Reed-Solomon interleave. Typical values include 4 o4 5. If this is not set
///     no Reed-Solomon FEC is used and it is assumed the frames will not include any
///     Reed-Solomon parity bytes.
///
/// Returns
/// -------
/// DecodedPacketIterator
///     An interable providing all DecodedPackets
#[pyfunction(signature=(source, scid, cadu_len, izone_len=0, trailer_len=0, interleave=None))]
fn decode_framed_packets(
    source: &str,
    scid: i32,
    cadu_len: i32,
    izone_len: Option<i32>,
    trailer_len: Option<i32>,
    interleave: Option<i32>,
) -> PyResult<DecodedPacketIterator> {
    if cadu_len < 4 {
        return Err(PyValueError::new_err(
            "cadu_len cannot be less than the ASM size (4)",
        ));
    }
    if !(0..16384).contains(&scid) {
        return Err(PyValueError::new_err(format!(
            "invalid scid value; expected 0..16384, got {scid}"
        )));
    }
    let scid: ccsds::SCID = scid.try_into().unwrap();
    let izone_len: usize = if let Some(x) = izone_len {
        if !(0..16).contains(&x) {
            return Err(PyValueError::new_err(format!(
                "invalid izone_len value; expected 0..16, got {x}"
            )));
        }
        x.try_into().unwrap()
    } else {
        0
    };
    let trailer_len: usize = if let Some(x) = trailer_len {
        if !(0..16).contains(&x) {
            return Err(PyValueError::new_err(format!(
                "invalid trailer_len value; expected 0..16, got {x}"
            )));
        }
        x.try_into().unwrap()
    } else {
        0
    };

    let file = BufReader::new(File::open(source)?);
    let block_size: usize = usize::try_from(cadu_len).unwrap() - ccsds::ASM.len();
    let blocks = ccsds::Synchronizer::new(file, &ccsds::ASM.to_vec(), block_size)
        .into_iter()
        .filter_map(Result::ok);

    let mut builder = ccsds::FrameDecoderBuilder::default();
    if let Some(interleave) = interleave {
        if !(2..=10).contains(&interleave) {
            return Err(PyValueError::new_err(
                "invalid interleave value; expected 2..10: got {interleave}",
            ));
        }
        let interleave: u8 = interleave.try_into().unwrap(); // checked above
        builder = builder.reed_solomon_interleave(interleave);
    }
    let frames = builder.build().start(blocks).filter_map(Result::ok);

    let packets: Box<dyn Iterator<Item = ccsds::DecodedPacket> + Send + 'static> = Box::new(
        ccsds::decode_framed_packets(scid, frames, izone_len, trailer_len),
    );

    Ok(DecodedPacketIterator { packets })
}

/// Decode the provided CCSDS Day-Segmented timecode bytes into UTC milliseconds.
///
/// Parameters
/// ----------
/// dat : bytearray
///     Byte array of at least 8 bytes for a CSD timecode. Only the first 8 are used
///     if there are more. Raises a ValueError if there are not enough bytes to decode.
#[pyfunction(signature=(dat))]
fn decode_cds_timecode(dat: &[u8]) -> PyResult<i64> {
    match ccsds::timecode::decode_cds(dat) {
        Ok(tc) => Ok(tc.timestamp_millis()),
        Err(_) => Err(PyValueError::new_err("not enough bytes")),
    }
}

/// Decode provided bytes representing a CCSDS Unsegmented Timecode as used by the
/// NASA EOS mission (Aqua & Terra) into a UTC timestamp in milliseconds.
#[pyfunction(signature=(dat))]
fn decode_eoscuc_timecode(dat: &[u8]) -> PyResult<i64> {
    match ccsds::timecode::decode_eoscuc(dat) {
        Ok(tc) => Ok(tc.timestamp_millis()),
        Err(_) => Err(PyValueError::new_err("not enough bytes")),
    }
}

/// Calculate the number of missing packets between cur and last.
///
/// Note, packet sequence counters are per-APID.
#[pyfunction(signature=(cur, last))]
fn missing_packets(cur: u16, last: u16) -> u16 {
    ccsds::missing_packets(cur, last)
}

/// Calculate the number of missing frames between cur and last.
///
/// Note frame sequence counts are per-VCID.
#[pyfunction(signature=(cur, last))]
fn missing_frames(cur: u32, last: u32) -> u32 {
    ccsds::missing_frames(cur, last)
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct PnConfig;

impl PnConfig {
    fn new(config: Option<spacecrafts::PnConfig>) -> Option<Self> {
        match config {
            Some(_) => Some(Self {}),
            None => None,
        }
    }
}

#[pymethods]
impl PnConfig {
    fn __repr__(&self) -> String {
        self.__str__()
    }
    fn __str__(&self) -> String {
        "PnConfig()".to_string()
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct RSConfig {
    #[pyo3(get)]
    pub interleave: u8,
    #[pyo3(get)]
    pub virtual_fill_length: usize,
    #[pyo3(get)]
    pub num_correctable: u32,
}

#[pymethods]
impl RSConfig {
    fn __repr__(&self) -> String {
        self.__str__()
    }
    fn __str__(&self) -> String {
        format!(
            "RSConfig(interleave={}, virtual_fill_length={}, num_correctable={})",
            self.interleave, self.virtual_fill_length, self.num_correctable
        )
    }
}

impl RSConfig {
    fn new(config: Option<spacecrafts::RSConfig>) -> Option<Self> {
        match config {
            Some(rs) => Some(Self {
                interleave: rs.interleave,
                virtual_fill_length: rs.virtual_fill_length,
                num_correctable: rs.num_correctable,
            }),
            None => None,
        }
    }
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct FramingConfig {
    #[pyo3(get)]
    pub length: usize,
    #[pyo3(get)]
    pub insert_zone_length: usize,
    #[pyo3(get)]
    pub trailer_length: usize,
    #[pyo3(get)]
    pub pseudo_noise: Option<PnConfig>,
    #[pyo3(get)]
    pub reed_solomon: Option<RSConfig>,
}

impl FramingConfig {
    fn new(config: spacecrafts::FramingConfig) -> Self {
        Self {
            length: config.length,
            insert_zone_length: config.insert_zone_length,
            trailer_length: config.trailer_length,
            pseudo_noise: PnConfig::new(config.pseudo_noise),
            reed_solomon: RSConfig::new(config.reed_solomon),
        }
    }
}

#[pymethods]
impl FramingConfig {
    fn __repr__(&self) -> String {
        self.__str__()
    }
    fn __str__(&self) -> String {
        let pn = match &self.pseudo_noise {
            Some(pn) => pn.__str__(),
            None => "None".to_string(),
        };
        let rs = match &self.reed_solomon {
            Some(rs) => rs.__str__(),
            None => "None".to_string(),
        };
        format!("FramingConfig(length={}, insert_zone_length={}, trailer_length={}, pseudo_noise={}, reed_solomon={})",
        self.length, self.insert_zone_length, self.trailer_length, pn, rs).to_string()
    }

    /// Length of the RS codeblock.
    pub fn codeblock_len(&self) -> usize {
        match &self.reed_solomon {
            Some(rs) => self.length + 2 * rs.num_correctable as usize * rs.interleave as usize,
            None => self.length,
        }
    }
}

#[pyfunction]
fn framing_config(scid: u16, path: Option<&str>) -> PyResult<Option<FramingConfig>> {
    match ccsds::framing_config(scid, path) {
        Ok(Some(framing)) => Ok(Some(FramingConfig::new(framing))),
        Ok(None) => Ok(None),
        Err(err) => Err(PyFileNotFoundError::new_err(format!("{err}"))),
    }
}

/// ccsds
///
/// Python wrapper for the [ccsds](https://github.com/bmflynn/ccsds) Rust crate.
#[pymodule]
#[pyo3(name = "ccsds")]
fn ccsdspy(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(decode_packets, m)?)?;
    m.add_class::<Packet>()?;
    m.add_class::<DecodedPacket>()?;
    m.add_class::<PrimaryHeader>()?;
    m.add_class::<RSState>()?;

    m.add_function(wrap_pyfunction!(decode_frames, m)?)?;
    m.add_function(wrap_pyfunction!(decode_framed_packets, m)?)?;
    m.add_class::<Frame>()?;
    m.add_class::<VCDUHeader>()?;

    m.add_function(wrap_pyfunction!(decode_cds_timecode, m)?)?;
    m.add_function(wrap_pyfunction!(decode_eoscuc_timecode, m)?)?;

    m.add_function(wrap_pyfunction!(missing_packets, m)?)?;
    m.add_function(wrap_pyfunction!(missing_frames, m)?)?;
    m.add_function(wrap_pyfunction!(framing_config, m)?)?;

    Ok(())
}
