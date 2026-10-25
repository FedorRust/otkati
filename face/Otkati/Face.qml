import QtQuick
import QtQuick.Window
import Otkati.Files 1.0

Window {
    id: root
    width: 360
    height: 80
    visible: false
    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus

    enum PlashkaEvent {
        NothingToSave,
        NoSpace,
        ClipDidNotSave,
        Ordinary
    }

    readonly property var colorKeys: [
        "base", "mantle", "crust", "text", "subtext0", "subtext1",
        "surface0", "surface1", "surface2", "overlay0", "overlay1", "overlay2",
        "blue", "sapphire", "peach", "green", "red", "mauve", "pink",
        "yellow", "maroon", "teal"
    ]

    property color themeColor: "#000000"
    property string language: "en"
    property string cataloguePath: ""
    property string shellSettingsPath: ""
    property string matugenColorsPath: ""
    property string themeStorePath: ""
    property string themeName: ""
    property var themeColors: ({})
    property var readyThemes: []
    property var themeByName: ({})
    property var settingsSurface: null
    readonly property color settingsColor: settingsSurface ? settingsSurface.color : "transparent"
    readonly property color settingsTextColor: settingsSurface ? settingsSurface.ink : "transparent"
    readonly property bool settingsVisible: settingsSurface ? settingsSurface.visible : false
    property int shownPlashka: -1
    readonly property alias plashkaText: label.text
    readonly property alias plashkaColor: background.color

    function setTheme(next) {
        if (typeof next === "string" && next.charAt(0) === "#") {
            themeColor = next
            return
        }
        if (!hasReadyTheme(next))
            return
        const data = readJson(themeByName[next])
        const colors = data && data.isMatugen === true
                ? colorsFrom(readJson(matugenColorsPath))
                : colorsFrom(data)
        if (!colors)
            return
        applyTheme(next, colors)
        rememberTheme()
    }

    function hasReadyTheme(name) {
        for (let i = 0; i < readyThemes.length; i++) {
            if (readyThemes[i] === name)
                return true
        }
        return false
    }

    function loadCatalogue() {
        const paths = Files.jsonFiles(cataloguePath)
        const names = []
        const byName = {}
        for (let i = 0; i < paths.length; i++) {
            const data = readJson(paths[i])
            if (!data || typeof data.name !== "string" || data.name.length === 0)
                continue
            names.push(data.name)
            byName[data.name] = paths[i]
        }
        names.sort()
        readyThemes = names
        themeByName = byName
    }

    function colorSet(source) {
        if (!source)
            return null
        const colors = {}
        for (let i = 0; i < colorKeys.length; i++) {
            const key = colorKeys[i]
            const value = source[key]
            if (typeof value === "string" && value.length > 0)
                colors[key] = value
        }
        if (!colors.base)
            return null
        return colors
    }

    function readJson(path) {
        try {
            return JSON.parse(Files.readText(path))
        } catch (e) {
            return null
        }
    }

    function colorsFrom(parsed) {
        if (!parsed)
            return null
        return colorSet(parsed.colors || parsed)
    }

    function applyTheme(name, colors) {
        if (!settingsSurface)
            settingsSurface = settingsWindowComponent.createObject(null)
        themeName = name
        themeColors = colors
        settingsSurface.color = colors.base
        settingsSurface.ink = colors.text
        settingsSurface.visible = true
    }

    function storedTheme() {
        const stored = readJson(themeStorePath)
        if (!stored || !stored.name)
            return null
        const colors = colorSet(stored.colors)
        if (!colors)
            return null
        return { name: stored.name, colors: colors }
    }

    function rememberTheme() {
        if (!themeStorePath || !themeName)
            return
        Files.writeText(themeStorePath, JSON.stringify({
            name: themeName,
            colors: themeColors
        }))
    }

    function openSettings() {
        loadCatalogue()
        const stored = storedTheme()
        if (stored) {
            applyTheme(stored.name, stored.colors)
            return
        }
        let settings
        try {
            settings = JSON.parse(Files.readText(shellSettingsPath))
        } catch (e) {
            return
        }
        const theme = settings.theme || {}
        const colors = theme.matugen === true ? colorsFrom(readJson(matugenColorsPath)) : colorSet(theme.colors)
        if (!colors)
            return
        applyTheme(theme.activePreset || "", colors)
        rememberTheme()
    }

    Component.onDestruction: {
        if (settingsSurface)
            settingsSurface.destroy()
    }

    function setLanguage(next) {
        language = next
        if (visible)
            label.text = plashkaTextFor(shownPlashka)
    }

    function nothingToSaveText() {
        if (language === "ru")
            return "Сохранять нечего"
        return "Nothing to save"
    }

    function clipDidNotSaveText() {
        if (language === "ru")
            return "Откат не сохранился"
        return "Couldn't save the clip"
    }

    function plashkaTextFor(event) {
        if (event === Face.PlashkaEvent.NothingToSave)
            return nothingToSaveText()
        if (event === Face.PlashkaEvent.ClipDidNotSave)
            return clipDidNotSaveText()
        return ""
    }

    function showPlashka(event) {
        const alarm = event === Face.PlashkaEvent.NothingToSave
                || event === Face.PlashkaEvent.NoSpace
                || event === Face.PlashkaEvent.ClipDidNotSave
        if (!alarm && event !== Face.PlashkaEvent.Ordinary)
            return
        shownPlashka = event
        label.text = plashkaTextFor(event)
        if (alarm)
            background.color = "#ff0000"
        else if (themeColors.base)
            background.color = themeColors.base
        visible = true
        hideTimer.restart()
    }

    Timer {
        id: hideTimer
        interval: 2000
        onTriggered: root.visible = false
    }

    Component {
        id: settingsWindowComponent
        Window {
            property color ink: "transparent"
            width: 360
            height: 240
            color: "transparent"

            Text {
                anchors.centerIn: parent
                color: ink
            }
        }
    }

    Rectangle {
        id: background
        anchors.fill: parent
        color: "#ff0000"
    }

    Text {
        id: label
        anchors.centerIn: parent
        color: "white"
        text: ""
    }
}
