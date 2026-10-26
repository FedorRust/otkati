import QtQuick
import QtQuick.Window
import QtQuick.Effects

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
    property color crustColor: wipeColor
    property color surfaceColor: wipeColor
    property color orbColor: fillColor
    property color mutedColor: lineColor
    property string buttonText: ""
    property var lines: []
    property string assetsDir: ""

    readonly property bool shellLook: assetsDir.length > 0

    signal continued()
    signal escaped()

    property string step: ""
    property real wipe: 0
    property real typedCount: 0
    property real fillLevel: 0
    property real orbit: 0

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
    readonly property real penguinSide: Math.max(280, Math.min(460, height * 0.42))

    title: "otkati-welcome"
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
        overlay.requestActivate()
    }

    Shortcut {
        sequence: "Escape"
        context: Qt.WindowShortcut
        onActivated: overlay.escaped()
    }

    Component.onCompleted: play()

    onClosing: function(close) {
        close.accepted = false
    }

    NumberAnimation on orbit {
        from: 0
        to: Math.PI * 2
        duration: 120000
        loops: Animation.Infinite
        running: overlay.shellLook
    }

    SequentialAnimation {
        id: gesture

        ScriptAction { script: overlay.step = "wipe" }
        NumberAnimation {
            target: overlay
            property: "wipe"
            from: 0
            to: 1
            duration: 700
            easing.type: Easing.InOutCubic
        }
        ScriptAction { script: overlay.step = "lines" }
        NumberAnimation {
            target: overlay
            property: "typedCount"
            from: 0
            to: overlay.lineCharacters
            duration: 1600
            easing.type: Easing.InOutSine
        }
        ScriptAction { script: overlay.step = "penguin" }
        NumberAnimation {
            target: overlay
            property: "fillLevel"
            from: 0
            to: 1
            duration: 1400
            easing.type: Easing.OutCubic
        }
        ScriptAction { script: overlay.step = "ready" }
    }

    Loader {
        anchors.fill: parent
        active: overlay.shellLook
        sourceComponent: shellBackdrop
        z: 1
    }

    Rectangle {
        anchors.fill: parent
        color: overlay.wipeColor
        opacity: overlay.shellLook ? 0 : overlay.wipe
        z: 0
    }

    Rectangle {
        id: wipeLayer
        anchors.left: parent.left
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        width: parent.width * overlay.wipe
        color: "transparent"
        z: 0
    }

    Column {
        anchors.centerIn: parent
        spacing: 14
        opacity: overlay.step === "lines" ? 1 : 0
        z: 3

        Behavior on opacity { NumberAnimation { duration: 280 } }

        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: overlay.typedLines.length > 0 ? overlay.typedLines[0] : ""
            color: overlay.lineColor
            font.pixelSize: 56
            font.bold: true
        }
        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: overlay.typedLines.length > 1 ? overlay.typedLines[1] : ""
            color: overlay.lineColor
            font.pixelSize: 26
            font.bold: true
        }
        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: overlay.typedLines.length > 2 ? overlay.typedLines[2] : ""
            color: overlay.fillColor
            font.pixelSize: 32
            font.bold: true
        }
        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: overlay.typedLines.length > 3 ? overlay.typedLines[3] : ""
            color: overlay.mutedColor
            font.pixelSize: 16
        }
    }

    Item {
        id: penguin
        width: overlay.penguinSide
        height: overlay.penguinSide
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.verticalCenter: parent.verticalCenter
        anchors.verticalCenterOffset: -height * 0.18
        opacity: overlay.step === "penguin" || overlay.step === "ready" ? 1 : 0
        scale: overlay.step === "penguin" || overlay.step === "ready" ? 1 : 0.94
        z: 4

        Behavior on opacity { NumberAnimation { duration: 420 } }
        Behavior on scale { NumberAnimation { duration: 700; easing.type: Easing.OutExpo } }

        TuxTint {
            anchors.fill: parent
            ink: overlay.outlineColor
            stamp: Qt.resolvedUrl("tux-mask.png")
        }

        Item {
            id: fillWindow
            width: parent.width
            height: parent.height * overlay.fillLevel
            y: parent.height - height
            clip: true

            Item {
                width: penguin.width
                height: penguin.height
                y: -fillWindow.y

                TuxTint {
                    anchors.fill: parent
                    ink: overlay.outlineColor
                    stamp: Qt.resolvedUrl("tux-body.png")
                }
                TuxTint {
                    anchors.fill: parent
                    ink: overlay.crustColor
                    stamp: Qt.resolvedUrl("tux-belly.png")
                }
                TuxTint {
                    anchors.fill: parent
                    ink: overlay.fillColor
                    stamp: Qt.resolvedUrl("tux-accent.png")
                }
            }
        }
    }

    component TuxTint: Item {
        property color ink: "transparent"
        property url stamp

        Rectangle {
            id: chip
            anchors.fill: parent
            color: parent.ink
            visible: false
        }

        Image {
            id: stampImage
            anchors.fill: parent
            source: parent.stamp
            fillMode: Image.PreserveAspectFit
            sourceSize: Qt.size(512, 512)
            smooth: true
            antialiasing: true
            visible: false
            layer.enabled: true
            layer.smooth: true
        }

        MultiEffect {
            anchors.fill: parent
            source: chip
            maskEnabled: true
            maskSource: stampImage
            autoPaddingEnabled: false
        }
    }

    Text {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: penguin.bottom
        anchors.topMargin: 18
        text: overlay.typedLines.length > 3 ? overlay.typedLines[3] : ""
        color: overlay.mutedColor
        font.pixelSize: 16
        opacity: overlay.step === "penguin" || overlay.step === "ready" ? 0.85 : 0
        z: 5
    }

    Rectangle {
        id: continueButton
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom
        anchors.bottomMargin: 56
        width: Math.max(240, continueLabel.implicitWidth + 48)
        height: 48
        radius: 10
        color: overlay.surfaceColor
        border.color: overlay.fillColor
        border.width: 1
        opacity: overlay.step === "penguin" || overlay.step === "ready" ? 1 : 0
        z: 6

        Behavior on opacity { NumberAnimation { duration: 420 } }

        Text {
            id: continueLabel
            anchors.centerIn: parent
            text: overlay.buttonText
            color: overlay.lineColor
            font.pixelSize: 16
        }

        MouseArea {
            anchors.fill: parent
            onClicked: overlay.continued()
        }
    }

    Component {
        id: shellBackdrop
        Item {
            ShaderEffect {
                id: bgCanvas
                anchors.fill: parent

                property real phase: 0
                NumberAnimation on phase {
                    loops: Animation.Infinite
                    running: overlay.wipe > 0 && overlay.wipe < 1
                    from: 0
                    to: Math.PI * 2
                    duration: 3600
                }

                property vector2d itemSize: Qt.vector2d(width, height)
                property real reveal: overlay.wipe
                property color color0: overlay.crustColor
                property color color1: overlay.surfaceColor
                property color color2: overlay.fillColor
                property color color3: overlay.orbColor
                property color color4: overlay.wipeColor
                property vector4d params: Qt.vector4d(45, 0, 0, 0)

                fragmentShader: "file://" + overlay.assetsDir + "/shaders/effects/screen_wipe.frag.qsb"
            }

            Item {
                width: 900
                height: 900
                x: parent.width / 2 - width / 2 + Math.cos(overlay.orbit * 2) * 280
                y: parent.height / 2 - height / 2 + Math.sin(overlay.orbit * 2) * 160
                opacity: overlay.wipe * 0.16

                Rectangle {
                    id: orbA
                    anchors.fill: parent
                    radius: width / 2
                    color: overlay.orbColor
                    visible: false
                }
                MultiEffect {
                    anchors.fill: parent
                    source: orbA
                    blurEnabled: true
                    blur: 0.8
                    blurMax: 32
                }
            }

            Item {
                width: 1000
                height: 1000
                x: parent.width / 2 - width / 2 + Math.sin(overlay.orbit * 1.5) * -280
                y: parent.height / 2 - height / 2 + Math.cos(overlay.orbit * 1.5) * -160
                opacity: overlay.wipe * 0.14

                Rectangle {
                    id: orbB
                    anchors.fill: parent
                    radius: width / 2
                    color: overlay.fillColor
                    visible: false
                }
                MultiEffect {
                    anchors.fill: parent
                    source: orbB
                    blurEnabled: true
                    blur: 0.8
                    blurMax: 32
                }
            }

            ShaderEffect {
                anchors.fill: parent
                property vector2d itemSize: Qt.vector2d(width, height)
                property color vignetteColor: "#000000"
                property vector4d params: Qt.vector4d(0.4, 0.75, 0.4, 0)
                fragmentShader: "file://" + overlay.assetsDir + "/shaders/effects/vignette.frag.qsb"
            }
        }
    }

}
