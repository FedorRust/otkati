import QtQuick
import QtQuick.Window
import QtCore
import Otkati.Files 1.0

Window {
    id: root
    width: 480
    height: 88
    minimumWidth: 480
    maximumWidth: 480
    minimumHeight: 88
    maximumHeight: 88
    color: "transparent"
    title: "otkati-note"
    visible: false
    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus | Qt.WindowTransparentForInput

    enum PlashkaEvent {
        NothingToSave,
        NoSpace,
        ClipDidNotSave,
        Ordinary,
        Saved,
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
    property var firstLaunchOverlay: null
    property bool firstLaunchPlayed: false
    // Live session only. Tests leave both off, so they never touch Hyprland or the shell shaders.
    property bool grabBind: false
    property bool shellSplash: false
    property string bindMarkPath: "/tmp/otkati-bind"
    readonly property string shellAssets: homePath + "/.local/share/serpantinum/src/assets"
    // Connected monitors. The overlay covers only the ring monitor.
    property var monitors: []
    readonly property bool firstLaunchVisible: firstLaunchOverlay !== null && firstLaunchOverlay.visible
    readonly property string firstLaunchMonitor: firstLaunchOverlay ? firstLaunchOverlay.monitorName : ""
    readonly property int firstLaunchWidth: firstLaunchOverlay ? firstLaunchOverlay.width : 0
    readonly property int firstLaunchHeight: firstLaunchOverlay ? firstLaunchOverlay.height : 0
    readonly property bool firstLaunchFullScreen: firstLaunchOverlay !== null
            && firstLaunchOverlay.coversMonitor
            && firstLaunchOverlay.width === firstLaunchOverlay.monitorWidth
            && firstLaunchOverlay.height === firstLaunchOverlay.monitorHeight
    readonly property string firstLaunchStep: firstLaunchOverlay ? firstLaunchOverlay.step : ""
    readonly property real screenWipe: firstLaunchOverlay ? firstLaunchOverlay.wipe : 0
    readonly property real screenWipeWidth: firstLaunchOverlay ? firstLaunchOverlay.screenWipeWidth : 0
    readonly property var wordsByLanguage: ({
        en: {
            length: "Length",
            resolutionShare: "Resolution share",
            bitrate: "Bitrate",
            encoder: "Encoder",
            h264: "NVIDIA, AMD, Intel, and machines with no hardware encoder",
            hevc: "NVIDIA, including the 3060 Ti and 3070 Ti, and AMD or Intel when the card can do HEVC",
            av1: "NVIDIA from the 40 series and AMD from the RX 7000",
            ringPlace: "Ring place",
            ringMonitor: "Ring monitor",
            bind: "Bind",
            language: "Language",
            theme: "Theme",
            otkatFolder: "Otkat folder",
            switch: "Switch",
            english: "English",
            russian: "Russian",
            nothingToSave: "Nothing to save",
            clipDidNotSave: "Couldn't save the clip",
            saved: "Saved",
            continueButton: "Continue",
            welcome: "Welcome",
            kept: "The last minutes are already kept.",
            bindSaves: "Alt+Z saves a clip."
        },
        ru: {
            length: "Длина",
            resolutionShare: "Доля разрешения",
            bitrate: "Битрейт",
            encoder: "Кодировщик",
            h264: "NVIDIA, AMD, Intel и компьютеры без аппаратного кодировщика",
            hevc: "NVIDIA, включая 3060 Ti и 3070 Ti, и AMD или Intel, если карта умеет HEVC",
            av1: "NVIDIA с 40-й серии и AMD с RX 7000",
            ringPlace: "Место кольца",
            ringMonitor: "Монитор кольца",
            bind: "Бинд",
            language: "Язык",
            theme: "Тема",
            otkatFolder: "Папка откатов",
            switch: "Выключатель",
            english: "Английский",
            russian: "Русский",
            nothingToSave: "Сохранять нечего",
            clipDidNotSave: "Откат не сохранился",
            saved: "Откат сохранён",
            continueButton: "Дальше",
            welcome: "Добро пожаловать",
            kept: "Последние минуты уже пишутся.",
            bindSaves: "Alt+Z сохраняет откат."
        }
    })
    readonly property var words: wordsByLanguage[language] || wordsByLanguage.en
    readonly property var firstLaunchLines: [
        words.welcome,
        words.kept,
        words.bindSaves,
        "FedorRust"
    ]
    readonly property string firstLaunchButton: firstLaunchOverlay ? firstLaunchOverlay.buttonText : ""
    readonly property var typedLines: firstLaunchOverlay ? firstLaunchOverlay.typedLines : []
    readonly property color penguinOutlineColor: firstLaunchOverlay ? firstLaunchOverlay.outlineColor : "transparent"
    readonly property color penguinFillColor: firstLaunchOverlay ? firstLaunchOverlay.fillColor : "transparent"
    readonly property color screenWipeColor: firstLaunchOverlay ? firstLaunchOverlay.wipeColor : "transparent"
    readonly property color firstLaunchTextColor: firstLaunchOverlay ? firstLaunchOverlay.lineColor : "transparent"
    readonly property color firstLaunchButtonColor: firstLaunchOverlay ? firstLaunchOverlay.lineColor : "transparent"
    readonly property real penguinFillLevel: firstLaunchOverlay ? firstLaunchOverlay.fillLevel : 0
    readonly property real penguinFillTop: firstLaunchOverlay ? firstLaunchOverlay.penguinFillTop : 0
    readonly property real penguinFillHeight: firstLaunchOverlay ? firstLaunchOverlay.penguinFillHeight : 0
    readonly property real penguinHeight: firstLaunchOverlay ? firstLaunchOverlay.penguinHeight : 0
    readonly property color settingsColor: settingsSurface ? settingsSurface.color : "transparent"
    readonly property color settingsTextColor: settingsSurface ? settingsSurface.ink : "transparent"
    readonly property bool settingsVisible: settingsSurface ? settingsSurface.open : false
    // The otkat folder the face hands to the ring. ~/Videos/Откаты until the person changes it.
    property string otkatFolder: homePath + "/Videos/Откаты"
    // The bind the face hands to the ring. Alt+Z until the person changes it.
    property string bind: "Alt+Z"
    // Percent of each side of the ring monitor. 100 until the person changes it.
    property int resolutionShare: 100
    // Full sides until the resolution share changes.
    readonly property int sideShare: resolutionShare
    // Picture bitrate in Mbit/s. 25 until the person changes it.
    // It does not follow the resolution share. There is no audio-quality knob.
    property int bitrate: 25
    // The encoder the face hands to the ring. H.264 until the person switches.
    property string encoder: "H.264"
    readonly property var encoderItems: ["H.264", "HEVC", "AV1"]
    // Items the ring says this card can do. H.264 until the ring says otherwise.
    property var offeredEncoders: ["H.264"]
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
    // The length, in seconds, the face hands to the ring. Ten minutes until the person changes it.
    property int length: 600
    // The ring place the face hands to the ring. Memory until the person chooses disk.
    property int ringPlace: Face.RingPlace.Memory
    // The switch the face hands to the ring. On until the person turns the ring off.
    property bool ringOn: true
    // The ring monitor the face hands to the ring. Empty until the person chooses one.
    // The program does not fill this from position, focus, or a connector name.
    property string ringMonitor: ""

    function knob(key) {
        if (key === "ringPlace")
            return ringPlace
        if (key === "encoder")
            return encoder
        if (key === "sideShare")
            return sideShare
        if (key === "bitrate")
            return bitrate
        if (key === "ringMonitor")
            return ringMonitor
        return undefined
    }
    // Reported by the shell. The program language does not follow either value.
    property string shellLanguage: ""
    property string serpantinumLanguage: ""
    readonly property alias plashkaText: label.text
    readonly property alias plashkaColor: background.color
    readonly property int plashkaEvent: plashka.event
    // Gap between the card and the right edge of the ring monitor.
    readonly property int plashkaMargin: 24
    // Below the shell bar. That bar sits at y 4 and is 40 tall, so 52 clears it.
    readonly property int plashkaTop: 52
    // 0 is off the right edge, 1 is in the corner. The card slides; the window does not.
    property real slide: 0
    // The plashka is shown on the current ring monitor.
    readonly property string plashkaMonitor: ringMonitor

    QtObject {
        id: plashka
        property int event: -1
        property bool alarm: false
    }

    function setTheme(next) {
        if (typeof next !== "string" || next.charAt(0) === "#")
            return
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
        const cover = !firstLaunchPlayed && ringMonitorGeometry() !== null
        if (!settingsSurface)
            settingsSurface = settingsWindowComponent.createObject(null, { host: root, covered: cover })
        themeName = name
        themeColors = colors
        themeColor = colors.base
        settingsSurface.color = colors.base
        settingsSurface.ink = colors.text
        settingsSurface.covered = cover
        if (!cover)
            placeSettings(settingsSurface)
        settingsSurface.open = true
    }

    function leftmostScreen() {
        const screens = Qt.application.screens
        let screen = screens.length ? screens[0] : null
        for (let i = 0; i < screens.length; i++) {
            if (screen && (screens[i].virtualX < screen.virtualX || (screens[i].virtualX === screen.virtualX && screens[i].virtualY < screen.virtualY)))
                screen = screens[i]
        }
        return screen
    }

    function placeSettings(surface) {
        const screen = leftmostScreen()
        if (screen)
            surface.screen = screen
        surface.x = screen ? screen.virtualX + 80 : 80
        surface.y = screen ? screen.virtualY + 80 : 80
    }

    function ringMonitorGeometry() {
        if (!ringMonitor) {
            if (monitors.length > 0)
                return null
            return {
                name: Screen.name,
                width: Screen.width,
                height: Screen.height
            }
        }
        for (let i = 0; i < monitors.length; i++) {
            if (monitors[i].name === ringMonitor)
                return monitors[i]
        }
        if (monitors.length > 0)
            return null
        return {
            name: ringMonitor,
            width: Screen.width,
            height: Screen.height
        }
    }

    function placeOnRingMonitor() {
        const margin = plashkaMargin
        if (!ringMonitor) {
            if (!grabBind)
                return
            const place = leftmostScreen()
            if (!place)
                return
            screen = place
            x = place.virtualX + place.width - width - margin
            y = place.virtualY + plashkaTop
            return
        }
        for (let i = 0; i < monitors.length; i++) {
            const monitor = monitors[i]
            if (monitor.name !== ringMonitor)
                continue
            if (monitor.x === undefined || monitor.y === undefined || monitor.width === undefined)
                return
            x = monitor.x + monitor.width - width - margin
            y = monitor.y + plashkaTop
            return
        }
        const named = screenNamed(ringMonitor)
        if (!named)
            return
        screen = named
        x = named.virtualX + named.width - width - margin
        y = named.virtualY + plashkaTop
    }

    function screenNamed(name) {
        const screens = Qt.application.screens
        for (let i = 0; i < screens.length; i++) {
            if (screens[i].name === name)
                return screens[i]
        }
        return null
    }

    function playFirstLaunch() {
        if (firstLaunchPlayed)
            return
        const monitor = ringMonitorGeometry()
        if (!monitor)
            return
        firstLaunchPlayed = true
        rememberTheme()
        if (firstLaunchOverlay)
            return
        const overlay = firstLaunchComponent.createObject(null, {
            monitorName: monitor.name,
            monitorWidth: monitor.width,
            monitorHeight: monitor.height,
            width: monitor.width,
            height: monitor.height,
            coversMonitor: true,
            lines: Qt.binding(function() { return root.firstLaunchLines }),
            wipeColor: Qt.binding(function() { return root.themeColors.base }),
            lineColor: Qt.binding(function() { return root.themeColors.text }),
            outlineColor: Qt.binding(function() { return root.themeColors.text }),
            fillColor: Qt.binding(function() { return root.themeColors.blue }),
            buttonText: Qt.binding(function() { return root.words.continueButton }),
            assetsDir: root.shellSplash ? root.shellAssets : "",
            crustColor: Qt.binding(function() { return root.themeColors.crust || root.themeColors.base }),
            surfaceColor: Qt.binding(function() { return root.themeColors.surface1 || root.themeColors.base }),
            orbColor: Qt.binding(function() { return root.themeColors.mauve || root.themeColors.blue }),
            mutedColor: Qt.binding(function() { return root.themeColors.subtext0 || root.themeColors.text })
        })
        if (!overlay) {
            if (settingsSurface)
                settingsSurface.covered = false
            return
        }
        const screen = screenNamed(monitor.name)
        if (screen) {
            overlay.screen = screen
            overlay.visibility = Window.FullScreen
            if (overlay.width !== monitor.width || overlay.height !== monitor.height) {
                overlay.visibility = Window.Windowed
                overlay.width = monitor.width
                overlay.height = monitor.height
            }
        }
        overlay.continued.connect(continueFirstLaunch)
        overlay.escaped.connect(escapeFirstLaunch)
        firstLaunchOverlay = overlay
    }

    function overlayCovers(name) {
        return firstLaunchVisible && firstLaunchMonitor === name
    }

    function continueFirstLaunch() {
        dismissFirstLaunch()
    }

    function escapeFirstLaunch() {
        dismissFirstLaunch()
    }

    function dismissFirstLaunch() {
        if (firstLaunchOverlay)
            firstLaunchOverlay.visible = false
        if (!settingsSurface)
            return
        placeSettings(settingsSurface)
        settingsSurface.covered = false
    }

    function storedTheme() {
        const stored = readJson(themeStorePath)
        if (!stored || !stored.name)
            return null
        const colors = colorSet(stored.colors)
        if (!colors)
            return null
        if (stored.firstLaunch === true)
            firstLaunchPlayed = true
        return { name: stored.name, colors: colors }
    }

    function rememberTheme() {
        if (!themeStorePath || !themeName)
            return
        const payload = {
            name: themeName,
            colors: themeColors
        }
        if (firstLaunchPlayed)
            payload.firstLaunch = true
        Files.writeText(themeStorePath, JSON.stringify(payload))
    }

    function openSettings() {
        loadCatalogue()
        const stored = storedTheme()
        if (stored) {
            applyTheme(stored.name, stored.colors)
            playFirstLaunch()
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
        if (!matugen && !theme.activePreset)
            return
        applyTheme(matugen ? (theme.activePreset || "Matugen") : theme.activePreset, colors)
        rememberTheme()
        playFirstLaunch()
    }

    function isMatugen(theme) {
        if (theme.matugen !== undefined)
            return theme.matugen === true
        return theme.theme === "matugen"
                || theme.name === "matugen"
                || theme.currentTheme === "matugen"
                || theme.current === "matugen"
                || theme.activePreset === "Matugen"
    }

    Component.onCompleted: Qt.quitOnLastWindowClosed = false

    Component.onDestruction: {
        if (settingsSurface)
            settingsSurface.destroy()
        if (firstLaunchOverlay)
            firstLaunchOverlay.destroy()
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

    function setResolutionShare(percent) {
        if (percent < 50 || percent > 100 || percent % 10 !== 0)
            return
        resolutionShare = percent
    }

    function setBitrate(mbit) {
        if (mbit < 8 || mbit > 80 || mbit % 1 !== 0)
            return
        bitrate = mbit
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

    function setSwitch(ringOn) {
        root.ringOn = ringOn
    }

    function setRingMonitor(name) {
        ringMonitor = name
        placeOnRingMonitor()
    }

    function setShellLanguage(next) {
        shellLanguage = next
    }

    function setSerpantinumLanguage(next) {
        serpantinumLanguage = next
    }

    function plashkaPhrase(event) {
        if (event === Face.PlashkaEvent.NothingToSave)
            return words.nothingToSave
        if (event === Face.PlashkaEvent.ClipDidNotSave)
            return words.clipDidNotSave
        if (event === Face.PlashkaEvent.Saved)
            return words.saved
        return ""
    }

    function setOfferedEncoders(items) {
        offeredEncoders = items
    }

    function encoderSelectable(item) {
        for (let i = 0; i < offeredEncoders.length; ++i) {
            if (offeredEncoders[i] === item)
                return true
        }
        return false
    }

    function chooseEncoder(item) {
        if (!encoderSelectable(item))
            return
        encoder = item
    }

    function encoderMark(item) {
        if (item === "H.264")
            return words.h264
        if (item === "HEVC")
            return words.hevc
        if (item === "AV1")
            return words.av1
        return ""
    }

    function settingLabel(key) {
        return words[key] || ""
    }

    function languageChoice(choice) {
        if (choice === "en")
            return words.english
        if (choice === "ru")
            return words.russian
        return ""
    }

    function chooseLanguage(choice) {
        if (choice === "en" || choice === "ru")
            setLanguage(choice)
    }

    Item {
        id: settings
        visible: false

        Text { text: root.words.length }
        Text { text: root.words.resolutionShare }
        Text { text: root.words.bitrate }
        Text { text: root.words.encoder }
        Text { text: root.words.h264 }
        Text { text: root.words.hevc }
        Text { text: root.words.av1 }
        Text { text: root.words.ringPlace }
        Text { text: root.words.ringMonitor }
        Text { text: root.words.bind }
        Text { text: root.words.language }
        Text { text: root.words.theme }
        Text { text: root.words.otkatFolder }
        Text { text: root.words.switch }
        Text {
            text: root.words.english
            MouseArea {
                anchors.fill: parent
                onClicked: root.chooseLanguage("en")
            }
        }
        Text {
            text: root.words.russian
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
    }

    function ordinaryEvent(event) {
        return event === Face.PlashkaEvent.Ordinary
                || event === Face.PlashkaEvent.Saved
                || event === Face.PlashkaEvent.NoAccess
                || event === Face.PlashkaEvent.FolderNotCreated
                || event === Face.PlashkaEvent.RingOff
    }

    function eventFromRing(reply) {
        if (reply === "nothing")
            return Face.PlashkaEvent.NothingToSave
        if (reply === "no-access")
            return Face.PlashkaEvent.NoAccess
        if (reply === "folder")
            return Face.PlashkaEvent.FolderNotCreated
        if (reply === "no-space")
            return Face.PlashkaEvent.NoSpace
        if (reply === "clip-did-not-save")
            return Face.PlashkaEvent.ClipDidNotSave
        if (reply === "ring-off")
            return Face.PlashkaEvent.RingOff
        if (reply.indexOf("saved\t") === 0)
            return Face.PlashkaEvent.Saved
        return -1
    }

    function pullRingMonitor() {
        const reply = Files.ask("query")
        if (!reply)
            return
        const name = reply.split("\t")[1]
        if (name && name !== "-" && name !== ringMonitor)
            setRingMonitor(name)
    }

    function raiseBindPlashka() {
        const reply = Files.ask("bind\t" + bind)
        const event = eventFromRing(reply)
        if (event < 0)
            return
        showPlashka(event)
        if (event !== Face.PlashkaEvent.Saved)
            return
        const committed = Files.ask("commit")
        if (committed === "clip-did-not-save")
            showPlashka(Face.PlashkaEvent.ClipDidNotSave)
    }

    Timer {
        interval: 120
        repeat: true
        running: root.grabBind
        onTriggered: {
            root.pullRingMonitor()
            const mark = Files.readText(root.bindMarkPath)
            if (!mark || mark.trim().length === 0)
                return
            Files.writeText(root.bindMarkPath, "")
            root.raiseBindPlashka()
        }
    }

    Shortcut {
        sequence: "Alt+Z"
        context: Qt.ApplicationShortcut
        enabled: root.grabBind
        onActivated: root.raiseBindPlashka()
    }

    function showPlashka(event) {
        if (!alarmEvent(event) && !ordinaryEvent(event))
            return
        plashka.event = event
        plashka.alarm = alarmEvent(event)
        label.text = plashkaPhrase(event)
        slideIn.stop()
        slideOut.stop()
        slide = 0
        placeOnRingMonitor()
        visible = true
        slideIn.start()
        hideTimer.restart()
    }

    NumberAnimation {
        id: slideIn
        target: root
        property: "slide"
        to: 1
        duration: 320
        easing.type: Easing.OutCubic
    }

    NumberAnimation {
        id: slideOut
        target: root
        property: "slide"
        to: 0
        duration: 220
        easing.type: Easing.InCubic
        onFinished: {
            if (root.slide < 0.01)
                root.visible = false
        }
    }

    Timer {
        id: hideTimer
        interval: 2000
        onTriggered: slideOut.start()
    }

    Component {
        id: settingsWindowComponent
        Window {
            id: settingsWindow
            property var host: null
            property color ink: "transparent"
            property bool open: false
            property bool covered: false
            title: "otkati-settings"
            flags: Qt.Window | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint
            visible: open && !covered
            width: 520
            height: 560
            minimumWidth: 520
            maximumWidth: 520
            minimumHeight: 560
            maximumHeight: 560
            x: 80
            y: 80

            Column {
                anchors.fill: parent
                anchors.margins: 28
                spacing: 22
                visible: !settingsWindow.covered && settingsWindow.host

                Text {
                    text: settingsWindow.host.words.length
                    color: settingsWindow.ink
                    font.pixelSize: 13
                }
                Row {
                    spacing: 16
                    Text {
                        text: settingsWindow.host.length
                        color: settingsWindow.ink
                        font.pixelSize: 28
                        font.bold: true
                    }
                    Rectangle {
                        width: 36
                        height: 36
                        radius: 8
                        color: Qt.rgba(1, 1, 1, 0.08)
                        Text { anchors.centerIn: parent; text: "−"; color: settingsWindow.ink; font.pixelSize: 20 }
                        MouseArea {
                            anchors.fill: parent
                            onClicked: settingsWindow.host.setLength(settingsWindow.host.length - 1)
                        }
                    }
                    Rectangle {
                        width: 36
                        height: 36
                        radius: 8
                        color: Qt.rgba(1, 1, 1, 0.08)
                        Text { anchors.centerIn: parent; text: "+"; color: settingsWindow.ink; font.pixelSize: 20 }
                        MouseArea {
                            anchors.fill: parent
                            onClicked: settingsWindow.host.setLength(settingsWindow.host.length + 1)
                        }
                    }
                }

                Text {
                    text: settingsWindow.host.words.resolutionShare
                    color: settingsWindow.ink
                    font.pixelSize: 13
                }
                Row {
                    spacing: 16
                    Text {
                        text: settingsWindow.host.resolutionShare
                        color: settingsWindow.ink
                        font.pixelSize: 28
                        font.bold: true
                    }
                    Rectangle {
                        width: 36
                        height: 36
                        radius: 8
                        color: Qt.rgba(1, 1, 1, 0.08)
                        Text { anchors.centerIn: parent; text: "−"; color: settingsWindow.ink; font.pixelSize: 20 }
                        MouseArea {
                            anchors.fill: parent
                            onClicked: settingsWindow.host.setResolutionShare(settingsWindow.host.resolutionShare - 10)
                        }
                    }
                    Rectangle {
                        width: 36
                        height: 36
                        radius: 8
                        color: Qt.rgba(1, 1, 1, 0.08)
                        Text { anchors.centerIn: parent; text: "+"; color: settingsWindow.ink; font.pixelSize: 20 }
                        MouseArea {
                            anchors.fill: parent
                            onClicked: settingsWindow.host.setResolutionShare(settingsWindow.host.resolutionShare + 10)
                        }
                    }
                }

                Text {
                    text: settingsWindow.host.words.bitrate
                    color: settingsWindow.ink
                    font.pixelSize: 13
                }
                Row {
                    spacing: 16
                    Text {
                        text: settingsWindow.host.bitrate
                        color: settingsWindow.ink
                        font.pixelSize: 28
                        font.bold: true
                    }
                    Rectangle {
                        width: 36
                        height: 36
                        radius: 8
                        color: Qt.rgba(1, 1, 1, 0.08)
                        Text { anchors.centerIn: parent; text: "−"; color: settingsWindow.ink; font.pixelSize: 20 }
                        MouseArea {
                            anchors.fill: parent
                            onClicked: settingsWindow.host.setBitrate(settingsWindow.host.bitrate - 1)
                        }
                    }
                    Rectangle {
                        width: 36
                        height: 36
                        radius: 8
                        color: Qt.rgba(1, 1, 1, 0.08)
                        Text { anchors.centerIn: parent; text: "+"; color: settingsWindow.ink; font.pixelSize: 20 }
                        MouseArea {
                            anchors.fill: parent
                            onClicked: settingsWindow.host.setBitrate(settingsWindow.host.bitrate + 1)
                        }
                    }
                }

                Text {
                    text: settingsWindow.host.words.bind + "  " + settingsWindow.host.bind
                    color: settingsWindow.ink
                    font.pixelSize: 18
                }
                Row {
                    spacing: 18
                    Text {
                        text: settingsWindow.host.words.english
                        color: settingsWindow.ink
                        font.pixelSize: 16
                        font.bold: settingsWindow.host.language === "en"
                        MouseArea { anchors.fill: parent; onClicked: settingsWindow.host.chooseLanguage("en") }
                    }
                    Text {
                        text: settingsWindow.host.words.russian
                        color: settingsWindow.ink
                        font.pixelSize: 16
                        font.bold: settingsWindow.host.language === "ru"
                        MouseArea { anchors.fill: parent; onClicked: settingsWindow.host.chooseLanguage("ru") }
                    }
                }
            }
        }
    }

    Component {
        id: firstLaunchComponent
        FirstLaunch {}
    }

    Item {
        anchors.fill: parent
        clip: true

        Rectangle {
            id: background
            width: parent.width
            height: parent.height
            radius: 18
            x: (1 - root.slide) * width
            color: plashka.alarm ? "#ff0000" : root.themeColor
            border.width: 1
            border.color: label.color

            Text {
                id: label
                anchors.centerIn: parent
                width: parent.width - 48
                horizontalAlignment: Text.AlignHCenter
                elide: Text.ElideRight
                color: plashka.alarm ? "#ffffff" : (root.themeColors.text ? root.themeColors.text : "#ffffff")
                font.pixelSize: 22
                font.bold: true
                text: ""
            }
        }
    }
}
