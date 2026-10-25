import QtQuick
import QtCore
import QtTest
import Otkati 1.0

TestCase {
    id: testCase
    name: "FirstLaunch"
    property int storeSerial: 0

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

    function freshStore() {
        storeSerial += 1
        let url = StandardPaths.writableLocation(StandardPaths.TempLocation).toString()
        if (url.startsWith("file://"))
            url = url.slice("file://".length)
        return url + "/otkati-theme-" + storeSerial + "-" + Date.now() + ".json"
    }

    function faceOn(shellFile, store) {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        face.cataloguePath = filePath("themes")
        face.shellSettingsPath = filePath(shellFile)
        face.matugenColorsPath = filePath("matugen-colors.json")
        face.themeStorePath = store
        face.openSettings()
        return face
    }

    function test_the_first_opening_of_settings_shows_the_overlay() {
        const face = faceOn("shell-north.json", freshStore())

        compare(face.settingsVisible, true)
        compare(face.firstLaunchVisible, true)
    }

    function test_the_next_opening_of_settings_does_not_play_the_overlay() {
        const store = freshStore()
        const face = faceOn("shell-north.json", store)
        compare(face.firstLaunchVisible, true)

        face.continueFirstLaunch()
        compare(face.firstLaunchVisible, false)
        compare(face.settingsVisible, true)

        face.openSettings()
        compare(face.firstLaunchVisible, false)
        compare(face.settingsVisible, true)

        const again = faceOn("shell-north.json", store)
        compare(again.settingsVisible, true)
        compare(again.firstLaunchVisible, false)
    }
}
