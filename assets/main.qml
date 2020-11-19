import QtQuick 2.15
import QtQuick.Window 2.15
import Qt.labs.platform 1.1

Window
{
    id: window
    visible: false
    width: 400
    height: 200
    title: qsTr("Hello World")

    SystemTrayIcon {
        visible: true
        iconSource: "qrc:/images/icon.png"

        onActivated: {
            window.show()
            window.raise()
            window.requestActivate()
        }
    }
}
