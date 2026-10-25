import QtQuick
import QtTest
import Otkati 1.0

TestCase {
    id: testCase
    name: "EmptyRingPlashka"

    Component {
        id: faceComponent
        Face {}
    }

    function test_empty_ring_plashka_says_nothing_to_save() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        face.showPlashka(Face.PlashkaEvent.NothingToSave)
        compare(face.plashkaText, "Nothing to save")
    }
}
