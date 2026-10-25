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
        ClipNotSaved,
        NoAccess
    }

    property color themeColor: "#000000"
    property string language: "en"
    // Reported by the shell. The program language does not follow either value.
    property string shellLanguage: ""
    property string serpantinumLanguage: ""
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
        if (!visible)
            return
        const text = plashkaPhrase(plashka.event)
        if (text !== "")
            label.text = text
    }

    function setShellLanguage(next) {
        shellLanguage = next
    }

    function setSerpantinumLanguage(next) {
        serpantinumLanguage = next
    }

    function plashkaPhrase(event) {
        if (event === Face.PlashkaEvent.NothingToSave)
            return language === "ru" ? "Сохранять нечего" : "Nothing to save"
        if (event === Face.PlashkaEvent.ClipNotSaved)
            return language === "ru" ? "Откат не сохранился" : "Couldn't save the clip"
        return ""
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

    function languageChoice(choice) {
        if (choice === "en")
            return englishChoice.text
        if (choice === "ru")
            return russianChoice.text
        return ""
    }

    function chooseLanguage(choice) {
        if (choice === "en" || choice === "ru")
            setLanguage(choice)
    }

    Item {
        id: settings
        visible: false

        Text { id: lengthLabel; text: root.language === "ru" ? "Длина" : "Length" }
        Text { id: resolutionShareLabel; text: root.language === "ru" ? "Доля разрешения" : "Resolution share" }
        Text { id: bitrateLabel; text: root.language === "ru" ? "Битрейт" : "Bitrate" }
        Text { id: encoderLabel; text: root.language === "ru" ? "Кодировщик" : "Encoder" }
        Text { id: ringPlaceLabel; text: root.language === "ru" ? "Место кольца" : "Ring place" }
        Text { id: ringMonitorLabel; text: root.language === "ru" ? "Монитор кольца" : "Ring monitor" }
        Text { id: bindLabel; text: root.language === "ru" ? "Бинд" : "Bind" }
        Text { id: languageLabel; text: root.language === "ru" ? "Язык" : "Language" }
        Text { id: themeLabel; text: root.language === "ru" ? "Тема" : "Theme" }
        Text { id: otkatFolderLabel; text: root.language === "ru" ? "Папка откатов" : "Otkat folder" }
        Text { id: switchLabel; text: root.language === "ru" ? "Выключатель" : "Switch" }
        Text {
            id: englishChoice
            text: root.language === "ru" ? "Английский" : "English"
            MouseArea {
                anchors.fill: parent
                onClicked: root.chooseLanguage("en")
            }
        }
        Text {
            id: russianChoice
            text: root.language === "ru" ? "Русский" : "Russian"
            MouseArea {
                anchors.fill: parent
                onClicked: root.chooseLanguage("ru")
            }
        }
    }

    function showPlashka(event) {
        if (event === Face.PlashkaEvent.NoAccess) {
            plashka.event = event
            plashka.alarm = false
            label.text = ""
            visible = true
            hideTimer.restart()
            return
        }
        const text = plashkaPhrase(event)
        if (text === "")
            return
        plashka.event = event
        plashka.alarm = true
        label.text = text
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
        color: plashka.alarm ? "#ff0000" : root.themeColor
    }

    Text {
        id: label
        anchors.centerIn: parent
        color: "white"
        text: ""
    }
}
