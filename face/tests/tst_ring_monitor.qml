import QtQuick
import QtTest
import Otkati 1.0

TestCase {
    id: testCase
    name: "RingMonitor"

    Component {
        id: faceComponent
        Face {}
    }

    function test_the_face_hands_no_ring_monitor_until_the_person_chooses() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        compare(face.ringMonitor, "")
        compare(face.knob("ringMonitor"), "")
    }

    function test_changing_the_ring_monitor_hands_it_to_the_ring() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        face.setRingMonitor("desk")
        compare(face.ringMonitor, "desk")
        compare(face.knob("ringMonitor"), "desk")
        face.setRingMonitor("laptop")
        compare(face.ringMonitor, "laptop")
        compare(face.knob("ringMonitor"), "laptop")
    }
}
