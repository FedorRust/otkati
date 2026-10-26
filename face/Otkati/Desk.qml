import QtQuick
import QtQuick.Layouts
import QtQuick.Controls

Item {
    id: root
    property var host: null
    property color ink: "#e1e4da"
    property string page: "ring"
    property string query: ""
    property bool searchOpen: false

    function tone(key, fallback) {
        if (!host || !host.themeColors)
            return fallback
        const value = host.themeColors[key]
        return (typeof value === "string" && value.length > 0) ? value : fallback
    }

    function icon(name) {
        const points = {
            search: 0xF0349,
            ring: 0xF0459,
            picture: 0xF02E9,
            monitor: 0xF0379,
            look: 0xF03D8,
            folder: 0xF024B,
            power: 0xF0425,
            timer: 0xF0954,
            memory: 0xF035B,
            disk: 0xF02CA,
            language: 0xF05CA,
            theme: 0xF03D8,
            tune: 0xF062E,
            gauge: 0xF04C5,
            video: 0xF0567
        }
        return String.fromCodePoint(points[name] || 0xF0493)
    }

    readonly property color accent: tone("mauve", tone("blue", "#a5d396"))
    readonly property color muted: tone("subtext0", ink)
    readonly property color card: tone("surface0", "#1d211b")
    readonly property color row: tone("surface1", "#272b25")
    readonly property color line: tone("surface2", "#32362f")
    readonly property color crust: tone("crust", "#11140f")
    readonly property color base: tone("base", "#0b0f0a")
    readonly property string faceFont: "Adwaita Mono"
    readonly property string iconFont: nerdFont.status === FontLoader.Ready ? nerdFont.name : faceFont

    FontLoader {
        id: nerdFont
        source: host ? "file://" + host.homePath + "/.local/share/serpantinum/src/assets/fonts/SymbolsNerdFontMono-Regular.ttf" : ""
    }

    readonly property var pages: {
        if (!host)
            return []
        const w = host.words
        return [
            { id: "ring", icon: "ring", title: w.navRing, blob: [w.navRing, w.switch, w.length, w.ringPlace, w.memory, w.disk].join("\n") },
            { id: "picture", icon: "picture", title: w.navPicture, blob: [w.navPicture, w.resolutionShare, w.bitrate, w.encoder, "H.264", "HEVC", "AV1"].join("\n") },
            { id: "monitor", icon: "monitor", title: w.navMonitor, blob: [w.navMonitor, w.ringMonitor].join("\n") },
            { id: "look", icon: "look", title: w.navLook, blob: [w.navLook, w.language, w.theme, w.english, w.russian].join("\n") },
            { id: "folder", icon: "folder", title: w.navFolder, blob: [w.navFolder, w.otkatFolder, w.bind].join("\n") }
        ]
    }

    readonly property var shownPages: {
        const q = query.trim().toLowerCase()
        if (!q)
            return pages
        const found = []
        for (let i = 0; i < pages.length; i++) {
            if (pages[i].blob.toLowerCase().indexOf(q) !== -1)
                found.push(pages[i])
        }
        return found
    }

    readonly property bool queryMiss: query.trim().length > 0 && shownPages.length === 0
    readonly property int activeIndex: {
        for (let i = 0; i < shownPages.length; i++) {
            if (shownPages[i].id === page)
                return i
        }
        return 0
    }

    readonly property var monitorModel: {
        if (!host)
            return []
        const listed = host.monitors
        const source = (listed && listed.length) ? listed : Qt.application.screens
        const rows = []
        for (let i = 0; i < source.length; i++) {
            const mon = source[i]
            rows.push({
                name: mon.name || "",
                width: mon.width || 0,
                height: mon.height || 0,
                refresh: Math.round(mon.refreshRate || mon.refresh || 0)
            })
        }
        return rows
    }

    onShownPagesChanged: keepPage()

    function keepPage() {
        for (let i = 0; i < shownPages.length; i++) {
            if (shownPages[i].id === page)
                return
        }
        if (shownPages.length > 0)
            page = shownPages[0].id
    }

    function lengthText(seconds) {
        if (!host)
            return ""
        const w = host.words
        const whole = Math.round(seconds)
        const minutes = Math.floor(whole / 60)
        const rest = whole % 60
        if (rest === 0)
            return minutes + " " + w.min
        if (minutes === 0)
            return rest + " " + w.sec
        return minutes + " " + w.min + " " + rest + " " + w.sec
    }

    function parseLength(text) {
        const raw = String(text || "").trim().toLowerCase()
        if (!raw)
            return null
        const clock = raw.match(/^(\d+)\s*:\s*(\d{1,2})$/)
        if (clock) {
            const seconds = parseInt(clock[2], 10)
            if (seconds >= 60)
                return null
            return acceptLength(parseInt(clock[1], 10) * 60 + seconds)
        }
        const both = raw.match(/^(\d+)\s*(?:min|mins|m|мин|минута|минуты|минут)\s+(\d+)\s*(?:s|sec|secs|с|сек|секунд|секунды|секунду)?$/)
        if (both)
            return acceptLength(parseInt(both[1], 10) * 60 + parseInt(both[2], 10))
        const asMinutes = raw.match(/^(\d+)\s*(?:min|mins|m|мин|минута|минуты|минут)$/)
        if (asMinutes)
            return acceptLength(parseInt(asMinutes[1], 10) * 60)
        const asSeconds = raw.match(/^(\d+)\s*(?:s|sec|secs|с|сек|секунд|секунды|секунду)$/)
        if (asSeconds)
            return acceptLength(parseInt(asSeconds[1], 10))
        const bare = raw.match(/^(\d+)$/)
        if (!bare)
            return null
        const number = parseInt(bare[1], 10)
        if (number <= 20)
            return acceptLength(number * 60)
        return acceptLength(number)
    }

    function acceptLength(seconds) {
        if (!Number.isInteger(seconds) || seconds < 15 || seconds > 1200)
            return null
        return seconds
    }

    function monitorAside(mon) {
        if (!mon || !mon.width)
            return ""
        let text = mon.width + "×" + mon.height
        if (mon.refresh)
            text += "  •  " + mon.refresh + "Hz"
        return text
    }

    component Switcher: Item {
        id: sw
        property bool checked: false
        implicitWidth: 44
        implicitHeight: 24
        Rectangle {
            anchors.fill: parent
            radius: height / 2
            color: sw.checked ? root.accent : root.base
            border.width: sw.checked ? 0 : 1
            border.color: root.line
            Behavior on color { ColorAnimation { duration: 180 } }
            Rectangle {
                width: 18
                height: 18
                radius: 9
                anchors.verticalCenter: parent.verticalCenter
                x: sw.checked ? parent.width - width - 3 : 3
                color: sw.checked ? root.crust : root.ink
                Behavior on x { NumberAnimation { duration: 200; easing.type: Easing.OutCubic } }
                Behavior on color { ColorAnimation { duration: 180 } }
            }
        }
    }

    component Mark: Item {
        property bool chosen: false
        implicitWidth: 18
        implicitHeight: 18
        Rectangle {
            anchors.fill: parent
            radius: 9
            color: "transparent"
            border.width: 2
            border.color: parent.chosen ? root.accent : root.muted
            Rectangle {
                anchors.centerIn: parent
                width: 8
                height: 8
                radius: 4
                color: root.accent
                visible: parent.parent.chosen
            }
        }
    }

    component Meter: Item {
        id: meter
        property real from: 0
        property real to: 100
        property real step: 1
        property real value: 0
        signal committed(real next)
        implicitWidth: 220
        implicitHeight: 22
        property bool dragging: false
        property real dragValue: 0

        readonly property real shown: dragging ? dragValue : value
        readonly property real pos: Math.max(0, Math.min(1, (shown - from) / Math.max(0.0001, to - from)))

        function snap(x) {
            const span = Math.max(1, width)
            const t = Math.max(0, Math.min(1, x / span))
            const raw = from + t * (to - from)
            const steps = Math.round((raw - from) / step)
            return Math.max(from, Math.min(to, from + steps * step))
        }

        Rectangle {
            anchors.verticalCenter: parent.verticalCenter
            width: parent.width
            height: 10
            radius: 5
            color: root.base
            Rectangle {
                width: Math.max(height, parent.width * meter.pos)
                height: parent.height
                radius: height / 2
                color: root.accent
            }
        }

        MouseArea {
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onPressed: function(mouse) {
                meter.dragging = true
                meter.dragValue = meter.snap(mouse.x)
                meter.committed(meter.dragValue)
            }
            onPositionChanged: function(mouse) {
                if (!meter.dragging)
                    return
                const next = meter.snap(mouse.x)
                if (next === meter.dragValue)
                    return
                meter.dragValue = next
                meter.committed(next)
            }
            onReleased: meter.dragging = false
            onCanceled: meter.dragging = false
        }
    }

    component LengthEntry: Item {
        id: entry
        property bool editing: false
        implicitWidth: editing ? 148 : shown.implicitWidth
        implicitHeight: 28

        function begin() {
            field.text = root.host ? root.lengthText(root.host.length) : ""
            editing = true
            Qt.callLater(function() {
                field.forceActiveFocus()
                field.selectAll()
            })
        }

        function commit() {
            if (!editing)
                return
            editing = false
            const seconds = root.parseLength(field.text)
            if (seconds !== null)
                root.host.setLength(seconds)
        }

        Text {
            id: shown
            visible: !entry.editing
            anchors.verticalCenter: parent.verticalCenter
            text: root.host ? root.lengthText(root.host.length) : ""
            color: root.muted
            font.family: root.faceFont
            font.pixelSize: 12
            MouseArea {
                anchors.fill: parent
                cursorShape: Qt.IBeamCursor
                onClicked: entry.begin()
            }
        }

        TextField {
            id: field
            visible: entry.editing
            width: 148
            height: 28
            anchors.verticalCenter: parent.verticalCenter
            color: root.ink
            font.family: root.faceFont
            font.pixelSize: 12
            horizontalAlignment: Text.AlignHCenter
            leftPadding: 8
            rightPadding: 8
            selectByMouse: true
            background: Rectangle {
                radius: 8
                color: root.base
                border.width: 1
                border.color: field.activeFocus ? root.accent : root.line
            }
            onEditingFinished: entry.commit()
            Keys.onEscapePressed: function(event) {
                entry.editing = false
                event.accepted = true
            }
        }
    }

    component PlacePick: Rectangle {
        id: place
        readonly property bool disk: root.host && root.host.ringPlace === 1
        implicitWidth: placeRow.implicitWidth
        implicitHeight: 32
        radius: 10
        color: root.base
        clip: true

        Rectangle {
            id: placePill
            height: 32
            radius: 10
            color: root.accent
            x: place.disk ? memoryCell.width : 0
            width: place.disk ? diskCell.width : memoryCell.width
            Behavior on x { NumberAnimation { duration: 240; easing.type: Easing.OutQuint } }
            Behavior on width { NumberAnimation { duration: 240; easing.type: Easing.OutQuint } }
        }

        Row {
            id: placeRow
            z: 1
            Rectangle {
                id: memoryCell
                width: memoryLabel.implicitWidth + 28
                height: 32
                color: "transparent"
                Text {
                    id: memoryLabel
                    anchors.centerIn: parent
                    text: root.host ? root.host.words.memory : ""
                    color: place.disk ? root.muted : root.crust
                    font.family: root.faceFont
                    font.pixelSize: 12
                    font.weight: place.disk ? Font.Medium : Font.Bold
                    Behavior on color { ColorAnimation { duration: 180 } }
                }
                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.host.setRingPlace(0)
                }
            }
            Rectangle {
                id: diskCell
                width: diskLabel.implicitWidth + 28
                height: 32
                color: "transparent"
                Text {
                    id: diskLabel
                    anchors.centerIn: parent
                    text: root.host ? root.host.words.disk : ""
                    color: place.disk ? root.crust : root.muted
                    font.family: root.faceFont
                    font.pixelSize: 12
                    font.weight: place.disk ? Font.Bold : Font.Medium
                    Behavior on color { ColorAnimation { duration: 180 } }
                }
                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.host.setRingPlace(1)
                }
            }
        }
    }

    component SettingRow: Rectangle {
        id: rowItem
        property string title: ""
        property string hint: ""
        property string glyph: ""
        property string valueText: ""
        property bool clickable: false
        signal activated()
        default property alias extra: slot.data
        width: parent ? parent.width : implicitWidth
        implicitHeight: Math.max(58, body.implicitHeight + 16)
        radius: 12
        color: rowPress.containsMouse && clickable ? Qt.lighter(root.row, 1.12) : root.row
        Behavior on color { ColorAnimation { duration: 140 } }

        RowLayout {
            id: body
            anchors.fill: parent
            anchors.leftMargin: 8
            anchors.rightMargin: 12
            anchors.topMargin: 8
            anchors.bottomMargin: 8
            spacing: 12

            Rectangle {
                Layout.preferredWidth: 32
                Layout.preferredHeight: 32
                Layout.alignment: Qt.AlignVCenter
                radius: 10
                color: root.base
                Text {
                    anchors.centerIn: parent
                    text: rowItem.glyph
                    font.family: root.iconFont
                    font.pixelSize: 16
                    color: root.ink
                }
            }

            ColumnLayout {
                Layout.fillWidth: true
                Layout.alignment: Qt.AlignVCenter
                spacing: 2
                Text {
                    text: rowItem.title
                    color: root.ink
                    font.family: root.faceFont
                    font.pixelSize: 13
                    font.weight: Font.Medium
                    Layout.fillWidth: true
                    elide: Text.ElideRight
                }
                Text {
                    text: rowItem.hint
                    visible: text !== ""
                    color: root.muted
                    font.family: root.faceFont
                    font.pixelSize: 11
                    wrapMode: Text.WordWrap
                    Layout.fillWidth: true
                }
            }

            Text {
                visible: rowItem.valueText !== ""
                text: rowItem.valueText
                color: root.muted
                font.family: root.faceFont
                font.pixelSize: 12
                Layout.alignment: Qt.AlignVCenter
            }

            Item {
                id: slot
                Layout.alignment: Qt.AlignVCenter
                implicitWidth: childrenRect.width
                implicitHeight: Math.max(24, childrenRect.height)
            }
        }

        MouseArea {
            id: rowPress
            anchors.fill: parent
            enabled: rowItem.clickable
            hoverEnabled: rowItem.clickable
            cursorShape: rowItem.clickable ? Qt.PointingHandCursor : Qt.ArrowCursor
            onClicked: rowItem.activated()
        }
    }

    component Card: Rectangle {
        id: card
        property string heading: ""
        property string aside: ""
        default property alias content: rows.data
        width: parent ? parent.width : implicitWidth
        implicitHeight: inner.implicitHeight + 24
        radius: 16
        color: root.card
        border.width: 1
        border.color: Qt.rgba(root.line.r, root.line.g, root.line.b, 0.55)

        Column {
            id: inner
            x: 12
            y: 12
            width: parent.width - 24
            spacing: 6

            Item {
                width: parent.width
                height: card.heading === "" ? 0 : 28
                visible: card.heading !== ""
                Text {
                    text: card.heading
                    color: root.ink
                    font.family: root.faceFont
                    font.pixelSize: 16
                    font.bold: true
                    anchors.verticalCenter: parent.verticalCenter
                    width: parent.width - asideText.implicitWidth - 12
                    elide: Text.ElideRight
                }
                Text {
                    id: asideText
                    text: card.aside
                    color: root.muted
                    font.family: root.faceFont
                    font.pixelSize: 12
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                }
            }

            Column {
                id: rows
                width: parent.width
                spacing: 6
            }
        }
    }

    Rectangle {
        id: sidebar
        width: 260
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        color: "transparent"

        Column {
            anchors.fill: parent
            anchors.margins: 16
            spacing: 10

            Item {
                width: parent.width
                height: 36

                Text {
                    visible: !root.searchOpen
                    anchors.verticalCenter: parent.verticalCenter
                    text: root.host ? root.host.words.settingsTitle : ""
                    color: root.ink
                    font.family: root.faceFont
                    font.pixelSize: 15
                    font.bold: true
                }

                Rectangle {
                    visible: !root.searchOpen
                    anchors.right: parent.right
                    width: 36
                    height: 36
                    radius: 10
                    color: root.card
                    Text {
                        anchors.centerIn: parent
                        text: root.icon("search")
                        font.family: root.iconFont
                        font.pixelSize: 16
                        color: root.muted
                    }
                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            root.searchOpen = true
                            Qt.callLater(function() { searchField.forceActiveFocus() })
                        }
                    }
                }

                TextField {
                    id: searchField
                    visible: root.searchOpen
                    anchors.fill: parent
                    placeholderText: root.host ? root.host.words.search : ""
                    color: root.ink
                    placeholderTextColor: root.muted
                    font.family: root.faceFont
                    font.pixelSize: 13
                    leftPadding: 12
                    rightPadding: 12
                    selectByMouse: true
                    background: Rectangle {
                        radius: 10
                        color: root.card
                        border.width: 1
                        border.color: searchField.activeFocus ? root.accent : root.line
                    }
                    onTextChanged: root.query = text
                    Keys.onEscapePressed: function(event) {
                        root.searchOpen = false
                        root.query = ""
                        text = ""
                        event.accepted = true
                    }
                }
            }

            Item {
                width: parent.width
                height: parent.height - 36 - 10
                clip: true

                Rectangle {
                    z: 0
                    width: navCol.width
                    height: 44
                    radius: 12
                    color: root.accent
                    visible: root.shownPages.length > 0
                    y: root.activeIndex * 48
                    Behavior on y { NumberAnimation { duration: 250; easing.type: Easing.OutQuint } }
                }

                Column {
                    id: navCol
                    width: parent.width
                    spacing: 4
                    z: 1

                    Repeater {
                        model: root.shownPages
                        delegate: Item {
                            required property var modelData
                            required property int index
                            width: navCol.width
                            height: 44
                            z: 1

                            Row {
                                anchors.fill: parent
                                anchors.leftMargin: 6
                                anchors.rightMargin: 10
                                spacing: 10

                                Rectangle {
                                    width: 32
                                    height: 32
                                    radius: 10
                                    anchors.verticalCenter: parent.verticalCenter
                                    color: root.card
                                    Text {
                                        anchors.centerIn: parent
                                        text: root.icon(modelData.icon)
                                        font.family: root.iconFont
                                        font.pixelSize: 16
                                        color: root.ink
                                    }
                                }

                                Text {
                                    width: parent.width - 52
                                    anchors.verticalCenter: parent.verticalCenter
                                    text: modelData.title
                                    elide: Text.ElideRight
                                    font.family: root.faceFont
                                    font.pixelSize: 13
                                    font.weight: modelData.id === root.page ? Font.Bold : Font.Medium
                                    color: modelData.id === root.page ? root.crust : (navPress.containsMouse ? root.ink : root.muted)
                                }
                            }

                            MouseArea {
                                id: navPress
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: root.page = modelData.id
                            }
                        }
                    }
                }

                Text {
                    visible: root.queryMiss
                    anchors.top: parent.top
                    anchors.topMargin: 12
                    width: parent.width
                    wrapMode: Text.WordWrap
                    horizontalAlignment: Text.AlignHCenter
                    text: root.host ? root.host.words.nothingFound : ""
                    color: root.muted
                    font.family: root.faceFont
                    font.pixelSize: 12
                }
            }
        }
    }

    Rectangle {
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        anchors.left: sidebar.right
        width: 1
        color: Qt.rgba(root.line.r, root.line.g, root.line.b, 0.45)
    }

    Flickable {
        id: scroll
        anchors.left: sidebar.right
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        anchors.margins: 18
        clip: true
        contentWidth: width
        contentHeight: pageStack.implicitHeight
        boundsBehavior: Flickable.StopAtBounds

        ScrollBar.vertical: ScrollBar {
            policy: ScrollBar.AsNeeded
            width: 4
            contentItem: Rectangle {
                radius: 2
                color: root.line
            }
        }

        Column {
            id: pageStack
            width: scroll.width
            spacing: 12

            Text {
                visible: root.queryMiss
                width: parent.width
                horizontalAlignment: Text.AlignHCenter
                topPadding: 28
                text: root.host ? root.host.words.nothingFound : ""
                color: root.muted
                font.family: root.faceFont
                font.pixelSize: 14
            }

            Column {
                width: parent.width
                spacing: 12
                visible: !root.queryMiss && root.page === "ring" && root.host

                Card {
                    SettingRow {
                        glyph: root.icon("power")
                        title: host.words.switch
                        hint: host.words.switchHint
                        clickable: true
                        onActivated: host.setSwitch(!host.ringOn)
                        Switcher { checked: host.ringOn }
                    }
                    SettingRow {
                        glyph: root.icon("timer")
                        title: host.words.length
                        hint: host.words.lengthHint
                        Row {
                            spacing: 14
                            LengthEntry {}
                            Meter {
                                from: 15
                                to: 1200
                                step: 1
                                value: host.length
                                onCommitted: function(next) { host.setLength(Math.round(next)) }
                            }
                        }
                    }
                    SettingRow {
                        glyph: host.ringPlace === 0 ? root.icon("memory") : root.icon("disk")
                        title: host.words.ringPlace
                        hint: host.ringPlace === 0 ? host.words.memoryHint : host.words.diskHint
                        PlacePick {}
                    }
                }
            }

            Column {
                width: parent.width
                spacing: 12
                visible: !root.queryMiss && root.page === "picture" && root.host

                Card {
                    SettingRow {
                        glyph: root.icon("tune")
                        title: host.words.resolutionShare
                        hint: host.words.resolutionHint
                        valueText: host.resolutionShare + "%"
                        Meter {
                            from: 50
                            to: 100
                            step: 10
                            value: host.resolutionShare
                            onCommitted: function(next) { host.setResolutionShare(Math.round(next)) }
                        }
                    }
                    SettingRow {
                        glyph: root.icon("gauge")
                        title: host.words.bitrate
                        hint: host.words.bitrateHint
                        valueText: host.bitrate + " " + host.words.mbit
                        Meter {
                            from: 8
                            to: 80
                            step: 1
                            value: host.bitrate
                            onCommitted: function(next) { host.setBitrate(Math.round(next)) }
                        }
                    }
                }

                Card {
                    heading: host.words.encoder
                    aside: host.words.chooseH264
                    Repeater {
                        model: host.encoderItems
                        delegate: SettingRow {
                            required property string modelData
                            width: parent.width
                            glyph: root.icon("video")
                            title: modelData
                            hint: host.encoderMark(modelData)
                            clickable: true
                            onActivated: host.chooseEncoder(modelData)
                            Mark { chosen: host.encoder === modelData }
                        }
                    }
                }
            }

            Column {
                id: monitorPage
                width: parent.width
                spacing: 12
                visible: !root.queryMiss && root.page === "monitor" && root.host

                Repeater {
                    model: root.monitorModel
                    delegate: Card {
                        required property var modelData
                        width: monitorPage.width
                        heading: modelData.name
                        aside: root.monitorAside(modelData)
                        SettingRow {
                            glyph: root.icon("monitor")
                            title: host.words.ringMonitor
                            hint: host.words.monitorHint
                            clickable: true
                            onActivated: host.setRingMonitor(modelData.name)
                            Switcher { checked: host.ringMonitor === modelData.name }
                        }
                    }
                }
            }

            Column {
                width: parent.width
                spacing: 12
                visible: !root.queryMiss && root.page === "look" && root.host

                Card {
                    heading: host.words.language
                    SettingRow {
                        glyph: root.icon("language")
                        title: host.words.english
                        hint: host.words.languageHint
                        clickable: true
                        onActivated: host.chooseLanguage("en")
                        Mark { chosen: host.language === "en" }
                    }
                    SettingRow {
                        glyph: root.icon("language")
                        title: host.words.russian
                        hint: host.words.languageHint
                        clickable: true
                        onActivated: host.chooseLanguage("ru")
                        Mark { chosen: host.language === "ru" }
                    }
                }

                Card {
                    heading: host.words.theme
                    Repeater {
                        model: host.readyThemes
                        delegate: SettingRow {
                            required property string modelData
                            width: parent.width
                            glyph: root.icon("theme")
                            title: modelData
                            hint: host.words.themeHint
                            clickable: true
                            onActivated: host.setTheme(modelData)
                            Mark { chosen: host.themeName === modelData }
                        }
                    }
                }
            }

            Column {
                width: parent.width
                spacing: 12
                visible: !root.queryMiss && root.page === "folder" && root.host

                Card {
                    SettingRow {
                        glyph: root.icon("folder")
                        title: host.words.otkatFolder
                        hint: host.words.folderHint
                        TextField {
                            id: folderField
                            implicitWidth: 300
                            implicitHeight: 36
                            text: host.otkatFolder
                            color: root.ink
                            font.family: root.faceFont
                            font.pixelSize: 12
                            leftPadding: 12
                            rightPadding: 12
                            selectByMouse: true
                            background: Rectangle {
                                radius: 10
                                color: root.base
                                border.width: 1
                                border.color: folderField.activeFocus ? root.accent : root.line
                            }
                            onEditingFinished: host.setOtkatFolder(text)
                        }
                    }
                    SettingRow {
                        glyph: root.icon("ring")
                        title: host.words.bind
                        hint: host.words.bindHint
                        TextField {
                            id: bindField
                            implicitWidth: 160
                            implicitHeight: 36
                            text: host.bind
                            color: root.ink
                            font.family: root.faceFont
                            font.pixelSize: 13
                            horizontalAlignment: Text.AlignHCenter
                            leftPadding: 12
                            rightPadding: 12
                            selectByMouse: true
                            background: Rectangle {
                                radius: 10
                                color: root.base
                                border.width: 1
                                border.color: bindField.activeFocus ? root.accent : root.line
                            }
                            onEditingFinished: host.setBind(text)
                        }
                    }
                }
            }
        }
    }
}
