use chrono::{Datelike, Timelike};
use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{timezone_utc, PyDateTime},
};
use std::{fs::File, io::Read};

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

#[pyclass]
struct Packet {
    #[pyo3(get)]
    header: PrimaryHeader,
    #[pyo3(get)]
    data: Vec<u8>,
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
        // slf.packets.get(index).map(|user| user.clone_ref(slf.py()))
    }
}

/// Formats the sum of two numbers as string.
#[pyfunction]
fn read_packets(path: PyObject) -> PyResult<PacketIterator> {
    let path = match Python::with_gil(|py| -> PyResult<String> { path.extract(py) }) {
        Ok(s) => s,
        Err(e) => return Err(e),
    };

    let file: Box<dyn Read + Send> = Box::new(File::open(path)?);
    let packets = ccsds::read_packets(file).filter_map(Result::ok);

    Ok(PacketIterator {
        packets: Box::new(packets),
    })
}

#[pyclass]
#[derive(Clone, Debug)]
enum RSState {
    Ok,
    Corrected,
    Uncorrectable,
    NotPerformed,
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

impl Frame {
    fn new(decoded_frame: ccsds::DecodedFrame) -> Self {
        use ccsds::RSState::*;
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

#[pyfunction]
fn read_frames(path: &str, interleave: i32) -> PyResult<FrameIterator> {
    let interleave: u8 = match interleave.try_into() {
        Ok(x) => x,
        Err(_) => return Err(PyValueError::new_err("interleave must be between 1 and 7")),
    };

    let file: Box<dyn Read + Send> = Box::new(File::open(path)?);
    let frames = ccsds::FrameDecoderBuilder::new(255 * interleave as i32 + 4)
        .reed_solomon_interleave(interleave)
        .build(file);

    Ok(FrameIterator {
        frames: Box::new(frames),
    })
}

#[pyfunction(signature=(path, scid, interleave, izone_len=0, trailer_len=0))]
fn read_framed_packets(
    path: &str,
    scid: i32,
    interleave: i32,
    izone_len: Option<i32>,
    trailer_len: Option<i32>,
) -> PyResult<PacketIterator> {
    let interleave: u8 = match interleave.try_into() {
        Ok(x) => x,
        Err(_) => return Err(PyValueError::new_err("interleave must be between 1 and 7")),
    };
    let scid: u16 = match scid.try_into() {
        Ok(x) => x,
        Err(_) => return Err(PyValueError::new_err("scid must be between 1 and 16535")),
    };
    let izone_len: usize = match izone_len {
        Some(x) => match x.try_into() {
            Ok(x) => x,
            Err(_) => return Err(PyValueError::new_err("izone_len must be >= 0")),
        },
        None => 0,
    };
    let trailer_len: usize = match trailer_len {
        Some(x) => match x.try_into() {
            Ok(x) => x,
            Err(_) => return Err(PyValueError::new_err("izone_len must be >= 0")),
        },
        None => 0,
    };

    let file: Box<dyn Read + Send> = Box::new(File::open(path)?);
    let frames = ccsds::FrameDecoderBuilder::new(255 * interleave as i32 + 4)
        .reed_solomon_interleave(interleave)
        .build(file);
    let packets = ccsds::decode_framed_packets(scid, Box::new(frames), izone_len, trailer_len);

    Ok(PacketIterator {
        packets: Box::new(packets),
    })
}

#[pyfunction]
#[pyo3(pass_module)]
fn decode_cds_timecode<'p>(m: &'p PyModule, dat: &'p [u8]) -> PyResult<&'p PyDateTime> {
    match ccsds::timecode::decode_cds_timecode(dat) {
        Ok(tc) => PyDateTime::new(
            m.py(),
            tc.year(),
            tc.month().try_into()?,
            tc.day().try_into()?,
            tc.hour().try_into()?,
            tc.minute().try_into()?,
            tc.second().try_into()?,
            (tc.nanosecond() / 1000).try_into()?,
            Some(timezone_utc(m.py())),
        ),
        Err(_) => Err(PyValueError::new_err("not enough bytes")),
    }
}

#[pyfunction]
#[pyo3(pass_module)]
fn decode_eoscuc_timecode<'p>(m: &'p PyModule, dat: &'p [u8]) -> PyResult<&'p PyDateTime> {
    match ccsds::timecode::decode_eoscuc_timecode(dat) {
        Ok(tc) => PyDateTime::new(
            m.py(),
            tc.year(),
            tc.month().try_into()?,
            tc.day().try_into()?,
            tc.hour().try_into()?,
            tc.minute().try_into()?,
            tc.second().try_into()?,
            (tc.nanosecond() / 1000).try_into()?,
            Some(timezone_utc(m.py())),
        ),
        Err(_) => Err(PyValueError::new_err("not enough bytes")),
    }
}

#[pyfunction]
fn missing_packets(cur: u16, last: u16) -> PyResult<u16> {
    Ok(ccsds::missing_packets(cur, last))
}

#[pyfunction]
fn missing_frames(cur: u32, last: u32) -> PyResult<u32> {
    Ok(ccsds::missing_frames(cur, last))
}

/// ccsdspy
///
/// Python wrapper for the [ccsds](https://github.com/bmflynn/ccsds) Rust crate.
#[pymodule]
fn ccsdspy(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(read_packets, m)?)?;
    m.add_class::<Packet>()?;
    m.add_class::<PrimaryHeader>()?;

    m.add_function(wrap_pyfunction!(read_frames, m)?)?;
    m.add_function(wrap_pyfunction!(read_framed_packets, m)?)?;
    m.add_class::<Frame>()?;
    m.add_class::<VCDUHeader>()?;

    m.add_function(wrap_pyfunction!(decode_cds_timecode, m)?)?;
    m.add_function(wrap_pyfunction!(decode_eoscuc_timecode, m)?)?;

    m.add_function(wrap_pyfunction!(missing_packets, m)?)?;
    m.add_function(wrap_pyfunction!(missing_frames, m)?)?;

    Ok(())
}
