import QtQuick 2.12
import QtQuick.Window 2.12

Window {
    id: window
    flags: Qt.CustomizeWindowHint

    width: screen.desktopAvailableWidth * 0.6
    height: 48
    x: (screen.desktopAvailableWidth - width) / 2
    y: 10

    visible: true
    title: qsTr("my-tool")

    TextInput {
        width: window.width
        height: window.height
        text: "<b>Hello</b> <i>World!</i>"
        font.family: "Hack"
        font.pointSize: 20

        horizontalAlignment: TextEdit.AlignHCenter
        verticalAlignment: TextEdit.AlignVCenter
        focus: true
    }
}
