import QtQuick
import QtTest
import Otkati 1.0

TestCase {
    id: testCase
    name: "NoAccessPlashka"

    Component {
        id: faceComponent
        Face {}
    }

    function test_no_access_plashka_is_painted_with_the_theme_color() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        face.setTheme("#336699")
        verify(Face.PlashkaEvent.NoAccess !== undefined)
        face.showPlashka(Face.PlashkaEvent.NoAccess)
        compare(face.plashkaEvent, Face.PlashkaEvent.NoAccess)
        compare(face.plashkaColor, "#336699")
        face.setTheme("#aabbcc")
        compare(face.plashkaColor, "#aabbcc")
    }

    function test_no_access_plashka_carries_no_phrase() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        face.showPlashka(Face.PlashkaEvent.NothingToSave)
        face.showPlashka(Face.PlashkaEvent.NoAccess)
        compare(face.plashkaText, "")
        compare(face.plashkaColor, "#000000")
        face.setLanguage("ru")
        compare(face.plashkaText, "")
        compare(face.plashkaColor, "#000000")
        compare(face.plashkaEvent, Face.PlashkaEvent.NoAccess)
    }
}
