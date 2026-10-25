import QtQuick
import QtQuick.Window
import QtCore
import Otkati.Files 1.0

Window {
    id: root
    width: 360
    height: 80
    visible: false
    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus

    enum PlashkaEvent {
        NothingToSave,
        NoSpace,
        ClipDidNotSave,
        Ordinary,
        Saved,
        ClipNotSaved,
        NoAccess,
        FolderNotCreated,
        RingOff
    }

    enum RingPlace {
        Memory,
        Disk
    }

    readonly property var colorKeys: [
        "base", "mantle", "crust", "text", "subtext0", "subtext1",
        "surface0", "surface1", "surface2", "overlay0", "overlay1", "overlay2",
        "blue", "sapphire", "peach", "green", "red", "mauve", "pink",
        "yellow", "maroon", "teal"
    ]

    property color themeColor: "#000000"
    property string language: "en"
    readonly property string homePath: {
        let url = StandardPaths.writableLocation(StandardPaths.HomeLocation).toString()
        if (url.startsWith("file://"))
            url = decodeURIComponent(url.slice("file://".length))
        return url
    }
    property string cataloguePath: homePath + "/.local/share/serpantinum/src/assets/themes"
    property string userCataloguePath: homePath + "/.local/state/serpantinum/themes"
    property string shellSettingsPath: homePath + "/.config/serpantinum/settings.json"
    property string matugenColorsPath: homePath + "/.local/state/serpantinum/qs_matugen_colors.json"
    property string themeStorePath: homePath + "/.local/state/otkati/theme.json"
    property string themeName: ""
    property var themeColors: ({})
    property var readyThemes: []
    property var themeByName: ({})
    property var settingsSurface: null
    readonly property color settingsColor: settingsSurface ? settingsSurface.color : "transparent"
    readonly property color settingsTextColor: settingsSurface ? settingsSurface.ink : "transparent"
    readonly property bool settingsVisible: settingsSurface ? settingsSurface.visible : false
    // The otkat folder the face hands to the ring. ~/Videos/Откаты until the person changes it.
    property string otkatFolder: homePath + "/Videos/Откаты"
    // The bind the face hands to the ring. Alt+Z until the person changes it.
    property string bind: "Alt+Z"
    // Knobs the face can hand the ring. The microphone stays in the clip
    // while the ring is on, so there is no microphone-only knob.
    readonly property var knobNames: [
        "length",
        "resolutionShare",
        "bitrate",
        "encoder",
        "ringPlace",
        "ringMonitor",
        "bind",
        "language",
        "theme",
        "otkatFolder",
        "switch"
    ]
    // Values the face hands the ring until those knobs change.
    property string encoder: "H.264"
    property int sideShare: 100
    property int bitrate: 25
    // The length, in seconds, the face hands to the ring. Ten minutes until the person changes it.
    property int length: 600
    // The ring place the face hands to the ring. Memory until the person chooses disk.
    property int ringPlace: Face.RingPlace.Memory
    // The switch the face hands to the ring. On until the person turns the ring off.
    property bool ringOn: true

    function knob(key) {
        if (key === "ringPlace")
            return ringPlace
        if (key === "encoder")
            return encoder
        if (key === "sideShare")
            return sideShare
        if (key === "bitrate")
            return bitrate
        return undefined
    }
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

    function setTheme(next) {
        if (typeof next === "string" && next.charAt(0) === "#") {
            themeColor = next
            return
        }
        if (!hasReadyTheme(next))
            return
        const data = readJson(themeByName[next])
        const colors = data && data.isMatugen === true
                ? colorsFrom(readJson(matugenColorsPath))
                : colorsFrom(data)
        if (!colors)
            return
        applyTheme(next, colors)
        rememberTheme()
    }

    function hasReadyTheme(name) {
        for (let i = 0; i < readyThemes.length; i++) {
            if (readyThemes[i] === name)
                return true
        }
        return false
    }

    function catalogueFiles() {
        const paths = []
        const system = Files.jsonFiles(cataloguePath)
        const user = Files.jsonFiles(userCataloguePath)
        for (let i = 0; i < system.length; i++)
            paths.push(system[i])
        for (let i = 0; i < user.length; i++)
            paths.push(user[i])
        return paths
    }

    function loadCatalogue() {
        const paths = catalogueFiles()
        const names = []
        const byName = {}
        for (let i = 0; i < paths.length; i++) {
            const data = readJson(paths[i])
            if (!data || typeof data.name !== "string" || data.name.length === 0)
                continue
            names.push(data.name)
            byName[data.name] = paths[i]
        }
        names.sort()
        readyThemes = names
        themeByName = byName
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

    function readJson(path) {
        try {
            return JSON.parse(Files.readText(path))
        } catch (e) {
            return null
        }
    }

    function colorsFrom(parsed) {
        if (!parsed)
            return null
        return colorSet(parsed.colors || parsed)
    }

    function applyTheme(name, colors) {
        if (!settingsSurface)
            settingsSurface = settingsWindowComponent.createObject(null)
        themeName = name
        themeColors = colors
        themeColor = colors.base
        settingsSurface.color = colors.base
        settingsSurface.ink = colors.text
        settingsSurface.visible = true
    }

    function storedTheme() {
        const stored = readJson(themeStorePath)
        if (!stored || !stored.name)
            return null
        const colors = colorSet(stored.colors)
        if (!colors)
            return null
        return { name: stored.name, colors: colors }
    }

    function rememberTheme() {
        if (!themeStorePath || !themeName)
            return
        Files.writeText(themeStorePath, JSON.stringify({
            name: themeName,
            colors: themeColors
        }))
    }

    function openSettings() {
        loadCatalogue()
        const stored = storedTheme()
        if (stored) {
            applyTheme(stored.name, stored.colors)
            return
        }
        let settings
        try {
            settings = JSON.parse(Files.readText(shellSettingsPath))
        } catch (e) {
            return
        }
        const theme = settings.theme || {}
        const matugen = isMatugen(theme)
        const colors = matugen ? colorsFrom(readJson(matugenColorsPath)) : colorSet(theme.colors)
        if (!colors)
            return
        applyTheme(matugen ? (theme.activePreset || "Matugen") : (theme.activePreset || ""), colors)
        rememberTheme()
    }

    function isMatugen(theme) {
        if (theme.matugen !== undefined)
            return theme.matugen === true
        return theme.theme === "matugen"
                || theme.name === "matugen"
                || theme.currentTheme === "matugen"
                || theme.current === "matugen"
                || theme.activePreset === "Matugen"
                || theme.activePreset === undefined
    }

    Component.onDestruction: {
        if (settingsSurface)
            settingsSurface.destroy()
    }

    function setLanguage(next) {
        language = next
        if (!visible)
            return
        const text = plashkaPhrase(plashka.event)
        if (text !== "")
            label.text = text
    }

    function setBind(chord) {
        bind = chord
    }

    function setLength(seconds) {
        if (!Number.isInteger(seconds) || seconds < 15 || seconds > 1200)
            return
        length = seconds
    }

    function setRingPlace(place) {
        ringPlace = place
    }

    function setOtkatFolder(path) {
        otkatFolder = path
    }

    function setSwitch(gathering) {
        ringOn = gathering
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
        if (event === Face.PlashkaEvent.ClipNotSaved || event === Face.PlashkaEvent.ClipDidNotSave)
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

    function alarmEvent(event) {
        return event === Face.PlashkaEvent.NothingToSave
                || event === Face.PlashkaEvent.NoSpace
                || event === Face.PlashkaEvent.ClipDidNotSave
                || event === Face.PlashkaEvent.ClipNotSaved
    }

    function ordinaryEvent(event) {
        return event === Face.PlashkaEvent.Ordinary
                || event === Face.PlashkaEvent.Saved
                || event === Face.PlashkaEvent.NoAccess
                || event === Face.PlashkaEvent.FolderNotCreated
                || event === Face.PlashkaEvent.RingOff
    }

    function showPlashka(event) {
        if (!alarmEvent(event) && !ordinaryEvent(event))
            return
        plashka.event = event
        plashka.alarm = alarmEvent(event)
        label.text = plashkaPhrase(event)
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
            property color ink: "transparent"
            width: 360
            height: 240
            color: "transparent"

            Text {
                anchors.centerIn: parent
                color: ink
            }
        }
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
