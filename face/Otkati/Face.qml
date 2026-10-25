import QtQuick
import QtQuick.Window

Window {
    id: root
    width: 360
    height: 80
    visible: false
    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus

    enum PlashkaEvent {
        NothingToSave
    }

    property color themeColor: "#000000"
    property string language: "en"
    readonly property alias plashkaText: label.text
    readonly property alias plashkaColor: background.color

    function setTheme(color) {
        themeColor = color
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

    function settingLabel(key) {
        if (key === "length")
            return lengthLabel.text
        if (key === "resolutionShare")
            return resolutionShareLabel.text
        if (key === "bitrate")
            return bitrateLabel.text
        if (key === "encoder")
            return encoderLabel.text
        if (key === "ringPlace")
            return ringPlaceLabel.text
        if (key === "ringMonitor")
            return ringMonitorLabel.text
        if (key === "bind")
            return bindLabel.text
        if (key === "language")
            return languageLabel.text
        if (key === "theme")
            return themeLabel.text
        if (key === "otkatFolder")
            return otkatFolderLabel.text
        if (key === "switch")
            return switchLabel.text
        return ""
    }

    Item {
        id: settings
        visible: false

        Text { id: lengthLabel; text: "Length" }
        Text { id: resolutionShareLabel; text: "Resolution share" }
        Text { id: bitrateLabel; text: "Bitrate" }
        Text { id: encoderLabel; text: "Encoder" }
        Text { id: ringPlaceLabel; text: "Ring place" }
        Text { id: ringMonitorLabel; text: "Ring monitor" }
        Text { id: bindLabel; text: "Bind" }
        Text { id: languageLabel; text: "Language" }
        Text { id: themeLabel; text: "Theme" }
        Text { id: otkatFolderLabel; text: "Otkat folder" }
        Text { id: switchLabel; text: "Switch" }
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
