import QtQuick
import Otkati 1.0

Face {
    grabBind: true
    persistEncoder: true
    persistTheme: true
    Component.onCompleted: {
        Qt.quitOnLastWindowClosed = false
        loadKnobs()
        openSettings()
    }
}
