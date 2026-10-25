import QtQuick
import QtTest
import Otkati 1.0

TestCase {
    id: testCase
    name: "Encoder"

    Component {
        id: faceComponent
        Face {}
    }

    function test_the_knob_lists_three_items_and_chooses_h264() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        compare(face.encoderItems.length, 3)
        compare(face.encoderItems[0], "H.264")
        compare(face.encoderItems[1], "HEVC")
        compare(face.encoderItems[2], "AV1")
        compare(face.encoder, "H.264")
    }

    function test_each_item_is_marked_for_the_cards_it_suits() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        compare(face.encoderMark("H.264"), "NVIDIA, AMD, Intel, and machines with no hardware encoder")
        compare(face.encoderMark("HEVC"), "NVIDIA, including the 3060 Ti and 3070 Ti, and AMD or Intel when the card can do HEVC")
        compare(face.encoderMark("AV1"), "NVIDIA from the 40 series and AMD from the RX 7000")
    }

    function test_marks_follow_russian() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        face.chooseLanguage("ru")
        compare(face.encoderMark("H.264"), "NVIDIA, AMD, Intel и компьютеры без аппаратного кодировщика")
        compare(face.encoderMark("HEVC"), "NVIDIA, включая 3060 Ti и 3070 Ti, и AMD или Intel, если карта умеет HEVC")
        compare(face.encoderMark("AV1"), "NVIDIA с 40-й серии и AMD с RX 7000")
        face.chooseLanguage("en")
        compare(face.encoderMark("H.264"), "NVIDIA, AMD, Intel, and machines with no hardware encoder")
    }
}
