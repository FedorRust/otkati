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

    function test_no_access_plashka_is_painted_with_the_theme_color() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        useTheme(face, "North")
        verify(Face.PlashkaEvent.NoAccess !== undefined)
        face.showPlashka(Face.PlashkaEvent.NoAccess)
        compare(face.plashkaEvent, Face.PlashkaEvent.NoAccess)
        compare(face.plashkaColor, "#010101")
        useTheme(face, "South")
        compare(face.plashkaColor, "#202020")
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
