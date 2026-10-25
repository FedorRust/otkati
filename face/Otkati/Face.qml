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
        NothingToSave
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
    property string themeStorePath: ""
    property string themeName: ""
    property var themeColors: ({})
    property var settingsSurface: null
    readonly property color settingsColor: settingsSurface ? settingsSurface.color : "transparent"
    readonly property bool settingsVisible: settingsSurface ? settingsSurface.visible : false
    readonly property alias plashkaText: label.text
    readonly property alias plashkaColor: background.color

    function setTheme(color) {
        themeColor = color
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

    function applyTheme(name, colors) {
        if (!settingsSurface)
            settingsSurface = settingsWindowComponent.createObject(null)
        themeName = name
        themeColors = colors
        settingsSurface.color = colors.base
        settingsSurface.visible = true
    }

    function openSettings() {
        let settings
        try {
            settings = JSON.parse(Files.readText(shellSettingsPath))
        } catch (e) {
            return
        }
        const theme = settings.theme || {}
        if (theme.matugen === true)
            return
        const colors = colorSet(theme.colors)
        if (!colors)
            return
        applyTheme(theme.activePreset || "", colors)
    }

    Component.onDestruction: {
        if (settingsSurface)
            settingsSurface.destroy()
    }

    function setLanguage(next) {
        language = next
        if (visible)
            label.text = nothingToSaveText()
    }

    function nothingToSaveText() {
        if (language === "ru")
            return "Сохранять нечего"
        return "Nothing to save"
    }

    function showPlashka(event) {
        if (event !== Face.PlashkaEvent.NothingToSave)
            return
        label.text = nothingToSaveText()
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
            width: 360
            height: 240
            color: "transparent"
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
