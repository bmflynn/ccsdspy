from pathlib import Path

import ccsdspy


def fixture_path(name: str) -> str:
    return str(Path(__file__).parent / "fixtures" / name)


def test_read_framed_packets():
    packet_iter = ccsdspy.read_framed_packets(
        fixture_path("snpp_synchronized_cadus.dat"), 157, 4
    )
    packets = list(packet_iter)

    assert len(packets) == 12

    for packet in packets:
        assert packet.header.apid in [802, 803]
        assert len(packet.data) == packet.header.len_minus1 + 1 + 6
