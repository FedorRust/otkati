import QtQuick
import QtCore
import QtTest
import Otkati 1.0

TestCase {
    id: testCase
    name: "Theme"
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

    function openedFace() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        face.cataloguePath = filePath("themes")
        face.shellSettingsPath = filePath("shell-north.json")
        face.themeStorePath = freshStore()
        face.openSettings()
        return face
    }

    function test_first_open_takes_the_theme_serpantinum_has_on_now() {
        const face = openedFace()

        compare(face.settingsVisible, true)
        compare(face.themeName, "North")
        compare(face.settingsColor, "#101418")
        compare(face.themeColors.base, "#101418")
        compare(face.themeColors.mantle, "#181c22")
        compare(face.themeColors.crust, "#0c1014")
        compare(face.themeColors.text, "#e6edf3")
        compare(face.themeColors.subtext0, "#9aa4b2")
        compare(face.themeColors.subtext1, "#c5ced8")
        compare(face.themeColors.surface0, "#242a32")
        compare(face.themeColors.surface1, "#303844")
        compare(face.themeColors.surface2, "#3c4654")
        compare(face.themeColors.overlay0, "#6a7584")
        compare(face.themeColors.overlay1, "#7d8896")
        compare(face.themeColors.overlay2, "#909aaa")
        compare(face.themeColors.blue, "#7eb6ff")
        compare(face.themeColors.sapphire, "#5ec8e6")
        compare(face.themeColors.peach, "#ffb07a")
        compare(face.themeColors.green, "#8fd69a")
        compare(face.themeColors.red, "#ff8b9a")
        compare(face.themeColors.mauve, "#c4a6ff")
        compare(face.themeColors.pink, "#ffb3d9")
        compare(face.themeColors.yellow, "#ffe08a")
        compare(face.themeColors.maroon, "#e08090")
        compare(face.themeColors.teal, "#7ad7c8")
    }
}
