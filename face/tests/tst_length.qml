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
}
