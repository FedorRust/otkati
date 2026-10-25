import QtQuick
import QtTest
import Otkati 1.0

TestCase {
    id: testCase
    name: "SavedPlashka"

    Component {
        id: faceComponent
        Face {}
    }

    function test_success_plashka_uses_the_theme_color() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        face.setTheme("#336699")
        face.showPlashka(Face.PlashkaEvent.Saved)
        compare(face.visible, true)
        compare(face.plashkaColor, "#336699")

        face.setTheme("#aabbcc")
        compare(face.plashkaColor, "#aabbcc")
    }
}
