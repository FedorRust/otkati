import QtQuick
import QtTest
import Otkati 1.0

TestCase {
    id: testCase
    name: "RingSwitch"

    Component {
        id: faceComponent
        Face {}
    }

    function test_face_hands_the_switch_on_until_it_is_turned_off() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        compare(face.ringOn, true)
    }

    function test_turning_the_switch_hands_it_to_the_ring() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        face.setSwitch(false)
        compare(face.ringOn, false)
        face.setSwitch(true)
        compare(face.ringOn, true)
    }
}
