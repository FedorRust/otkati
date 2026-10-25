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
        verify(Face.PlashkaEvent.NoSpace !== Face.PlashkaEvent.ClipDidNotSave)

        face.showPlashka(Face.PlashkaEvent.NoSpace)
        compare(face.plashkaEvent, Face.PlashkaEvent.NoSpace)
        compare(face.plashkaText, "")
        face.setLanguage("ru")
        compare(face.plashkaText, "")
        compare(face.plashkaEvent, Face.PlashkaEvent.NoSpace)
    }

    function filePath(name) {
        let url = Qt.resolvedUrl("fixtures/" + name).toString()
        if (url.startsWith("file://"))
            url = decodeURIComponent(url.slice("file://".length))
        return url
    }

    function useTheme(face, name) {
        face.cataloguePath = filePath("themes")
        face.userCataloguePath = ""
        face.loadCatalogue()
        face.setTheme(name)
    }

    function test_no_space_plashka_is_red_in_any_theme() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)

        useTheme(face, "North")
        face.showPlashka(Face.PlashkaEvent.NoSpace)
        compare(face.themeColor, "#010101")
        compare(face.plashkaColor, "#ff0000")

        useTheme(face, "South")
        compare(face.themeColor, "#202020")
        compare(face.plashkaColor, "#ff0000")
        compare(face.plashkaEvent, Face.PlashkaEvent.NoSpace)
    }
}
