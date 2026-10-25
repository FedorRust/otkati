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

    function test_first_open_takes_matugen_colors_from_the_wallpaper() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        face.cataloguePath = filePath("themes")
        face.shellSettingsPath = filePath("shell-matugen.json")
        face.matugenColorsPath = filePath("matugen-colors.json")
        face.themeStorePath = freshStore()
        face.openSettings()

        compare(face.themeName, "Matugen")
        compare(face.settingsColor, "#112233")
        compare(face.themeColors.base, "#112233")
        compare(face.themeColors.mantle, "#112234")
        compare(face.themeColors.crust, "#112235")
        compare(face.themeColors.text, "#abcabc")
        compare(face.themeColors.subtext0, "#abcabd")
        compare(face.themeColors.subtext1, "#abcabe")
        compare(face.themeColors.surface0, "#223344")
        compare(face.themeColors.surface1, "#223345")
        compare(face.themeColors.surface2, "#223346")
        compare(face.themeColors.overlay0, "#334455")
        compare(face.themeColors.overlay1, "#334456")
        compare(face.themeColors.overlay2, "#334457")
        compare(face.themeColors.blue, "#445566")
        compare(face.themeColors.sapphire, "#445567")
        compare(face.themeColors.peach, "#556677")
        compare(face.themeColors.green, "#556678")
        compare(face.themeColors.red, "#667788")
        compare(face.themeColors.mauve, "#667789")
        compare(face.themeColors.pink, "#778899")
        compare(face.themeColors.yellow, "#77889a")
        compare(face.themeColors.maroon, "#8899aa")
        compare(face.themeColors.teal, "#8899ab")
    }
}
