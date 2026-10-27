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

    function test_the_first_opening_shows_settings_and_no_overlay() {
        const face = faceOn("shell-north.json", freshStore())

        compare(face.settingsVisible, true)
        compare(face.firstLaunchVisible, false)
        compare(face.settingLabel("length"), "Length")
    }

    function test_the_next_opening_still_has_no_overlay() {
        const store = freshStore()
        const face = faceOn("shell-north.json", store)
        compare(face.firstLaunchVisible, false)
        compare(face.settingsVisible, true)

        face.openSettings()
        compare(face.firstLaunchVisible, false)
        compare(face.settingsVisible, true)

        const again = faceOn("shell-north.json", store)
        compare(again.settingsVisible, true)
        compare(again.firstLaunchVisible, false)
    }

    function test_the_knobs_work_on_the_first_opening() {
        const face = faceOn("shell-north.json", freshStore())
        compare(face.firstLaunchVisible, false)
        compare(face.settingsVisible, true)
        face.setLength(45)
        face.setResolutionShare(70)
        face.setBitrate(32)
        compare(face.length, 45)
        compare(face.resolutionShare, 70)
        compare(face.bitrate, 32)

        face.chooseLanguage("ru")
        compare(face.settingLabel("length"), "Длина")
        compare(face.settingLabel("resolutionShare"), "Доля разрешения")
        compare(face.settingLabel("bitrate"), "Битрейт")
    }
}
