import QtQuick
import QtTest
import Otkati 1.0

TestCase {
    id: testCase
    name: "FolderNotCreatedPlashka"

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

    function test_folder_not_created_plashka_is_painted_with_the_theme_color() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        useTheme(face, "North")
        verify(Face.PlashkaEvent.FolderNotCreated !== undefined)
        face.showPlashka(Face.PlashkaEvent.FolderNotCreated)
        compare(face.visible, true)
        compare(face.plashkaEvent, Face.PlashkaEvent.FolderNotCreated)
        compare(face.plashkaColor, "#010101")
        useTheme(face, "South")
        compare(face.plashkaColor, "#202020")
    }

    function test_folder_not_created_plashka_carries_no_phrase() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        face.showPlashka(Face.PlashkaEvent.NothingToSave)
        face.showPlashka(Face.PlashkaEvent.FolderNotCreated)
        compare(face.plashkaText, "")
        compare(face.plashkaColor, "#000000")
        face.setLanguage("ru")
        compare(face.plashkaText, "")
        compare(face.plashkaColor, "#000000")
        compare(face.plashkaEvent, Face.PlashkaEvent.FolderNotCreated)
    }
}
