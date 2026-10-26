import QtQuick
import Otkati 1.0

Face {
    grabBind: true
    shellSplash: true
    Component.onCompleted: {
        Qt.quitOnLastWindowClosed = false
        openSettings()
    }
}
