import QtQuick
import QtQuick.Window

Window {
    id: root
    width: 360
    height: 80
    visible: false
    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus

    enum PlashkaEvent {
        NothingToSave,
        NoAccess
    }

    property color themeColor: "#000000"
    property string language: "en"
    readonly property alias plashkaText: label.text
    readonly property alias plashkaColor: background.color
    readonly property int plashkaEvent: plashka.event

    QtObject {
        id: plashka
        property int event: -1
        property bool alarm: false
    }

    function setTheme(color) {
        themeColor = color
    }

    function setLanguage(next) {
        language = next
        if (visible && plashka.event === Face.PlashkaEvent.NothingToSave)
            label.text = nothingToSaveText()
    }

    function nothingToSaveText() {
        if (language === "ru")
            return "Сохранять нечего"
        return "Nothing to save"
    }

    function showPlashka(event) {
        if (event === Face.PlashkaEvent.NothingToSave) {
            plashka.event = event
            plashka.alarm = true
            label.text = nothingToSaveText()
            visible = true
            hideTimer.restart()
            return
        }
        if (event === Face.PlashkaEvent.NoAccess) {
            plashka.event = event
            plashka.alarm = false
            label.text = ""
            visible = true
            hideTimer.restart()
        }
    }

    Timer {
        id: hideTimer
        interval: 2000
        onTriggered: root.visible = false
    }

    Rectangle {
        id: background
        anchors.fill: parent
        color: plashka.alarm ? "#ff0000" : root.themeColor
    }

    Text {
        id: label
        anchors.centerIn: parent
        color: "white"
        text: ""
    }
}
