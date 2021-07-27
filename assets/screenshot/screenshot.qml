import QtQuick 2.12
import QtQuick.Window 2.12
import QtQuick.Shapes 1.12

Window {
    visible: true
    visibility: Window.FullScreen

    Shape {
        width: parent.width
        height: parent.height
        anchors.centerIn: parent
    }

    Shortcut {
        sequence: StandardKey.Quit
        onActivated: Qt.quit()
    }
}
