import QtQuick
import QtTest
import Otkati 1.0

TestCase {
    id: testCase
    name: "Length"

    Component {
        id: faceComponent
        Face {}
    }

    function test_face_hands_ten_minutes_until_the_length_changes() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        compare(face.length, 600)
    }

    function test_changing_the_length_hands_the_new_length_to_the_ring() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        face.setLength(15)
        compare(face.length, 15)
        face.setLength(1200)
        compare(face.length, 1200)
    }
}
