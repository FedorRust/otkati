import QtQuick
import QtQuick.Window

Window {
    id: root
    width: 360
    height: 80
    visible: false

    enum PlashkaEvent {
        NothingToSave
    }

    readonly property alias plashkaText: label.text

    function showPlashka(event) {
        if (event !== Face.PlashkaEvent.NothingToSave)
            return
        label.text = "Nothing to save"
        visible = true
    }

    Text {
        id: label
        anchors.centerIn: parent
        text: ""
    }
}
