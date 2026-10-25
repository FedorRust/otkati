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

    function test_the_overlay_covers_only_the_ring_monitor() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        face.cataloguePath = filePath("themes")
        face.shellSettingsPath = filePath("shell-north.json")
        face.matugenColorsPath = filePath("matugen-colors.json")
        face.themeStorePath = freshStore()
        face.monitors = [
            { name: "desk", width: 1920, height: 1080 },
            { name: "side", width: 1280, height: 800 }
        ]
        face.ringMonitor = "desk"
        face.openSettings()

        compare(face.firstLaunchVisible, true)
        compare(face.firstLaunchMonitor, "desk")
        compare(face.overlayCovers("desk"), true)
        compare(face.overlayCovers("side"), false)
        compare(face.firstLaunchFullScreen, true)
        compare(face.firstLaunchWidth, 1920)
        compare(face.firstLaunchHeight, 1080)
    }

    function test_the_overlay_wipes_then_types_four_lines_then_fills_the_penguin() {
        const face = faceOn("shell-north.json", freshStore())
        const lines = [
            "Welcome",
            "The last minutes are already kept.",
            "Alt+Z saves a clip.",
            "FedorRust"
        ]

        compare(face.firstLaunchStep, "wipe")
        verify(face.screenWipe < 1)

        tryCompare(face, "firstLaunchStep", "lines")
        compare(face.screenWipe, 1)
        compare(face.screenWipeWidth, face.firstLaunchWidth)
        compare(face.firstLaunchLines[0], lines[0])
        compare(face.firstLaunchLines[1], lines[1])
        compare(face.firstLaunchLines[2], lines[2])
        compare(face.firstLaunchLines[3], lines[3])
        verify(face.typedLines.join("").length < lines.join("").length)

        tryVerify(function() { return face.typedLines.join("\n") === lines.join("\n") })

        tryCompare(face, "firstLaunchStep", "penguin")
        verify(face.penguinFillLevel < 1)
        verify(face.penguinFillHeight < face.penguinHeight)
        verify(Math.abs((face.penguinFillTop + face.penguinFillHeight) - face.penguinHeight) <= 1)

        tryCompare(face, "penguinFillLevel", 1)
        verify(Math.abs(face.penguinFillHeight - face.penguinHeight) <= 1)
        verify(Math.abs((face.penguinFillTop + face.penguinFillHeight) - face.penguinHeight) <= 1)
    }

    function test_the_lines_and_the_button_follow_the_program_language() {
        const english = [
            "Welcome",
            "The last minutes are already kept.",
            "Alt+Z saves a clip.",
            "FedorRust"
        ]
        const russian = [
            "Добро пожаловать",
            "Последние минуты уже пишутся.",
            "Alt+Z сохраняет откат.",
            "FedorRust"
        ]
        const face = faceOn("shell-north.json", freshStore())

        compare(face.firstLaunchLines[0], english[0])
        compare(face.firstLaunchLines[1], english[1])
        compare(face.firstLaunchLines[2], english[2])
        compare(face.firstLaunchLines[3], english[3])
        compare(face.firstLaunchButton, "Continue")

        face.chooseLanguage("ru")
        compare(face.firstLaunchLines[0], russian[0])
        compare(face.firstLaunchLines[1], russian[1])
        compare(face.firstLaunchLines[2], russian[2])
        compare(face.firstLaunchLines[3], russian[3])
        compare(face.firstLaunchLines[3], english[3])
        compare(face.firstLaunchButton, "Дальше")

        const again = createTemporaryObject(faceComponent, null)
        verify(again)
        again.cataloguePath = filePath("themes")
        again.shellSettingsPath = filePath("shell-north.json")
        again.matugenColorsPath = filePath("matugen-colors.json")
        again.themeStorePath = freshStore()
        again.chooseLanguage("ru")
        again.openSettings()
        compare(again.firstLaunchButton, "Дальше")
        tryVerify(function() { return again.typedLines.join("\n") === russian.join("\n") })
    }

    function test_continue_or_escape_closes_the_overlay_and_leaves_the_knobs() {
        const face = faceOn("shell-north.json", freshStore())
        compare(face.firstLaunchVisible, true)
        face.continueFirstLaunch()
        compare(face.firstLaunchVisible, false)
        compare(face.settingsVisible, true)
        compare(face.settingLabel("length"), "Length")
        compare(face.settingLabel("resolutionShare"), "Resolution share")
        compare(face.settingLabel("bitrate"), "Bitrate")
        face.setLength(45)
        face.setResolutionShare(70)
        face.setBitrate(32)
        compare(face.length, 45)
        compare(face.resolutionShare, 70)
        compare(face.bitrate, 32)

        const escaped = faceOn("shell-north.json", freshStore())
        compare(escaped.firstLaunchVisible, true)
        escaped.escapeFirstLaunch()
        compare(escaped.firstLaunchVisible, false)
        compare(escaped.settingsVisible, true)
        escaped.chooseLanguage("ru")
        compare(escaped.settingLabel("length"), "Длина")
        compare(escaped.settingLabel("resolutionShare"), "Доля разрешения")
        compare(escaped.settingLabel("bitrate"), "Битрейт")
        escaped.setLength(20)
        escaped.setResolutionShare(50)
        escaped.setBitrate(8)
        compare(escaped.length, 20)
        compare(escaped.resolutionShare, 50)
        compare(escaped.bitrate, 8)
    }

    function test_changing_the_theme_recolors_the_penguin() {
        const face = faceOn("shell-north.json", freshStore())
        compare(face.penguinOutlineColor, "#e6edf3")
        compare(face.penguinFillColor, "#7eb6ff")

        face.setTheme("South")
        compare(face.penguinOutlineColor, "#d0d0e0")
        compare(face.penguinFillColor, "#0000bb")
        verify(face.penguinOutlineColor !== "#000000")
        verify(face.penguinFillColor !== "#ffcc00")
        verify(face.penguinOutlineColor !== "#ffcc00")
        verify(face.penguinFillColor !== "#000000")
    }

    function test_the_overlay_does_not_use_the_no_space_alarm() {
        const face = faceOn("shell-north.json", freshStore())
        face.showPlashka(Face.PlashkaEvent.NoSpace)
        compare(face.plashkaColor, "#ff0000")
        compare(face.plashkaEvent, Face.PlashkaEvent.NoSpace)
        compare(face.penguinOutlineColor, "#e6edf3")
        compare(face.penguinFillColor, "#7eb6ff")
        compare(face.screenWipeColor, "#101418")
        compare(face.firstLaunchTextColor, "#e6edf3")
        compare(face.firstLaunchButtonColor, "#e6edf3")
        verify(face.penguinOutlineColor !== face.plashkaColor)
        verify(face.penguinFillColor !== face.plashkaColor)
        verify(face.screenWipeColor !== face.plashkaColor)
        verify(face.firstLaunchTextColor !== face.plashkaColor)
        verify(face.firstLaunchButtonColor !== face.plashkaColor)
    }
}
