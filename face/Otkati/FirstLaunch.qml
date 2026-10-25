import QtQuick
import QtQuick.Window
import QtQuick.Shapes

Window {
    id: overlay

    property string monitorName: ""
    property int monitorWidth: 0
    property int monitorHeight: 0
    property bool coversMonitor: false

    property color wipeColor: "transparent"
    property color lineColor: "transparent"
    property color outlineColor: "transparent"
    property color fillColor: "transparent"
    property var lines: []

    property string step: ""
    property real wipe: 0
    property real typedCount: 0
    property real fillLevel: 0

    readonly property int lineCharacters: {
        let count = 0
        for (let i = 0; i < lines.length; i++)
            count += lines[i].length
        return count
    }
    readonly property var typedLines: charactersTyped(typedCount, lines)
    readonly property real screenWipeWidth: wipeLayer.width
    readonly property real penguinFillTop: fillWindow.y
    readonly property real penguinFillHeight: fillWindow.height
    readonly property real penguinHeight: penguin.height

    width: 640
    height: 480
    visible: true
    color: "transparent"
    flags: Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint

    function charactersTyped(count, source) {
        const shown = []
        let left = count
        for (let i = 0; i < source.length; i++) {
            const length = source[i].length
            const take = Math.max(0, Math.min(length, Math.floor(left)))
            shown.push(source[i].substring(0, take))
            left -= length
        }
        return shown
    }

    function play() {
        step = "wipe"
        wipe = 0
        typedCount = 0
        fillLevel = 0
        gesture.start()
    }

    Component.onCompleted: play()

    SequentialAnimation {
        id: gesture

        ScriptAction { script: overlay.step = "wipe" }
        NumberAnimation {
            target: overlay
            property: "wipe"
            from: 0
            to: 1
            duration: 200
            easing.type: Easing.InOutCubic
        }
        ScriptAction { script: overlay.step = "lines" }
        NumberAnimation {
            target: overlay
            property: "typedCount"
            from: 0
            to: overlay.lineCharacters
            duration: 480
        }
        ScriptAction { script: overlay.step = "penguin" }
        NumberAnimation {
            target: overlay
            property: "fillLevel"
            from: 0
            to: 1
            duration: 420
            easing.type: Easing.OutCubic
        }
        ScriptAction { script: overlay.step = "ready" }
    }

    Rectangle {
        id: wipeLayer
        anchors.left: parent.left
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        width: parent.width * overlay.wipe
        color: overlay.wipeColor
    }

    Column {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom
        anchors.bottomMargin: 48
        spacing: 8
        visible: overlay.step === "lines" || overlay.step === "penguin" || overlay.step === "ready"

        Repeater {
            model: overlay.typedLines
            Text {
                required property string modelData
                text: modelData
                color: overlay.lineColor
                font.pixelSize: 22
            }
        }
    }

    Item {
        id: penguin
        width: 220
        height: 260
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.verticalCenter: parent.verticalCenter
        anchors.verticalCenterOffset: -24
        opacity: overlay.step === "penguin" || overlay.step === "ready" ? 1 : 0

        Item {
            id: fillWindow
            width: parent.width
            height: parent.height * overlay.fillLevel
            y: parent.height - height
            clip: true

            PenguinShape {
                filled: true
                ink: overlay.fillColor
                width: penguin.width
                height: penguin.height
                y: -fillWindow.y
            }
        }

        PenguinShape {
            anchors.fill: parent
            filled: false
            ink: overlay.outlineColor
        }
    }

    component PenguinShape: Shape {
        id: mark
        property bool filled: false
        property color ink: "transparent"
        preferredRendererType: Shape.CurveRenderer

        ShapePath { strokeColor: mark.filled ? "transparent" : mark.ink; strokeWidth: mark.filled ? 0 : 3; fillColor: mark.filled ? mark.ink : "transparent"; PathSvg { path: "M 110 80 A 58 68 0 0 1 110 216 A 58 68 0 0 1 110 80 Z" } }
        ShapePath { strokeColor: mark.filled ? "transparent" : mark.ink; strokeWidth: mark.filled ? 0 : 3; fillColor: mark.filled ? mark.ink : "transparent"; PathSvg { path: "M 110 30 A 32 32 0 0 1 110 94 A 32 32 0 0 1 110 30 Z" } }
        ShapePath { strokeColor: mark.filled ? "transparent" : mark.ink; strokeWidth: mark.filled ? 0 : 3; fillColor: mark.filled ? mark.ink : "transparent"; PathSvg { path: "M 134 56 L 176 70 L 134 84 Z" } }
        ShapePath { strokeColor: mark.filled ? "transparent" : mark.ink; strokeWidth: mark.filled ? 0 : 3; fillColor: mark.filled ? mark.ink : "transparent"; PathSvg { path: "M 48 116 A 14 34 0 0 1 48 184 A 14 34 0 0 1 48 116 Z" } }
        ShapePath { strokeColor: mark.filled ? "transparent" : mark.ink; strokeWidth: mark.filled ? 0 : 3; fillColor: mark.filled ? mark.ink : "transparent"; PathSvg { path: "M 172 116 A 14 34 0 0 1 172 184 A 14 34 0 0 1 172 116 Z" } }
        ShapePath { strokeColor: mark.filled ? "transparent" : mark.ink; strokeWidth: mark.filled ? 0 : 3; fillColor: mark.filled ? mark.ink : "transparent"; PathSvg { path: "M 82 219 A 22 11 0 0 1 82 241 A 22 11 0 0 1 82 219 Z" } }
        ShapePath { strokeColor: mark.filled ? "transparent" : mark.ink; strokeWidth: mark.filled ? 0 : 3; fillColor: mark.filled ? mark.ink : "transparent"; PathSvg { path: "M 142 219 A 22 11 0 0 1 142 241 A 22 11 0 0 1 142 219 Z" } }
        ShapePath { strokeColor: mark.filled ? "transparent" : mark.ink; strokeWidth: mark.filled ? 0 : 3; fillColor: "transparent"; PathSvg { path: "M 98 53 A 5 5 0 0 1 98 63 A 5 5 0 0 1 98 53 Z" } }
        ShapePath { strokeColor: mark.filled ? "transparent" : mark.ink; strokeWidth: mark.filled ? 0 : 3; fillColor: "transparent"; PathSvg { path: "M 124 53 A 5 5 0 0 1 124 63 A 5 5 0 0 1 124 53 Z" } }
        ShapePath { strokeColor: mark.filled ? "transparent" : mark.ink; strokeWidth: mark.filled ? 0 : 3; fillColor: "transparent"; PathSvg { path: "M 78 120 Q 110 200 142 120" } }
    }
}
