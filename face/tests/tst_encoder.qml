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

    function test_an_item_the_card_cannot_do_stays_visible_and_is_not_chosen() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        face.setTheme("#336699")
        face.setOfferedEncoders(["H.264", "HEVC"])

        compare(face.encoderItems[2], "AV1")
        compare(face.encoderSelectable("AV1"), false)
        compare(face.encoderSelectable("H.264"), true)
        compare(face.encoderSelectable("HEVC"), true)

        face.chooseEncoder("HEVC")
        compare(face.encoder, "HEVC")

        face.showPlashka(Face.PlashkaEvent.Saved)
        compare(face.plashkaColor, "#336699")
        face.chooseEncoder("AV1")
        compare(face.encoder, "HEVC")
        compare(face.plashkaEvent, Face.PlashkaEvent.Saved)
        compare(face.plashkaColor, "#336699")
        compare(face.visible, true)
    }

    function test_a_new_machine_stays_on_h264_until_an_offered_item_is_chosen() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        face.setTheme("#336699")
        compare(face.encoder, "H.264")
        compare(face.encoderSelectable("AV1"), false)
        face.chooseEncoder("AV1")
        compare(face.encoder, "H.264")
        compare(face.visible, false)
        compare(face.plashkaEvent, -1)
        compare(face.plashkaColor, "#336699")

        face.setOfferedEncoders(["H.264", "HEVC", "AV1"])
        compare(face.encoder, "H.264")
        face.chooseEncoder("AV1")
        compare(face.encoder, "AV1")
        compare(face.visible, false)
        compare(face.plashkaColor, "#336699")
    }
}
