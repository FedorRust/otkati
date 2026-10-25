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

    function test_empty_ring_plashka_is_red_in_any_theme() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)

        useTheme(face, "North")
        face.showPlashka(Face.PlashkaEvent.NothingToSave)
        compare(face.themeColor, "#010101")
        compare(face.plashkaColor, "#ff0000")

        useTheme(face, "South")
        compare(face.themeColor, "#202020")
        compare(face.plashkaColor, "#ff0000")
    }

    function test_empty_ring_plashka_leaves_after_about_two_seconds() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        face.showPlashka(Face.PlashkaEvent.NothingToSave)
        compare(face.visible, true)
        wait(1500)
        compare(face.visible, true)
        tryCompare(face, "visible", false, 1200)
    }

    function test_empty_ring_plashka_leaves_focus_in_the_game() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        face.showPlashka(Face.PlashkaEvent.NothingToSave)
        compare(face.visible, true)
        verify(face.flags & Qt.WindowDoesNotAcceptFocus)
        ignoreWarning(/requestActivate\(\) called for .*WindowDoesNotAcceptFocus set\./)
        face.requestActivate()
        compare(face.active, false)
    }

    function test_empty_ring_plashka_stays_english_until_the_language_changes() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        face.showPlashka(Face.PlashkaEvent.NothingToSave)
        compare(face.plashkaText, "Nothing to save")
        face.setLanguage("ru")
        compare(face.plashkaText, "Сохранять нечего")
    }
}
