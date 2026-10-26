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

    function test_success_plashka_uses_the_theme_color() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        useTheme(face, "North")
        face.showPlashka(Face.PlashkaEvent.Saved)
        compare(face.visible, true)
        compare(face.plashkaColor, "#010101")

        useTheme(face, "South")
        compare(face.plashkaColor, "#202020")
    }

    function test_each_bind_shows_the_success_plashka_immediately() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        useTheme(face, "North")

        face.showPlashka(Face.PlashkaEvent.Saved)
        compare(face.visible, true)
        compare(face.plashkaEvent, Face.PlashkaEvent.Saved)
        compare(face.plashkaColor, "#010101")
        compare(face.plashkaText, "Saved")

        face.showPlashka(Face.PlashkaEvent.Saved)
        compare(face.visible, true)
        compare(face.plashkaEvent, Face.PlashkaEvent.Saved)
        compare(face.plashkaColor, "#010101")
        compare(face.plashkaText, "Saved")
        face.setLanguage("ru")
        compare(face.plashkaText, "Откат сохранён")
    }
}
