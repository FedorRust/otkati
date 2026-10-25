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
    // The bind the face hands to the ring. Alt+Z until the person changes it.
    property string bind: "Alt+Z"
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

    function setBind(chord) {
        bind = chord
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
