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

    function test_a_length_outside_fifteen_seconds_to_twenty_minutes_is_not_handed() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        face.setLength(14)
        compare(face.length, 600)
        face.setLength(1201)
        compare(face.length, 600)
    }

    function test_a_fraction_of_a_second_is_not_handed() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        face.setLength(90)
        face.setLength(91.5)
        compare(face.length, 90)
    }
}
