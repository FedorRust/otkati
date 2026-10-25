import QtQuick
import QtTest
import Otkati 1.0

TestCase {
    id: testCase
    name: "NoSpacePlashka"

    Component {
        id: faceComponent
        Face {}
    }

    function test_no_space_plashka_is_its_own_kind() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        verify(Face.PlashkaEvent.NoSpace !== Face.PlashkaEvent.NothingToSave)
        verify(Face.PlashkaEvent.NoSpace !== Face.PlashkaEvent.ClipNotSaved)

        face.showPlashka(Face.PlashkaEvent.NoSpace)
        compare(face.plashkaEvent, Face.PlashkaEvent.NoSpace)
        compare(face.plashkaText, "")
        face.setLanguage("ru")
        compare(face.plashkaText, "")
        compare(face.plashkaEvent, Face.PlashkaEvent.NoSpace)
    }

    function test_no_space_plashka_is_red_in_any_theme() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)

        face.setTheme("#336699")
        face.showPlashka(Face.PlashkaEvent.NoSpace)
        compare(face.themeColor, "#336699")
        compare(face.plashkaColor, "#ff0000")

        face.setTheme("#aabbcc")
        compare(face.themeColor, "#aabbcc")
        compare(face.plashkaColor, "#ff0000")
        compare(face.plashkaEvent, Face.PlashkaEvent.NoSpace)
    }
}
