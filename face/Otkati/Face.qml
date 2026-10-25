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
    readonly property alias plashkaText: label.text
    readonly property alias plashkaColor: background.color

    function setTheme(color) {
        themeColor = color
    }

    function showPlashka(event) {
        if (event !== Face.PlashkaEvent.NothingToSave)
            return
        label.text = "Nothing to save"
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
