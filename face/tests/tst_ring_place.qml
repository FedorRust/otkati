import QtQuick
import QtTest
import Otkati 1.0

TestCase {
    id: testCase
    name: "RingPlace"

    Component {
        id: faceComponent
        Face {}
    }

    function test_face_hands_memory_until_the_place_changes() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        compare(face.ringPlace, Face.RingPlace.Memory)
    }

    function test_choosing_disk_hands_the_place_to_the_ring() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        face.setRingPlace(Face.RingPlace.Disk)
        compare(face.ringPlace, Face.RingPlace.Disk)
        face.setRingPlace(Face.RingPlace.Memory)
        compare(face.ringPlace, Face.RingPlace.Memory)
    }
}
