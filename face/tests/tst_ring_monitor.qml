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

    function test_the_plashka_is_shown_on_the_current_ring_monitor() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        face.showPlashka(Face.PlashkaEvent.Saved)
        compare(face.visible, true)
        compare(face.plashkaMonitor, "")

        face.setRingMonitor("desk")
        compare(face.plashkaMonitor, "desk")
        face.showPlashka(Face.PlashkaEvent.NothingToSave)
        compare(face.visible, true)
        compare(face.plashkaMonitor, "desk")

        face.setRingMonitor("laptop")
        compare(face.plashkaMonitor, "laptop")
        compare(face.visible, true)
    }

    function test_the_plashka_window_sits_on_the_ring_monitor() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        face.monitors = [
            { name: "origin", x: 400, y: 300, width: 1920, height: 1080, focused: true, connector: "DP-1" },
            { name: "side", x: 2000, y: 90, width: 1280, height: 800, focused: false, connector: "HDMI-A-1" }
        ]
        face.showPlashka(Face.PlashkaEvent.Saved)
        compare(face.plashkaMonitor, "")
        compare(face.x, 0)
        compare(face.y, 0)

        face.setRingMonitor("side")
        compare(face.x, 2000)
        compare(face.y, 90)
        face.setRingMonitor("origin")
        compare(face.x, 400)
        compare(face.y, 300)
    }
}
