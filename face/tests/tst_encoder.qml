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
}
