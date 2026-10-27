import QtQuick
import Otkati 1.0

Face {
    grabBind: true
    Component.onCompleted: {
        Qt.quitOnLastWindowClosed = false
        openSettings()
    }
}
