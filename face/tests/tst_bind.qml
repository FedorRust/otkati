import QtQuick
import QtTest
import Otkati 1.0

TestCase {
    id: testCase
    name: "BindChange"

    Component {
        id: faceComponent
        Face {}
    }

    function test_face_hands_alt_z_until_the_bind_changes() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        compare(face.bind, "Alt+Z")
    }

    function test_changing_the_bind_hands_the_new_chord_to_the_ring() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        face.setBind("Ctrl+Shift+F9")
        compare(face.bind, "Ctrl+Shift+F9")
    }
}
